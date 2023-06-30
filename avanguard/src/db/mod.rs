pub mod models;

use sqlx::postgres::PgConnectOptions;

pub type DbPool = sqlx::postgres::PgPool;

/// Initializes and migrates postgres database. Returns DB pool object.
pub async fn init_db(host: &str, port: u16, name: &str, user: &str, password: &str) -> DbPool {
    log::debug!("Connecting to database {}:{}/{}", host, port, name);
    let opts = PgConnectOptions::new()
        .host(host)
        .port(port)
        .username(user)
        .password(password)
        .database(name);
    let pool = DbPool::connect_with(opts)
        .await
        .expect("Database connection failed");
    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Cannot run database migrations.");
    log::info!("Connected to database {}:{}/{}", host, port, name);
    pool
}

pub use models::{RefreshToken, Wallet};
