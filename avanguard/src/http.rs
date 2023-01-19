use actix_web::{get, web};
use sqlx::query_as;

use crate::{state::AppState, db::Wallet, error::ApiError};

/// Simple HTTP server health check.
#[get("/api/health")]
async fn health_check() -> &'static str {
    "alive"
}

// List wallets
#[get("/api/wallet")]
async fn list_wallets(app_state: web::Data<AppState>) -> Result<web::Json<Vec<Wallet>>, ApiError> {
    let wallets = query_as!(
        Wallet,
        "SELECT id \"id?\", address, chain_id, challenge_message, challenge_signature, creation_timestamp, validation_timestamp FROM wallet"
    ).fetch_all(&app_state.pool).await?;
    Ok(web::Json(wallets))
}

/// Configure Actix Web server.
pub fn config_service(config: &mut web::ServiceConfig) {
    config.service(health_check).service(list_wallets);
}
