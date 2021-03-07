use crate::command::Command;
use crate::error::ApiError;
use sqlx::postgres::PgPool;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Robot {
    pub robot_serial_number: String,
    pub assigned: bool,
}

impl Robot {
    pub async fn new(conn: &PgPool, robot_serial_number: &str) -> Result<Self, ApiError> {
        sqlx::query!(
            r#"
INSERT INTO robot (robot_serial_number)
VALUES ($1)
        "#,
            &robot_serial_number
        )
        .fetch_one(conn)
        .await
        .map(|_| Self {
            robot_serial_number: robot_serial_number.to_string(),
            assigned: false,
        })
        .map_err(|_| ApiError::RobotInitializationFailed)
    }


        // By default a new robot will be in idle.
        // Command::idle(conn, robot_serial_number).await?;

        Ok(())
    }
}
