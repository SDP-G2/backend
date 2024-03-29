use actix_web::{middleware::Logger, App, HttpServer};
use std::env;

mod api;
mod auth;
mod command;
mod error;
mod poll;
mod robot;
mod test;
mod user;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=debug");
    env_logger::init();

    // Default port for the webserver is 8080, this can be overwritten
    // by an envirnment variable.
    // TODO: Test this works
    let port = env::var("PORT").unwrap_or("8080".to_string());
    let address = format!("0.0.0.0:{}", port);

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL to be set");
    let database_pool = sqlx::postgres::PgPool::connect(&database_url)
        .await
        .expect("to get database pool");

    HttpServer::new(move || {
        App::new()
            .wrap(actix_cors::Cors::default().allow_any_origin())
            .wrap(Logger::default())
            .data(database_pool.clone())
            // User Endpoints
            .service(api::user::create_user)
            .service(api::user::user_status)
            // Command Endpoints
            .service(api::command::create_command)
            .service(api::command::get_command)
            .service(api::command::cancel_command)
            .service(api::auth::auth)
            // Robot Endpoints
            .service(api::poll::robot_poll)
            .service(api::poll::robot_init)
            // Admin Endpoints
            .service(api::admin::create_robot)
            // Static Files Endpoint
            .service(actix_files::Files::new("/static", "/static").show_files_listing())
    })
    .bind(address)?
    .run()
    .await
}
