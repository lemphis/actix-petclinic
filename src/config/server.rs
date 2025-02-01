use actix_web::{middleware, web::Data, App, HttpServer};
use sea_orm::DatabaseConnection;
use std::sync::Arc;

use crate::web;

pub async fn start_server(db: DatabaseConnection) -> std::io::Result<()> {
    let db = Arc::new(db);

    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(Arc::clone(&db)))
            .wrap(middleware::Logger::default())
            .wrap(middleware::NormalizePath::trim())
            .configure(web::configure_route)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
