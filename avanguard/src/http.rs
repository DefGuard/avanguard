use actix_web::{
    get, post,
    web::{self, Json},
};
use chrono::{Duration, Utc};
use openidconnect::{
    core::{
        CoreGenderClaim, CoreHmacKey, CoreIdToken, CoreIdTokenClaims, CoreJsonWebKeyType,
        CoreJweContentEncryptionAlgorithm, CoreJwsSigningAlgorithm, CoreRsaPrivateSigningKey,
    },
    url::Url,
    Audience, EmptyAdditionalClaims, IdToken, IssuerUrl, JsonWebTokenError, Nonce, StandardClaims,
    SubjectIdentifier,
};
use sqlx::query_as;

use crate::{db::Wallet, error::ApiError, state::AppState, SESSION_TIMEOUT};

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
pub struct JwtToken {
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
        "SELECT id \"id?\", address, challenge_message, challenge_signature, creation_timestamp, validation_timestamp FROM wallet"
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
    let address = data.into_inner().address.to_lowercase();
    let mut wallet = match Wallet::find_by_address(&app_state.pool, &address).await? {
        Some(wallet) => wallet,
        None => {
            let mut wallet = Wallet::new(address);
            wallet.save(&app_state.pool).await?;
            wallet
        }
    };
    wallet.save(&app_state.pool).await?;
    Ok(Json(Challenge {
        challenge: wallet.challenge_message,
    }))
}

/// Creates OIDC id token for given wallet
fn issue_id_token<T>(
    wallet_address: &str,
    base_url: &Url,
    secret: T,
    rsa_key: Option<CoreRsaPrivateSigningKey>,
    nonce: &str,
    client_id: &str,
) -> Result<
    IdToken<
        EmptyAdditionalClaims,
        CoreGenderClaim,
        CoreJweContentEncryptionAlgorithm,
        CoreJwsSigningAlgorithm,
        CoreJsonWebKeyType,
    >,
    JsonWebTokenError,
>
where
    T: Into<Vec<u8>>,
{
    let wallet_address = wallet_address.to_lowercase();
    let issue_time = Utc::now();
    let expiration = issue_time + Duration::seconds(SESSION_TIMEOUT as i64);
    let claims = StandardClaims::new(SubjectIdentifier::new(wallet_address));
    let id_token_claims = CoreIdTokenClaims::new(
        IssuerUrl::from_url(base_url.clone()),
        vec![Audience::new(client_id.to_string())],
        expiration,
        issue_time,
        claims,
        openidconnect::EmptyAdditionalClaims {},
    )
    .set_nonce(Some(Nonce::new(nonce.to_string())));
    match rsa_key {
        // RSA flow
        Some(key) => CoreIdToken::new(
            id_token_claims,
            &key,
            CoreJwsSigningAlgorithm::RsaSsaPkcs1V15Sha256,
            None,
            None,
        ),
        // HMAC flow
        None => CoreIdToken::new(
            id_token_claims,
            &CoreHmacKey::new(secret),
            CoreJwsSigningAlgorithm::HmacSha256,
            None,
            None,
        ),
    }
}

/// Finish Web3 authentication. Verifies signature and returns OIDC id_token if correct.
#[post("/auth")]
pub async fn web3auth_end(
    app_state: web::Data<AppState>,
    signature: Json<WalletSignature>,
) -> Result<Json<JwtToken>, ApiError> {
    let address = signature.address.to_lowercase();
    let mut wallet = match Wallet::find_by_address(&app_state.pool, &address).await? {
        Some(wallet) => wallet,
        None => return Err(ApiError::WalletNotFound),
    };
    match wallet.verify_address(&wallet.challenge_message, &signature.signature) {
        Ok(true) => {
            let id_token = issue_id_token(
                &address,
                &app_state.config.issuer_url,
                app_state.config.client_secret.clone(),
                None,
                &signature.nonce,
                &app_state.config.client_id,
            )?;
            wallet.challenge_signature = Some(signature.signature.clone());
            wallet.save(&app_state.pool).await?;
            Ok(Json(JwtToken {
                token: id_token.to_string(),
            }))
        }
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
