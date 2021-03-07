use crate::command::Command;
use crate::command::{Instruction, Status};
use crate::error::ApiError;
use chrono::Utc;
use sqlx::postgres::PgPool;

impl Command {
    pub async fn completed(&self, conn: &PgPool) -> Result<Self, ApiError> {
        Self::update(
            conn,
            self.command_id,
            &self.robot_serial_number,
            self.time_issued,
            self.time_instruction,
            self.instruction.clone(),
            Status::Completed,
        )
        .await
    }

    pub async fn in_progress(&self, conn: &PgPool) -> Result<Self, ApiError> {
        Self::update(
            conn,
            self.command_id,
            &self.robot_serial_number,
            self.time_issued,
            self.time_instruction,
            self.instruction.clone(),
            Status::InProgress,
        )
        .await
    }

    pub async fn cancel(&self, conn: &PgPool) -> Result<Self, ApiError> {
        Self::update(
            conn,
            self.command_id,
            &self.robot_serial_number,
            self.time_issued,
            self.time_instruction,
            self.instruction.clone(),
            Status::Cancelled,
        )
        .await
    }

    // Updates the given command with the new status
    pub async fn update_status(
        &self,
        conn: &PgPool,
        new_status: &Status,
    ) -> Result<Self, ApiError> {
        Self::update(
            conn,
            self.command_id,
            &self.robot_serial_number,
            self.time_issued,
            self.time_instruction,
            self.instruction.clone(),
            new_status.clone(),
        )
        .await
    }

    pub async fn update(
        conn: &PgPool,
        command_id: i64,
        robot_serial_number: &str,
        time_issued: chrono::DateTime<Utc>,
        time_instruction: chrono::DateTime<Utc>,
        instruction: Instruction,
        status: Status,
    ) -> Result<Command, ApiError> {
        sqlx::query!(
            r#"
        UPDATE Commands C
        SET robot_serial_number = $1,
        time_issued = $2,
        time_instruction = $3,
        instruction = $4,
        status = $5
        WHERE C.command_id = $6

                        "#,
            robot_serial_number,
            time_issued,
            time_instruction,
            instruction.to_string(),
            status.to_string(),
            command_id,
        )
        .execute(conn)
        .await
        .map_err(|_| ApiError::DatabaseConnFailed)?;

        Self::get_by_id(conn, command_id).await
    }
}
