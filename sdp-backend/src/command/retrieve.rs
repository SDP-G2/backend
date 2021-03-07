use crate::command::Command;
use crate::error::ApiError;
use sqlx::postgres::PgPool;

impl Command {
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
            instruction: c.instruction.into(),
            status: c.status.into(),
        })
        .map_err(|_| ApiError::DatabaseConnFailed)
    }

    // Get all of the pending commands for this robot
    pub async fn get_all_pending(
        conn: &PgPool,
        robot_serial_number: &str,
    ) -> Result<Vec<Command>, ApiError> {
        let pending_commands = sqlx::query!(
            r#"
SELECT * FROM Commands C
WHERE C.robot_serial_number = $1 AND
      C.status = 'Status::Pending'
               "#,
            robot_serial_number
        )
        .fetch_all(conn)
        .await
        .map_err(|_| ApiError::DatabaseConnFailed)?;

        let mut commands = Vec::new();
        for c in pending_commands {
            commands.push(Self {
                command_id: c.command_id,
                robot_serial_number: c.robot_serial_number,
                time_issued: c.time_issued,
                time_instruction: c.time_issued,
                instruction: c.instruction.into(),
                status: c.status.into(),
            })
        }

        Ok(commands)
    }

    // Returns the command that should be executed next, or None if isn't anything
    // Prune the commands, if the time has expired and they have not been completed mark them as cancelled
    // Check that the command is in the time buffer before sending it.
    pub async fn pending(
        conn: &PgPool,
        robot_serial_number: &str,
    ) -> Result<Option<Self>, ApiError> {
        // Get all of the pending commands for this robot
        let all_pending_commands = Self::get_all_pending(conn, robot_serial_number).await?;

        // If the commands time_instruiction time is outside of the time buffer
        // mark it as cancelled.
        let mut commands = Vec::new();
        for c in all_pending_commands {
            // Check for errors in the time instruction
            let (_, time_instruction_error) = Self::check_time(c.time_issued, c.time_instruction);

            // If there is an error with the instruction time cancel it
            // otherwise add it to the list
            match time_instruction_error {
                None => commands.push(c),
                Some(_) => {
                    c.cancel(conn).await?;
                    ()
                }
            }
        }

        // Get the command that should be run text
        // The command with the first time_instruiction
        commands.sort_by(|a, b| a.time_instruction.cmp(&b.time_instruction));

        match commands.get(0) {
            Some(c) => Ok(Some(c.clone())),
            None => Ok(None),
        }
    }
}
