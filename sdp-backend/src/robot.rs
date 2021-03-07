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

    pub async fn get_by_serial(conn: &PgPool, robot_serial_number: &str) -> Result<Self, ApiError> {
        sqlx::query!(
            r#"
SELECT * FROM Robot R
WHERE R.robot_serial_number = $1
               "#,
            robot_serial_number
        )
        .fetch_one(conn)
        .await
        .map(|r| Self {
            robot_serial_number: r.robot_serial_number,
            assigned: r.assigned,
        })
        .map_err(|_| ApiError::DatabaseConnFailed)
    }

        // By default a new robot will be in idle.
        // Command::idle(conn, robot_serial_number).await?;

        Ok(())
    }
}
