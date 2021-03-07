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


            }
        }
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

    }

    }
}
