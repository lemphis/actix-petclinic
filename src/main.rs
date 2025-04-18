use config::i18n::I18n;
use sea_orm::DbConn;
use tera::Tera;

mod config;
mod domain;
mod model;
mod service;
mod web;

#[derive(Clone)]
struct AppState {
    conn: DbConn,
    tera: Tera,
    i18n: I18n,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    config::env::load();
    config::log::init();

    let i18n = config::i18n::I18n::new("locales");
    let tera = config::tera::init();
    let conn = config::db::connect_db().await;

    let app_state = AppState { conn, tera, i18n };

    config::server::start_server(app_state).await
}
