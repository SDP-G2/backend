use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPool;

use crate::command::{
    AbortReason, Command, Instruction,
    Instruction::{Abort, Idle, Task},
};
use crate::error::ApiError;

const MINIMUM_BATTERY_LEVEL: i64 = 50;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Poll {
    pub robot_serial_number: String,
    pub instruction: Instruction,
    battery_level: i64,
}

impl Poll {
    pub async fn poll(conn: &PgPool, next_command: &Self) -> Result<Command, ApiError> {
        // Check the battery of the robot
        if !next_command.check_battery().await {
            return Ok(Command::abort(
                conn,
                &next_command.robot_serial_number,
                &AbortReason::LowBattery,
            )
            .await?);
        }

        // If the new commands is the same as the previous keep doing it
        let prev_command = Command::current(conn, &next_command.robot_serial_number).await?;
        if &prev_command.instruction == &next_command.instruction {
            return Ok(prev_command);
        }

        match &next_command.instruction {
            Abort(reason) => Command::abort(conn, &next_command.robot_serial_number, reason).await,
            Task(task) => Command::task(conn, &next_command.robot_serial_number, task).await,
            Idle => Command::idle(conn, &next_command.robot_serial_number).await,
            _unsupported_instruction => Err(ApiError::CmdInstructionNotSupported),
        }
    }

    /// Checks the current battery level of the Robot
    ///
    /// If the battery level is not sufficent the robot will
    /// be told to abort due to low battery.
    async fn check_battery(&self) -> bool {
        self.battery_level >= 0
            && self.battery_level > MINIMUM_BATTERY_LEVEL
            && self.battery_level <= 100
    }
}
