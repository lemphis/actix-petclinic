use std::sync::Arc;

use actix_web::{middleware, web, App, HttpServer};
use sea_orm::DatabaseConnection;

pub async fn start_server(db: DatabaseConnection) -> std::io::Result<()> {
    let db = Arc::new(db);

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(Arc::clone(&db)))
            .wrap(middleware::Logger::default())
            .wrap(middleware::NormalizePath::trim())
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
