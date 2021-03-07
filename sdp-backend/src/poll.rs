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
    pub current_command_id: i64,
    pub current_command_status: Status,
    pub battery_level: i64,
}

impl Poll {
    pub async fn poll(conn: &PgPool, robot_status: &Self) -> Result<Command, ApiError> {
        let mut robot_status = robot_status.clone();

        // Check the battery of the robot
        if !robot_status.check_battery().await {
            robot_status.instruction = Abort(AbortReason::LowBattery);
        }

        // Get the previous commands for the robot
        let prev_command = Command::current(conn, &robot_status.robot_serial_number).await?;

        // Get the pending command for the robot, if the pending is the same as the current
        // task set it to None.
        let pending_command =
            match Command::pending(conn, &prev_command.robot_serial_number).await? {
                Some(pending) if pending == prev_command => None,
                Some(pending) => Some(pending),
                _ => None,
            };

        // println!("\nPrev Command: {:?}\n", prev_command);
        // println!("Pending Command: {:?}\n", pending_command);
        // println!("Robot Status: {:?}\n", robot_status);

        match (
            &prev_command.instruction,
            &pending_command,
            &robot_status.instruction,
        ) {
            // If there are no pending commands, and the new robot state is the same as the last keep doing the previous task
            (prev, None, new) if prev == new => {
                // println!("--- OPTION 1 ---");
                Ok(prev_command)
            }

            // Doing a task that has now completed, mark as complete and idle
            (Task(_), None, Idle) => {
                // println!("--- OPTION 2 ---");
                prev_command.complete(conn).await.ok();
                Command::idle(conn, &robot_status.robot_serial_number).await
            }

            // Doing a task that has now completed, mark as complete and do the pending task
            (Task(_), Some(pending), Idle) => {
                // println!("--- OPTION 3 ---");
                prev_command.complete(conn).await.ok();
                Ok(pending.clone())
            }

            // If the robot has aborted for some reason, mark the task as complete
            // then abort
            (_, _, Abort(reason)) => {
                // println!("--- OPTION 4 ---");
                prev_command.complete(conn).await.ok();
                Command::abort(conn, &robot_status.robot_serial_number, reason).await
            }
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Init {
    pub robot_serial_number: String,
    pub battery_level: i64,
}

            // IF we are previously aborted, set the state to idle
            (Abort(_), None, Idle) => {
                // println!("--- OPTION 5 ---");
                prev_command.complete(conn).await.ok();
                Command::idle(conn, &robot_status.robot_serial_number).await
            }

            // IF we are previously aborted, set the state to idle
            (Abort(_), Some(pending), Idle) => {
                // println!("--- OPTION 6 ---");
                prev_command.complete(conn).await.ok();
                Ok(pending.clone())
            }

            (Idle, Some(pending), Idle) => {
                // println!("--- OPTION 7 ---");
                prev_command.complete(conn).await.ok();
                Ok(pending.clone())
            }

            _ => Err(ApiError::CmdInstructionNotSupported),
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
