use crate::error::ApiError;
use chrono::{serde::ts_seconds, Utc};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPool;

// TODO: Set this to a sensible value
const TIME_ISSUED_BUFFER: i64 = 1000;
const TIME_INSTRUCTION_BUFFER: i64 = 1000;

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
    pub async fn new(
        conn: &PgPool,
        robot_serial_number: &str,
        time_issued: chrono::DateTime<Utc>,
        time_instruction: chrono::DateTime<Utc>,
        instruction: &Instruction,
    ) -> Result<Command, ApiError> {
        // Check that the commands was given within the
        //   time buffer
        let time_difference = (chrono::Utc::now() - time_issued).num_seconds().abs();
        if time_difference > TIME_ISSUED_BUFFER {
            println!(
                "Error: Outside of the time buffer\nTime Diff: {}",
                time_difference
            );
            return Err(ApiError::CommandNotInTimeIssuedBuffer);
        }

        let instruction_json = serde_json::to_string(instruction).map_err(|e| {
            println!("Instrution Json: {:?}", e);
            ApiError::SerializationError
        })?;

        let command_id = sqlx::query!(
            r#"
        INSERT INTO Commands (robot_serial_number, time_issued, time_instruction, instruction)
        VALUES ( $1, $2, $3, $4 )
        RETURNING command_id
                "#,
            robot_serial_number,
            time_issued,
            time_instruction,
            instruction_json
        )
        .fetch_one(conn)
        .await
        .map_err(|e| {
            println!("Command New: {:?}", e);
            ApiError::DatabaseConnFailed
        })?
        .command_id;

        let robot_serial_number = robot_serial_number.to_string();

        Ok(Self {
            command_id,
            robot_serial_number,
            time_issued,
            time_instruction,
            instruction: instruction.clone(),
            completed: false,
        })
    }

    // Get the current task the robot is doing
    pub async fn current(conn: &PgPool, robot_serial_number: &str) -> Result<Self, ApiError> {
        sqlx::query!(
            r#"
SELECT * FROM Commands C
NATURAL JOIN
(SELECT MAX(C1.time_issued) AS time_issued,
        $1 AS robot_serial_number
FROM Commands C1
WHERE C1.robot_serial_number = $1) MaxTimeIssued
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
            instruction: Instruction::Idle,
            completed: cmd.completed,
        })
        .map_err(|e| {
            println!("Command Latest: {:?}", e);
            ApiError::DatabaseConnFailed
        })
    }

    // pub async fn update(
    //     conn: &PgPool,
    //     robot_serial_number: &str,
    //     time_issued: chrono::DateTime<Utc>,
    //     time_instruction: chrono::DateTime<Utc>,
    //     instruction: &Instruction,
    // ) -> Result<Self, ApiError> {
    //     todo!();
    // }
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

    // Idle task the current task with the given reason
    pub async fn idle(conn: &PgPool, robot_serial_number: &str) -> Result<Self, ApiError> {
        // Create a new command with the current time
        let time_now = chrono::Utc::now();

        Ok(Command::new(
            conn,
            robot_serial_number,
            time_now,
            time_now,
            &Instruction::Idle,
        )
        .await?)
    }

    pub async fn task(
        conn: &PgPool,
        robot_serial_number: &str,
        cleaning_pattern: &CleaningPattern,
    ) -> Result<Self, ApiError> {
        // Create a new command with the current time
        let time_now = chrono::Utc::now();

        Ok(Command::new(
            conn,
            robot_serial_number,
            time_now,
            time_now,
            &Instruction::Task(cleaning_pattern.clone()),
        )
        .await?)
    }
}
