use crate::error::ApiError;
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPool;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Robot {
    pub robot_serial_number: String,
    pub battery_level: i64,
    pub assigned: bool,
}

impl Robot {
    pub async fn set_battery(
        conn: &PgPool,
        robot_serial_number: &str,
        battery_level: i64,
    ) -> Result<Self, ApiError> {
        sqlx::query!(
            r#"
UPDATE Robot
SET battery_level = $2
WHERE robot_serial_number = $1
               "#,
            robot_serial_number.clone(),
            battery_level
        )
        .execute(conn)
        .await
        .map_err(|_| ApiError::DatabaseConnFailed)?;

        Self::get_by_serial(conn, robot_serial_number).await
    }

    pub async fn new(conn: &PgPool, robot_serial_number: &str) -> Result<Self, ApiError> {
        sqlx::query!(
            r#"
INSERT INTO Robot (robot_serial_number)
VALUES ($1)
RETURNING robot_serial_number
        "#,
            &robot_serial_number,
        )
        .fetch_one(conn)
        .await
        .map(|_| Self {
            robot_serial_number: robot_serial_number.to_string(),
            battery_level: 0,
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
            battery_level: r.battery_level,
            assigned: r.assigned,
        })
        .map_err(|_| ApiError::DatabaseConnFailed)
    }

    // Assigning the robot to a user, making assigned true
    pub async fn assign(&self, conn: &PgPool) -> Result<Self, ApiError> {
        // Check if the robot already assigned
        // If it is return an error
        if self.assigned {
            return Err(ApiError::RobotAlreadyAssigned);
        }

        sqlx::query!(
            r#"
UPDATE Robot
SET assigned = TRUE
WHERE robot_serial_number = $1
               "#,
            self.robot_serial_number
        )
        .execute(conn)
        .await
        .map_err(|_| ApiError::DatabaseConnFailed)?;

        Self::get_by_serial(conn, &self.robot_serial_number).await
    }
}
