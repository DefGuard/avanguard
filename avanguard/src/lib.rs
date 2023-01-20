mod config;
pub use config::Config;
pub mod crypto;
pub mod db;
mod error;
mod http;
pub use http::{config_service, Challenge, WalletAddress};
use time::{OffsetDateTime, PrimitiveDateTime};
pub mod hex;
pub mod state;

#[macro_use]
extern crate serde;

pub fn now() -> PrimitiveDateTime {
    PrimitiveDateTime::new(
        OffsetDateTime::now_utc().date(),
        OffsetDateTime::now_utc().time(),
    )
}

pub static CHALLENGE_TEMPLATE: &str = "Please read this carefully:
 
Click to sign to prove you are in possesion of your private key to the account.
This request will not trigger a blockchain transaction or cost any gas fees.";
