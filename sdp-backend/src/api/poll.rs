use crate::poll::Init;
use crate::poll::Poll;

use actix_web::{get, web, web::Data, HttpResponse};
use sqlx::postgres::PgPool;

#[get("/poll")]
pub async fn robot_poll(conn: Data<PgPool>, poll: web::Json<Poll>) -> HttpResponse {
    Poll::poll(&conn, &poll)
        .await
        .map_or_else(|e| e.into(), |cmd| HttpResponse::Ok().json(cmd))
}

#[get("/init")]
pub async fn robot_init(conn: Data<PgPool>, init: web::Json<Init>) -> HttpResponse {
    Init::init(&conn, &init)
        .await
        .map_or_else(|e| e.into(), |cmd| HttpResponse::Ok().json(cmd))
}
