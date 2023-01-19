use time::PrimitiveDateTime;

#[derive(Serialize)]
pub struct Wallet {
    pub(crate) id: Option<i64>,
    pub address: String,
    pub chain_id: i64,
    pub challenge_message: String,
    pub challenge_signature: Option<String>,
    pub creation_timestamp: PrimitiveDateTime,
    pub validation_timestamp: Option<PrimitiveDateTime>,
}
