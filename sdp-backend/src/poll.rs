use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPool;

use crate::command::Instruction::{Abort, Idle, Task};
use crate::command::{Command, Status};
use crate::error::ApiError;
use crate::robot::Robot;

const MINIMUM_BATTERY_LEVEL: i64 = 50;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Poll {
    pub robot_serial_number: String,
    pub command_id: i64,
    pub status: Status,
    pub battery_level: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Init {
    pub robot_serial_number: String,
    pub battery_level: i64,
}

impl Poll {
    pub async fn poll(conn: &PgPool, poll: &Self) -> Result<Command, ApiError> {
        // Update the battery value stored in the database
        Robot::set_battery(conn, &poll.robot_serial_number, poll.battery_level).await?;

        // Get the current command from the database
        let current_command = match Command::get_by_id(conn, poll.command_id).await {
            Ok(c) if c.status.cancelled() => {
                match Command::pending(conn, &poll.robot_serial_number).await? {
                    Some(abort) => return Ok(abort),
                    _ => return Err(ApiError::CmdInstructionNotSupported),
                }
            }
            Ok(c) => c,
            Err(e) => return Err(e),
        };

        match &current_command.instruction {
            // If we are doing a task and it had not completed,
            // update the status and keep doing it
            Task(_) if !poll.status.is_completed() => {
                current_command.update_status(conn, &poll.status).await
            }

            // If we are doing a task and it has completed do the next
            Task(_) if poll.status.is_completed() => {
                current_command.update_status(conn, &poll.status).await?;
                match Command::pending(conn, &poll.robot_serial_number).await? {
                    Some(c) => Ok(c.in_progress(conn).await?),
                    None => {
                        Command::new_idle(conn, &poll.robot_serial_number)
                            .await?
                            .in_progress(conn)
                            .await
                    }
                }
            }

            // If we are idle check that the battery level is valid
            Idle if !poll.check_battery().await => current_command.low_battery_abort(conn).await,

            // If we are idle and battery is ok check for commands, if there are non stay idle
            Idle => match Command::pending(conn, &poll.robot_serial_number).await? {
                Some(c) => {
                    current_command.completed(conn).await?;
                    Ok(c.in_progress(conn).await?)
                }
                None => Ok(current_command),
            },

            // If we get the Abort instruction, just update the status of the command
            Abort(_) => current_command.update_status(conn, &poll.status).await,
            _unsupported => Err(ApiError::CmdInstructionNotSupported),
        }
    }

    /// Checks the current battery level of the Robot
    ///
    /// If the battery level is not sufficent the robot will
    /// be told to abort due to low battery.
    async fn check_battery(&self) -> bool {
        check_battery(self.battery_level)
    }
}

fn check_battery(battery_level: i64) -> bool {
    (battery_level >= 0 && battery_level <= 100) && (battery_level > MINIMUM_BATTERY_LEVEL)
}

// When the robot first turns on it will have no knowledge of previous communications
impl Init {
    pub async fn init(conn: &PgPool, init: &Self) -> Result<Command, ApiError> {
        if !init.check_battery() {
            return Command::new_low_battery(conn, &init.robot_serial_number).await;
        }

        let polling_command = Command::init_command(conn, &init.robot_serial_number).await?;
        println!("Init Command {:?}", polling_command);

        polling_command.in_progress(conn).await?;

        let poll = Poll {
            robot_serial_number: init.robot_serial_number.clone(),
            command_id: polling_command.command_id,
            status: polling_command.status,
            battery_level: init.battery_level,
        };

        Poll::poll(conn, &poll).await
    }

    fn check_battery(&self) -> bool {
        check_battery(self.battery_level)
    }
}
