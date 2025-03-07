use crate::{model::error_response::ErrorResponse, web, AppState};
use actix_files::Files;
use actix_web::{
    cookie::Key,
    dev::ServiceResponse,
    middleware::{self, ErrorHandlerResponse},
    web::Data,
    App, Error, HttpResponse, HttpServer, Result,
};
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

fn error_handler<B>(res: ServiceResponse<B>) -> Result<ErrorHandlerResponse<B>> {
    let error_message = res
        .response()
        .error()
        .map(Error::to_string)
        .unwrap_or_else(|| format!("HTTP {}", res.status()));
    let path = res.request().uri().path().to_owned();

    let error_response = ErrorResponse::new(error_message, path);

    let new_response = HttpResponse::build(res.status())
        .content_type("application/json")
        .json(error_response)
        .map_into_right_body();

    let new_res = ErrorHandlerResponse::Response(res.into_response(new_response));

    Ok(new_res)
}
