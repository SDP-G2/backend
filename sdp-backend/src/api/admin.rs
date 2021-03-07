use crate::robot::Robot;

use actix_web::{post, web, web::Data, HttpResponse};
use sqlx::postgres::PgPool;

#[post("/admin/robot")]
pub async fn create_robot(conn: Data<PgPool>, user: web::Json<Robot>) -> HttpResponse {
    Robot::new(&conn, &user.robot_serial_number)
        .await
        .map_or_else(|e| e.into(), |user| HttpResponse::Ok().json(user))
}
