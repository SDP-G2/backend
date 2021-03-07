use chrono::{serde::ts_seconds, Utc};
use serde::{Deserialize, Serialize};

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

impl Status {
    pub fn cancelled(&self) -> bool {
        self == &Self::Cancelled
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

impl std::fmt::Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Instruction::{Abort, Idle, Task};

        match self {
            Abort(AbortReason::LowBattery) => {
                write!(f, "Abort(AbortReason::LowBattery)")
            }
            Abort(AbortReason::Saftey) => write!(f, "Abort(AbortReason::Saftey)"),
            Task(CleaningPattern::Circular) => {
                write!(f, "Task(CleaningPattern::Circular)")
            }
            Task(CleaningPattern::ZigZag) => {
                write!(f, "Task(CleaningPattern::ZigZag)")
            }
            Idle => write!(f, "Idle"),
        }
    }
}

impl From<String> for Instruction {
    fn from(instruction: String) -> Self {
        use Instruction::{Abort, Idle, Task};

        match &instruction[..] {
            "Abort(AbortReason::LowBattery)" => Abort(AbortReason::LowBattery),
            "Abort(AbortReason::Saftey)" => Abort(AbortReason::Saftey),
            "Task(CleaningPattern::Circular)" => Task(CleaningPattern::Circular),
            "Task(CleaningPattern::ZigZag)" => Task(CleaningPattern::ZigZag),
            "Idle" => Idle,
            _ => Abort(AbortReason::Saftey),
        }
    }
}

    }

    }
}
