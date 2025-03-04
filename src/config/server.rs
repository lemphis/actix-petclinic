use crate::{web, AppState};
use actix_files::Files;
use actix_web::{cookie::Key, middleware, web::Data, App, HttpServer};
use actix_web_flash_messages::{storage::CookieMessageStore, FlashMessagesFramework};

pub async fn start_server(app_state: AppState) -> std::io::Result<()> {
    let signing_key = Key::generate();
    let message_store = CookieMessageStore::builder(signing_key).build();
    let message_framework = FlashMessagesFramework::builder(message_store).build();

    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(app_state.clone()))
            .wrap(middleware::Logger::default())
            .wrap(middleware::NormalizePath::trim())
            .wrap(middleware::Compress::default())
            .wrap(message_framework.clone())
            .service(Files::new("/static", "./static").show_files_listing())
            .configure(web::configure_route)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
