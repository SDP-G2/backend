use crate::error::ApiError;
use chrono::{serde::ts_seconds, Utc};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPool;

mod time;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Command {
    pub command_id: i64,
    pub robot_serial_number: String,
    #[serde(with = "ts_seconds")]
    time_issued: chrono::DateTime<Utc>,
    #[serde(with = "ts_seconds")]
    time_instruction: chrono::DateTime<Utc>,
    pub instruction: Instruction,
    pub completed: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum CleaningPattern {
    ZigZag,
    Circular,
}
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum AbortReason {
    LowBattery,
    Saftey,
    Obstacle,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Instruction {
    Continue,
    Pause,
    Abort(AbortReason),
    Task(CleaningPattern),
    Idle,
}

impl Command {
    // Get the current task the robot is doing
    pub async fn current(conn: &PgPool, robot_serial_number: &str) -> Result<Self, ApiError> {
        sqlx::query!(
            r#"
SELECT * FROM Commands C
NATURAL JOIN
(SELECT MIN(C1.time_instruction) AS time_instruction,
        $1 AS robot_serial_number
FROM Commands C1
WHERE C1.robot_serial_number = $1 AND
      C1.completed = false) MinTimeInstruction
               "#,
            robot_serial_number
        )
        .fetch_one(conn)
        .await
        .map(|cmd| Self {
            command_id: cmd.command_id,
            robot_serial_number: cmd.robot_serial_number,
            time_issued: cmd.time_issued,
            time_instruction: cmd.time_issued,
            instruction: serde_json::from_str(&cmd.instruction)
                .unwrap_or(Instruction::Abort(AbortReason::Saftey)),
            completed: cmd.completed,
        })
        .map_err(|e| {
            println!("Command Latest: {:?}", e);
            ApiError::DatabaseConnFailed
        })
    }

    /// Checks to see if there are any pending command for this robot
    pub async fn pending(
        conn: &PgPool,
        robot_serial_number: &str,
    ) -> Result<Option<Command>, ApiError> {
        let mut pending_commands = Vec::new();
        let all_cmd = Self::get_all_commands(conn, robot_serial_number).await?;

        for cmd in &all_cmd {
            if !cmd.completed {
                pending_commands.push(cmd.clone());
            }
        }

        match pending_commands.get(0) {
            Some(cmd) if cmd.valid_time_instruction() => Ok(Some(cmd.clone())),
            _ => Ok(None),
        }
    }

    pub async fn complete(&self, conn: &PgPool) -> Result<(), ApiError> {
        sqlx::query!(
            r#"
UPDATE Commands C
SET completed = true
WHERE C.command_id= $1
               "#,
            self.command_id
        )
        .execute(conn)
        .await
        .map_err(|_| ApiError::DatabaseConnFailed)?;

        Ok(())
    }

    pub fn valid_time_instruction(&self) -> bool {
        let time_difference = (chrono::Utc::now() - self.time_instruction)
            .num_seconds()
            .abs();

        time_difference < TIME_INSTRUCTION_BUFFER
    }
}

impl Command {
    // Abort the current task with the given reason
    pub async fn abort(
        conn: &PgPool,
        robot_serial_number: &str,
        reason: &AbortReason,
    ) -> Result<Self, ApiError> {
        // Create a new command with the current time
        let time_now = chrono::Utc::now();

        Ok(Command::new(
            conn,
            robot_serial_number,
            time_now,
            time_now,
            &Instruction::Abort(reason.clone()),
        )
        .await?)
    }

    }
}

impl Command {
    pub async fn get_all_commands(
        conn: &PgPool,
        robot_serial_number: &str,
    ) -> Result<Vec<Command>, ApiError> {
        sqlx::query!(
            r#"
SELECT * FROM Commands C
WHERE C.robot_serial_number = $1
ORDER BY C.time_instruction DESC
               "#,
            robot_serial_number
        )
        .fetch_all(conn)
        .await
        .map(|cmds| {
            let mut pending = Vec::new();

            for c in cmds {
                pending.push(Self {
                    command_id: c.command_id,
                    robot_serial_number: c.robot_serial_number,
                    time_issued: c.time_issued,
                    time_instruction: c.time_issued,
                    instruction: serde_json::from_str(&c.instruction)
                        .unwrap_or(Instruction::Abort(AbortReason::Saftey)),
                    completed: c.completed,
                })
            }
            pending
        })
        .map_err(|_| ApiError::DatabaseConnFailed)
    }

    pub async fn get_by_id(conn: &PgPool, command_id: i64) -> Result<Command, ApiError> {
        sqlx::query!(
            r#"
SELECT * FROM Commands C
WHERE C.command_id = $1
               "#,
            command_id
        )
        .fetch_one(conn)
        .await
        .map(|c| Self {
            command_id: c.command_id,
            robot_serial_number: c.robot_serial_number,
            time_issued: c.time_issued,
            time_instruction: c.time_issued,
            instruction: serde_json::from_str(&c.instruction)
                .unwrap_or(Instruction::Abort(AbortReason::Saftey)),
            completed: c.completed,
        })
        .map_err(|_| ApiError::DatabaseConnFailed)
    }
}
