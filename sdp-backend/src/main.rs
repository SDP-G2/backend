use rocket_contrib::serve::StaticFiles;
use actix_web::{middleware::Logger, App, HttpServer};
use std::env;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=debug");
    env_logger::init();

    // Default port for the webserver is 8080, this can be overwritten
    // by an envirnment variable.
    // TODO: Test this works
    let port = env::var("PORT").unwrap_or("8080".to_string());
    let address = format!("0.0.0.0:{}", port);
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .service(actix_files::Files::new("/static", "static/").show_files_listing())
    })
    .bind(address)?
    .run()
    .await
}
