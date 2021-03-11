use crate::command::Command;
use crate::command::{AbortReason, Instruction, Status};
use crate::error::ApiError;
use sqlx::postgres::PgPool;

impl Command {
    // If the saftey abort is given we need to create a new abort command
    // and cancell all of the pending commands that have time_instruction
    // in the past
    pub async fn saftey_abort(conn: &PgPool, robot_serial_number: &str) -> Result<Self, ApiError> {
        Self::cancel_all_previous_pending(conn, robot_serial_number).await?;

        let time_now = chrono::Utc::now();
        Self::new(
            conn,
            robot_serial_number,
            time_now,
            time_now,
            &Instruction::Abort(AbortReason::Safety),
            &Status::Pending,
        )
        .await
    }

    // Mark the given command as cancelled and will make a new abort command
    // Mark it as in progress and return it to the robot
    pub async fn new_low_battery(
        conn: &PgPool,
        robot_serial_number: &str,
    ) -> Result<Self, ApiError> {
        Self::cancel_all_previous_pending(conn, robot_serial_number).await?;

        let time_now = chrono::Utc::now();
        Self::new(
            conn,
            robot_serial_number,
            time_now,
            time_now,
            &Instruction::Abort(AbortReason::LowBattery),
            &Status::Pending,
        )
        .await
    }

    pub async fn low_battery_abort(&self, conn: &PgPool) -> Result<Self, ApiError> {
        Self::new_low_battery(conn, &self.robot_serial_number).await
    }

    // Cancel all of the pending commands set to run before the abort is made
    pub async fn cancel_all_previous_pending(
        conn: &PgPool,
        robot_serial_number: &str,
    ) -> Result<(), ApiError> {
        // Get all of the commands that are ongoing and cancel them
        for c in Self::ongoing(conn, robot_serial_number).await? {
            c.cancel(conn).await?;
        }

        let all_pending_commands = Self::get_all_pending(conn, robot_serial_number).await?;

        let time_now = chrono::Utc::now();
        for c in all_pending_commands {
            if c.time_instruction < time_now {
                c.cancel(conn).await?;
            }
        }
        Ok(())
    }
}
