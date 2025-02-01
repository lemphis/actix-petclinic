mod config;
mod domain;
mod model;
mod web;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    config::env::load();
    config::log::init();

    let connection = config::db::connect_db().await;

    config::server::start_server(connection).await
}
