use actix_web::{get, post, web, web::Data, HttpResponse};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPool;

use crate::robot::Robot;
use crate::user::User;

#[derive(Serialize, Deserialize, Debug)]
pub struct UserRequest {
    pub user_name: String,
    pub password: String,
    pub robot_serial_number: String,
}

#[post("/user")]
pub async fn create_user(conn: Data<PgPool>, user: web::Json<UserRequest>) -> HttpResponse {
    User::new(
        &conn,
        &user.user_name,
        &user.password,
        &user.robot_serial_number,
    )
    .await
    .map_or_else(|e| e.into(), |user| HttpResponse::Ok().json(user))
}

#[get("/user")]
pub async fn user_status(conn: Data<PgPool>, user: User) -> HttpResponse {
    Robot::get_by_serial(&conn, &user.robot_serial_number)
        .await
        .map_or_else(|e| e.into(), |r| HttpResponse::Ok().json(r))
}
