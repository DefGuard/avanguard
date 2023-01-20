use actix_web::{
    get, post,
    web::{self, Json},
};
use sqlx::query_as;

use crate::{db::Wallet, error::ApiError, state::AppState, CHALLENGE_TEMPLATE};

#[derive(Serialize, Deserialize)]
pub struct Challenge {
    pub challenge: String,
}

#[derive(Serialize, Deserialize)]
pub struct WalletAddress {
    pub address: String,
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

/// Start Web3 authentication
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

// TODO:
// /// Finish Web3 authentication
// #[post("/auth", format = "json", data = "<signature>")]
// pub async fn web3auth_end(
//     mut session: Session,
//     appstate: &State<AppState>,
//     signature: Json<WalletSignature>,
//     cookies: &CookieJar<'_>,
// ) -> ApiResult {
//     if let Some(ref challenge) = session.web3_challenge {
//         if let Some(wallet) =
//             Wallet::find_by_user_and_address(&appstate.pool, session.user_id, &signature.address)
//                 .await?
//         {
//             if wallet.use_for_mfa {
//                 return match wallet.verify_address(challenge, &signature.signature) {
//                     Ok(true) => {
//                         session
//                             .set_state(&appstate.pool, SessionState::MultiFactorVerified)
//                             .await?;
//                         if let Some(user) =
//                             User::find_by_id(&appstate.pool, session.user_id).await?
//                         {
//                             let user_info = UserInfo::from_user(&appstate.pool, user).await?;
//                             if let Some(openid_cookie) = cookies.get("known_sign_in") {
//                                 Ok(ApiResponse {
//                                     json: json!(AuthResponse {
//                                         user: user_info,
//                                         url: Some(openid_cookie.value().to_string())
//                                     }),
//                                     status: Status::Ok,
//                                 })
//                             } else {
//                                 Ok(ApiResponse {
//                                     json: json!(AuthResponse {
//                                         user: user_info,
//                                         url: None,
//                                     }),
//                                     status: Status::Ok,
//                                 })
//                             }
//                         } else {
//                             Ok(ApiResponse::default())
//                         }
//                     }
//                     _ => Err(OriWebError::Authorization("Signature not verified".into())),
//                 };
//             }
//         }
//     }
//     Err(OriWebError::Http(Status::BadRequest))
// }

/// Configure Actix Web server.
pub fn config_service(config: &mut web::ServiceConfig) {
    config
        .service(health_check)
        .service(list_wallets)
        .service(web3auth_start);
}
