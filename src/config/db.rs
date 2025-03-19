use sea_orm::{Database, DbConn};
use std::env;

pub async fn connect_db() -> DbConn {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env files");

    Database::connect(database_url)
        .await
        .expect("Failed to connect to database")
}
