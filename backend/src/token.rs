use crate::config::Config;
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub iss: String,
    pub sub: String,
    pub aud: Vec<String>,
    pub exp: i64,
    pub nonce: String,
}

pub async fn validate_token_response(
    config: &Config,
    token: &str,
    nonce: &str,
) -> Result<Claims, ()> {
    let token_message = decode::<Claims>(
        token,
        &DecodingKey::from_secret(config.client_secret.as_ref()),
        &Validation::new(Algorithm::HS256),
    )
    .map_err(|_| ())?;

    if token_message.claims.nonce != nonce {
        return Err(());
    };
    Ok(token_message.claims)
}
