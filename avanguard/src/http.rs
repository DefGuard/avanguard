use actix_web::{
    get, post,
    web::{self, Json},
};
use sqlx::query_as;

use crate::{db::Wallet, error::ApiError, state::AppState};

#[derive(Serialize, Deserialize)]
pub struct Challenge {
    pub challenge: String,
}

#[derive(Serialize, Deserialize)]
pub struct WalletAddress {
    pub address: String,
}

#[derive(Serialize, Deserialize)]
pub struct WalletSignature {
    pub address: String,
    pub signature: String,
    pub nonce: String,
}

#[derive(Serialize, Deserialize)]
pub struct IdToken {
    pub token: String,
}

/// Simple HTTP server health check.
#[get("/api/health")]
async fn health_check() -> &'static str {
    "alive"
}

// List wallets
#[get("/api/wallet")]
async fn list_wallets(app_state: web::Data<AppState>) -> Result<Json<Vec<Wallet>>, ApiError> {
    let wallets = query_as!(
        Wallet,
        "SELECT id \"id?\", address, chain_id, challenge_message, challenge_signature, creation_timestamp, validation_timestamp FROM wallet"
    ).fetch_all(&app_state.pool).await?;
    Ok(Json(wallets))
}

/// Start Web3 authentication. Returns challenge message for specified wallet address.
#[post("/auth/start")]
pub async fn web3auth_start(
    app_state: web::Data<AppState>,
    data: Json<WalletAddress>,
) -> Result<Json<Challenge>, ApiError> {
    // Create wallet if it does not exist yet
    let address = data.into_inner();
    let mut wallet = match Wallet::find_by_address(&app_state.pool, &address.address).await? {
        Some(wallet) => wallet,
        // TODO get actual wallet chain_id
        None => {
            let mut wallet = Wallet::new(address.address.clone(), 1);
            wallet.save(&app_state.pool).await?;
            wallet
        }
    };
    wallet.save(&app_state.pool).await?;
    Ok(Json(Challenge {
        challenge: wallet.challenge_message,
    }))
}

/// Finish Web3 authentication. Verifies signature and returns OIDC id_token if correct.
#[post("/auth")]
pub async fn web3auth_end(
    app_state: web::Data<AppState>,
    signature: Json<WalletSignature>,
) -> Result<Json<IdToken>, ApiError> {
    let wallet = match Wallet::find_by_address(&app_state.pool, &signature.address).await? {
        Some(wallet) => wallet,
        None => return Err(ApiError::WalletNotFound),
    };
    match wallet.verify_address(&wallet.challenge_message, &signature.signature) {
        Ok(true) => Ok(Json(IdToken {
            token: String::from("TODO"),
        })),
        _ => Err(ApiError::SignatureIncorrect),
    }
}

/// Configure Actix Web server.
pub fn config_service(config: &mut web::ServiceConfig) {
    config
        .service(health_check)
        .service(list_wallets)
        .service(web3auth_start)
        .service(web3auth_end);
}
