use crate::error::ApiError;
use crate::robot::Robot;
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPool;

#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    user_id: i64,
    user_name: String,
    password_hash: String,
    robot_serial_number: String,
}

impl User {
    pub async fn new(
        conn: &PgPool,
        user_name: &str,
        password: &str,
        robot_serial_number: &str,
    ) -> Result<Self, ApiError> {
        // TODO: fix this
        let user_name = user_name.to_string();
        let robot_serial_number = robot_serial_number.to_string();

        let password_hash = hash(password, DEFAULT_COST).map_err(|_| ApiError::HashingFailed)?;

        Robot::new(conn, &robot_serial_number).await?;

        let user_id = sqlx::query!(
            r#"
INSERT INTO users (user_name, password_hash, robot_serial_number)
VALUES ( $1, $2, $3 )
RETURNING user_id
        "#,
            user_name,
            password_hash,
            robot_serial_number
        )
        .fetch_one(conn)
        .await
        .map_err(|e| {
            println!("User Insert: {:?}", e);
            ApiError::DatabaseConnFailed
        })?
        .user_id;

        Ok(Self {
            user_id,
            user_name,
            password_hash,
            robot_serial_number,
        })
    }
}
