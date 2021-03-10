use crate::command::Command;
use crate::command::{AbortReason, Instruction, Status};
use crate::error::ApiError;
use chrono::Utc;
use sqlx::postgres::PgPool;

impl Command {
    pub async fn new(
        conn: &PgPool,
        robot_serial_number: &str,
        time_issued: chrono::DateTime<Utc>,
        time_instruction: chrono::DateTime<Utc>,
        instruction: &Instruction,
        status: &Status,
    ) -> Result<Self, ApiError> {
        // Check the times of the command, returning an error if required
        match Self::check_time(time_issued, time_instruction) {
            (Some(e), None) | (None, Some(e)) => return Err(e),
            _no_errors => (),
        }

        sqlx::query!(
            r#"
        INSERT INTO Commands (robot_serial_number, time_issued, time_instruction, instruction, status)
        VALUES ( $1, $2, $3, $4, $5)
        RETURNING command_id
                "#,
            robot_serial_number,
            time_issued,
            time_instruction,
            instruction.to_string(),
            status.to_string()
        )
        .fetch_one(conn)
        .await
        .map_err(|_| ApiError::DatabaseConnFailed)
        .map( |c|
            Ok(Self {
                command_id: c.command_id,
                robot_serial_number: robot_serial_number.to_string(),
                time_issued,
                time_instruction,
                instruction: instruction.clone(),
                status: status.clone(),
            })
        )?
    }

    pub async fn new_idle(conn: &PgPool, robot_serial_number: &str) -> Result<Self, ApiError> {
        let time_now = chrono::Utc::now();
        Self::new(
            conn,
            robot_serial_number,
            time_now,
            time_now,
            &Instruction::Idle,
            &Status::Pending,
        )
        .await
    }

    pub async fn batch_new(
        conn: &PgPool,
        robot_serial_number: &str,
        time_issued: chrono::DateTime<Utc>,
        time_instruction: Vec<chrono::DateTime<Utc>>,
        instruction: &Instruction,
    ) -> Result<Vec<Command>, ApiError> {
        // Check if the command is a SafetyAbort
        if instruction == &Instruction::Abort(AbortReason::Safety) {
            return Ok(vec![
                Command::saftey_abort(conn, robot_serial_number).await?,
            ]);
        }

        let mut commands = Vec::new();
        for t in time_instruction {
            let cmd = Self::new(
                conn,
                robot_serial_number,
                time_issued,
                t,
                instruction,
                &Status::Pending,
            )
            .await?;
            println!("{:?}", cmd);
            commands.push(cmd);
        }

        Ok(commands)
    }
}
