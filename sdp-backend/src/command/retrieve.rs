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
            time_instruction: c.time_instruction,
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
                time_instruction: c.time_instruction,
                instruction: c.instruction.into(),
                status: c.status.into(),
            })
        }

        Ok(commands)
    }

    // Cancel all of the commands that do not call in the
    async fn prune(conn: &PgPool, commands: Vec<Self>) -> Result<Vec<Self>, ApiError> {
        let mut pruned_commands = Vec::new();
        let time_now = chrono::Utc::now();
        for c in commands {
            // Check for errors in the time instruction
            let (_, time_instruction_error) = c.valid_time();

            match time_instruction_error {
                // If the command is scheduled for the future that is fine
                _ if c.time_instruction > time_now => pruned_commands.push(c),

                // If the command is inside the time buffer, its valid
                None => pruned_commands.push(c),

                // There was an error with the command, cancel it
                Some(_) => {
                    c.cancel(conn).await?;
                    ()
                }
            }
        }

        pruned_commands.sort_by(|a, b| a.time_instruction.cmp(&b.time_instruction));

        Ok(pruned_commands)
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

        // Cancel all of the invalid commands and get the earliest one
        let commands = Self::prune(conn, all_pending_commands).await?;

        match commands.get(0) {
            Some(c) if c.instruction_in_buffer() => {
                println!("{:?}", c);
                Ok(Some(c.clone()))
            }
            _ => Ok(None),
        }
    }

    // Get all of the commands that are currently ongoing, that is have a status that is paused or
    // in progress
    pub async fn ongoing(conn: &PgPool, robot_serial_number: &str) -> Result<Vec<Self>, ApiError> {
        let results = sqlx::query!(
            r#"
SELECT * FROM Commands C
WHERE C.robot_serial_number = $1 AND
     (C.status = 'Status::Paused' OR C.status = 'Status::InProgress')
               "#,
            robot_serial_number
        )
        .fetch_all(conn)
        .await
        .map_err(|_| ApiError::DatabaseConnFailed)?;

        let mut all_ongoing_commands = Vec::new();
        for c in results {
            all_ongoing_commands.push(Self {
                command_id: c.command_id,
                robot_serial_number: c.robot_serial_number,
                time_issued: c.time_issued,
                time_instruction: c.time_instruction,
                instruction: c.instruction.into(),
                status: c.status.into(),
            })
        }

        Ok(all_ongoing_commands)
    }

    pub async fn cancel_all_ongoing(
        conn: &PgPool,
        robot_serial_number: &str,
    ) -> Result<(), ApiError> {
        // Get all of the commands that are ongoing and cancel them
        for c in Self::ongoing(conn, robot_serial_number).await? {
            c.cancel(conn).await?;
        }

        Ok(())
    }

    pub async fn init_command(conn: &PgPool, robot_serial_number: &str) -> Result<Self, ApiError> {
        // Cancel all of the ongoing commands
        Self::cancel_all_ongoing(conn, robot_serial_number).await?;

        // If there is a pending command use that, otherwise make a new idle command
        match Self::pending(conn, robot_serial_number).await? {
            Some(c) => Ok(c),
            None => Ok(Self::new_idle(conn, robot_serial_number)
                .await?
                .in_progress(conn)
                .await?),
        }
    }

    pub async fn get_all_by_robot_serial_number(
        conn: &PgPool,
        robot_serial_number: &str,
    ) -> Result<Vec<Command>, ApiError> {
        let results = sqlx::query!(
            r#"
SELECT * FROM Commands C
WHERE C.robot_serial_number = $1
               "#,
            robot_serial_number.clone(),
        )
        .fetch_all(conn)
        .await
        .map_err(|_| ApiError::DatabaseConnFailed)?;

        println!("Results {:?}", results);

        let mut commands = Vec::new();
        for r in results {
            commands.push(Self {
                command_id: r.command_id,
                robot_serial_number: r.robot_serial_number,
                time_issued: r.time_issued,
                time_instruction: r.time_instruction,
                instruction: r.instruction.into(),
                status: r.status.into(),
            });
        }
        Ok(commands)
    }
}
