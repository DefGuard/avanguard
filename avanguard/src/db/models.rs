use crate::{
    crypto::keccak256,
    db::DbPool,
    error::Web3Error,
    hex::{hex_decode, to_lower_hex},
    random::gen_alphanumeric,
    CHALLENGE_TEMPLATE,
};
use chrono::{Duration, NaiveDateTime, Utc};
use ethers::types::transaction::eip712::{Eip712, TypedData};
use model_derive::Model;
use secp256k1::{
    ecdsa::{RecoverableSignature, RecoveryId},
    Message, Secp256k1,
};
use sqlx::{query, query_as};

#[derive(Model, Serialize)]
pub struct Wallet {
    pub(crate) id: Option<i64>,
    pub address: String,
    pub challenge_message: String,
    pub challenge_signature: Option<String>,
    pub creation_timestamp: NaiveDateTime,
    pub validation_timestamp: Option<NaiveDateTime>,
}

impl Wallet {
    #[must_use]
    pub fn new(address: String) -> Self {
        let challenge_message = Self::format_challenge(&address, CHALLENGE_TEMPLATE);
        Self {
            id: None,
            address,
            challenge_message,
            challenge_signature: None,
            creation_timestamp: Utc::now().naive_utc(),
            validation_timestamp: None,
        }
    }

    pub fn verify_address(&self, message: &str, signature: &str) -> Result<bool, Web3Error> {
        let address_array = hex_decode(&self.address).map_err(|_| Web3Error::Decode)?;
        let signature_array = hex_decode(signature).map_err(|_| Web3Error::Decode)?;

        let typed_data: TypedData = serde_json::from_str(message).map_err(|_| Web3Error::Decode)?;
        let hash_msg = typed_data.encode_eip712().map_err(|_| Web3Error::Decode)?;
        let message = Message::from_slice(&hash_msg).map_err(|_| Web3Error::InvalidMessage)?;
        let id = match signature_array[64] {
            0 | 27 => 0,
            1 | 28 => 1,
            v if v >= 35 => i32::from((v - 1) & 1),
            _ => return Err(Web3Error::InvalidRecoveryId),
        };
        let recovery_id = RecoveryId::from_i32(id).map_err(|_| Web3Error::ParseSignature)?;
        let recoverable_signature =
            RecoverableSignature::from_compact(&signature_array[0..64], recovery_id)
                .map_err(|_| Web3Error::ParseSignature)?;
        let public_key = Secp256k1::new()
            .recover_ecdsa(&message, &recoverable_signature)
            .map_err(|_| Web3Error::Recovery)?;
        let public_key = public_key.serialize_uncompressed();
        let hash = keccak256(&public_key[1..]);

        Ok(hash[12..] == address_array)
    }

    pub fn validate_signature(&self, signature: &str) -> Result<(), Web3Error> {
        if self.verify_address(&self.challenge_message, signature)? {
            Ok(())
        } else {
            Err(Web3Error::VerifyAddress)
        }
    }

    pub async fn set_signature(
        &mut self,
        pool: &DbPool,
        signature: &str,
    ) -> Result<(), sqlx::Error> {
        self.challenge_signature = Some(signature.into());
        self.validation_timestamp = Some(Utc::now().naive_utc());
        if let Some(id) = self.id {
            query!(
                "UPDATE wallet SET challenge_signature = $1, validation_timestamp = $2 WHERE id = $3",
                self.challenge_signature, self.validation_timestamp, id
            )
            .execute(pool)
            .await?;
        }
        Ok(())
    }

    /// Prepare challenge message using EIP-712 format
    pub fn format_challenge(address: &str, challenge_message: &str) -> String {
        let nonce = to_lower_hex(&keccak256(address.as_bytes()));

        format!(
            r#"{{
"domain": {{ "name": "Defguard", "version": "1" }},
"types": {{
    "EIP712Domain": [
        {{ "name": "name", "type": "string" }},
        {{ "name": "version", "type": "string" }}
    ],
    "ProofOfOwnership": [
        {{ "name": "wallet", "type": "address" }},
        {{ "name": "content", "type": "string" }},
        {{ "name": "nonce", "type": "string" }}
    ]
}},
"primaryType": "ProofOfOwnership",
"message": {{
    "wallet": "{}",
    "content": "{}",
    "nonce": "{}"
}}}}
"#,
            address, challenge_message, nonce
        )
        .chars()
        .filter(|c| c != &'\r' && c != &'\n' && c != &'\t')
        .collect()
    }

    pub async fn find_by_address(
        pool: &DbPool,
        address: &str,
    ) -> Result<Option<Self>, sqlx::Error> {
        query_as!(
            Self,
            "SELECT id \"id?\", address, challenge_message, challenge_signature, \
            creation_timestamp, validation_timestamp FROM wallet \
            WHERE address = $1",
            address
        )
        .fetch_optional(pool)
        .await
    }
}

pub fn hash_message<S: AsRef<[u8]>>(message: S) -> [u8; 32] {
    let message = message.as_ref();
    let mut eth_message = format!("\x19Ethereum Signed Message:\n{}", message.len()).into_bytes();
    eth_message.extend_from_slice(message);
    keccak256(&eth_message)
}

#[derive(Model)]
pub struct RefreshToken {
    pub(crate) id: Option<i64>,
    pub wallet_id: i64,
    pub token: String,
    pub expires_at: i64,
    pub used_at: Option<i64>,
    pub blacklisted: bool,
}

impl RefreshToken {
    #[must_use]
    pub fn new(wallet_id: i64, expires_in: i64) -> Self {
        let expiration = Utc::now() + Duration::seconds(expires_in);
        Self {
            id: None,
            wallet_id,
            token: gen_alphanumeric(24),
            expires_at: expiration.timestamp(),
            used_at: None,
            blacklisted: false,
        }
    }
    #[must_use]
    pub fn is_expired(&self) -> bool {
        self.expires_at < Utc::now().timestamp()
    }

    /// Blacklist token
    pub async fn blacklist(&self, pool: &DbPool) -> Result<(), sqlx::Error> {
        query!(
            "UPDATE refreshtoken SET blacklisted = true \
            WHERE token = $1",
            self.token,
        )
        .execute(pool)
        .await?;
        Ok(())
    }
    /// Find by refresh token.
    pub async fn find_refresh_token(
        pool: &DbPool,
        token: &str,
    ) -> Result<Option<Self>, sqlx::Error> {
        match query_as!(
            Self,
            r#"SELECT id "id?", wallet_id, token, expires_at, blacklisted, used_at "used_at?"
            FROM refreshtoken WHERE token = $1 
            AND blacklisted = false 
            AND used_at IS NULL"#,
            token
        )
        .fetch_optional(pool)
        .await
        {
            Ok(Some(token)) => {
                if token.is_expired() {
                    token.delete(pool).await?;
                    Ok(None)
                } else {
                    Ok(Some(token))
                }
            }
            Ok(None) => Ok(None),
            Err(err) => Err(err),
        }
    }
    /// Mark token as used
    pub async fn set_used(&mut self, pool: &DbPool) -> Result<(), sqlx::Error> {
        let used_at = Utc::now().timestamp();
        query!(
            "UPDATE refreshtoken SET used_at = $2 \
            WHERE token = $1",
            self.token,
            Some(used_at),
        )
        .execute(pool)
        .await?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_verify_address() {
        for (address, signature) in [
            ("0x6cD15DA14A4Ef26047f1D7858D7A82b59DDCa102",
            "0xfb812c61b3d5f3ea729a049b4f14c28c07938367c91062c959150e1a3273f07772f162c5abf8312be39c3a6640c47e02866bcd19b5545bc5650d5870547a1a8f1c"),
            ("0x8AEF669452465635355923E4Dc80990aEAEE3b8d",
            "0xefa7641e06c0d35e9386a3d97d50c5a2fffc7c5838ea42647093417b62bd1dc830ce8ecea3f9173190ee6c215a8a423fd1110caba06b6dc474e0792f802dfdc31b"),
            ("0xE8e659AD9E99afd41f97015Cb2E2a96dD7456fA0",
            "0x47d3eddfb2ed3ad1776c704fbe90737286ede2931c9e561abe6ce33606f411a00eafc25ec540e5db7ea82364e7df1e4722a916a828f02746a28773ae0e7bf3f31b"),
        ] {
            let message =  Wallet::format_challenge(address, CHALLENGE_TEMPLATE);
            let wallet = Wallet::new(address.into());
            let result = wallet.verify_address(
                &message,
                signature,
            )
            .unwrap();
            assert!(result);
        }
    }
}
