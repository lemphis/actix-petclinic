use crate::{web, AppState};
use actix_files::Files;
use actix_web::{middleware, web::Data, App, HttpServer};

pub async fn start_server(app_state: AppState) -> std::io::Result<()> {
    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(app_state.clone()))
            .wrap(middleware::Logger::default())
            .wrap(middleware::NormalizePath::trim())
            .wrap(middleware::Compress::default())
            .service(Files::new("/static", "./static").show_files_listing())
            .configure(web::configure_route)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
