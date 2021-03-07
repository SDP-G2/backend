use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPool;

use crate::command::Instruction::{Idle, Task};
use crate::command::{Command, Status};
use crate::error::ApiError;

const MINIMUM_BATTERY_LEVEL: i64 = 50;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Poll {
    pub robot_serial_number: String,
    pub current_command_id: i64,
    pub current_command_status: Status,
    pub battery_level: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Init {
    pub robot_serial_number: String,
    pub battery_level: i64,
}

impl Poll {
    pub async fn poll(conn: &PgPool, poll: &Self) -> Result<Command, ApiError> {
        // Get the current command from the database
        let current_command = match Command::get_by_id(conn, poll.current_command_id).await {
            Ok(c) if c.status.cancelled() => {
                return Command::pending(conn, &poll.robot_serial_number)
                    .await
                    .map(|c| c.expect("The abort command should have been inserted"));
            }
            Ok(c) => c,
            Err(_) => return Err(ApiError::DatabaseConnFailed),
        };

        match &current_command.instruction {
            Task(_) => {
                current_command
                    .update_status(conn, &poll.current_command_status)
                    .await
            }
            Idle if !poll.check_battery().await => current_command.low_battery_abort(conn).await,
            Idle => {
                // current_command.completed(conn).await
                match Command::pending(conn, &poll.robot_serial_number).await? {
                    Some(cmd) => {
                        current_command.completed(conn).await?;
                        cmd.in_progress(conn).await?;
                        Ok(cmd)
                    }
                    None => Ok(current_command),
                }
            }
            _unsupported => Err(ApiError::CmdInstructionNotSupported),
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

// When the robot first turns on it will have no knowledge of previous communications
impl Init {
    pub async fn init(conn: &PgPool, init: &Self) -> Result<Command, ApiError> {
        let polling_command = match Command::pending(conn, &init.robot_serial_number).await? {
            Some(c) => c,
            None => Command::new_idle(conn, &init.robot_serial_number).await?,
        };

        let poll = Poll {
            robot_serial_number: init.robot_serial_number.clone(),
            current_command_id: polling_command.command_id,
            current_command_status: polling_command.status,
            battery_level: init.battery_level,
        };

        Poll::poll(conn, &poll).await
    }
}
