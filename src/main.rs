use dotenvy::dotenv;

mod config;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    tracing_subscriber::fmt().init();

    let connection = config::db::connect_db().await;

    config::server::start_server(connection).await
}
