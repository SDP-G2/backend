use crate::command::Command;
use crate::error::ApiError;
use chrono::{Duration, Utc};

// TODO: Set this to a sensible value
const TIME_BUFFER: i64 = 5;

impl Command {
    pub fn valid_time(&self) -> (Option<ApiError>, Option<ApiError>) {
        Self::check_time(self.time_issued, self.time_instruction)
    }

    pub fn check_time(
        time_issued: chrono::DateTime<Utc>,
        time_instruction: chrono::DateTime<Utc>,
    ) -> (Option<ApiError>, Option<ApiError>) {
        let time_now = chrono::Utc::now();
        let time_period = Duration::seconds(TIME_BUFFER);

        // Check that the time the command was made was inside the time buffer
        // The command could have been issued a little before or after
        let valid_time_issued = (time_now - time_period)..(time_now + time_period);
        let time_issued_error = if !valid_time_issued.contains(&time_issued) {
            Some(ApiError::InvalidTimeIssued)
        } else {
            None
        };

        // Check that the command is to be executed in the future or within the last time buffer
        let valid_time_instruction = (time_now - time_period)..time_now;
        let time_instruction_error =
            if !valid_time_instruction.contains(&time_instruction) || time_instruction > time_now {
                Some(ApiError::InvalidTimeInstruction)
            } else {
                None
            };

        (time_issued_error, time_instruction_error)
    }

    pub fn instruction_in_buffer(&self) -> bool {
        let time_now = chrono::Utc::now();
        let time_period = Duration::seconds(TIME_BUFFER);

        let valid_time_instruction = (time_now - time_period)..time_now;
        valid_time_instruction.contains(&self.time_instruction)
    }
}
