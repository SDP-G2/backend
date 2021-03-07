use crate::error::ApiError;
use chrono::{serde::ts_seconds, Utc};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPool;

mod abort;
mod create;
mod retrieve;
mod time;
mod update;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Command {
    pub command_id: i64,
    pub robot_serial_number: String,
    #[serde(with = "ts_seconds")]
    time_issued: chrono::DateTime<Utc>,
    #[serde(with = "ts_seconds")]
    time_instruction: chrono::DateTime<Utc>,
    pub instruction: Instruction,
    pub status: Status,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Status {
    // A command that has been submitted to the system but not started
    Pending,
    // The command the robot is currently doing
    InProgress,
    // The command completed successfully
    Completed,
    // The command has been paused due to an obstacle
    Paused,
    // The command has beenc ancelled due to an abort
    Cancelled,
}
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
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Instruction {
    Abort(AbortReason),
    Task(CleaningPattern),
    Idle,
}


            }
        }
    }
}


    }
}

    }

    }
}
