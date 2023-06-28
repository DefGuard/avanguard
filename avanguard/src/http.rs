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

use crate::{
    db::{RefreshToken, Wallet},
    error::ApiError,
    state::AppState,
};

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
    pub refresh_token: String,
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
    token_expiration: i64,
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
    let expiration = issue_time + Duration::seconds(token_expiration);
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
                app_state.config.token_timeout,
            )?;
            wallet.challenge_signature = Some(signature.signature.clone());
            wallet.save(&app_state.pool).await?;
            let mut refresh_token =
                RefreshToken::new(wallet.id.unwrap(), app_state.config.refresh_token_timeout);
            refresh_token.save(&app_state.pool).await?;
            Ok(Json(JwtToken {
                token: id_token.to_string(),
                refresh_token: refresh_token.token,
            }))
        }
        _ => Err(ApiError::SignatureIncorrect),
    }
}

/// Issue new id token and refresh token set old as used
#[post("/refresh")]
pub async fn refresh(
    app_state: web::Data<AppState>,
    refresh_token: String,
) -> Result<Json<JwtToken>, ApiError> {
    if let Ok(Some(mut refresh_token)) =
        RefreshToken::find_refresh_token(&app_state.pool, &refresh_token).await
    {
        refresh_token.set_used(&app_state.pool).await?;
        let new_token = RefreshToken::new(
            refresh_token.wallet_id,
            app_state.config.refresh_token_timeout,
        );
        if let Some(wallet) = Wallet::find_by_id(&app_state.pool, refresh_token.wallet_id).await? {
            // Doesn't return nonce while refreshing token
            // https://openid.net/specs/openid-connect-core-1_0.html#RefreshTokenResponse
            let id_token = issue_id_token(
                &wallet.address,
                &app_state.config.issuer_url,
                app_state.config.client_secret.clone(),
                None,
                "",
                &app_state.config.client_id,
                app_state.config.token_timeout,
            )?;
            return Ok(Json(JwtToken {
                token: id_token.to_string(),
                refresh_token: refresh_token.token,
            }));
        } else {
            Err(ApiError::WalletNotFound)
        }
    } else {
        Err(ApiError::TokenNotFound)
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
