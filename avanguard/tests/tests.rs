use actix_web::{
    dev::{ServiceFactory, ServiceRequest},
    middleware, test, web, App,
};
use avanguard::{
    config_service, crypto::keccak256, db::Wallet, state::AppState, Challenge, WalletAddress,
    WalletSignature, CHALLENGE_TEMPLATE,
};
use ethers::types::transaction::eip712::{Eip712, TypedData};
use secp256k1::{rand::rngs::OsRng, Message, Secp256k1};

use avanguard::{
    db::{init_db, DbPool},
    Config,
};
use clap::Parser;
use sqlx::{postgres::PgConnectOptions, query, types::Uuid};

/// Initializes & migrates database with random name for tests.
async fn init_test_db() -> (DbPool, Config) {
    let config = Config::parse();
    let opts = PgConnectOptions::new()
        .host(&config.db_host)
        .port(config.db_port)
        .username(&config.db_user)
        .password(&config.db_password)
        .database(&config.db_name);
    let pool = DbPool::connect_with(opts)
        .await
        .expect("Failed to connect to Postgres");
    let db_name = Uuid::new_v4().to_string();
    query(&format!("CREATE DATABASE \"{}\"", db_name))
        .execute(&pool)
        .await
        .expect("Failed to create test database");
    let pool = init_db(
        &config.db_host,
        config.db_port,
        &db_name,
        &config.db_user,
        &config.db_password,
    )
    .await;
    (pool, config)
}

/// Initializes actix App and creates a wallet for testing
async fn init_app(wallet_address: &str) -> (App<impl ServiceFactory<ServiceRequest>>, Wallet) {
    let (pool, _) = init_test_db().await;

    let app = App::new()
        .app_data(web::Data::new(AppState::new(pool.clone())))
        .wrap(middleware::Logger::default())
        .configure(config_service);

    let mut wallet = Wallet::new(wallet_address.to_owned(), 1);
    wallet.save(&pool).await.unwrap();
    (app, wallet)
}

#[actix_web::test]
async fn test_challenge_signing() {
    let secp = Secp256k1::new();
    let (secret_key, public_key) = secp.generate_keypair(&mut OsRng);

    pub fn to_lower_hex(bytes: &[u8]) -> String {
        let mut hex = String::with_capacity(bytes.len() + 2);
        let to_char = |nibble: u8| -> char {
            (match nibble {
                0..=9 => b'0' + nibble,
                _ => nibble + b'a' - 10,
            }) as char
        };
        bytes.iter().for_each(|byte| {
            hex.push(to_char(*byte >> 4));
            hex.push(to_char(*byte & 0xf));
        });
        hex
    }
    // create eth wallet address
    let public_key = public_key.serialize_uncompressed();
    let hash = keccak256(&public_key[1..]);
    let addr = &hash[hash.len() - 20..];
    let wallet_address = to_lower_hex(addr);

    // TODO
    // let (app, _) = init_app(&wallet_address).await;

    // TODO: remove after implementing init_app method
    let (pool, _) = init_test_db().await;

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(AppState::new(pool.clone())))
            .wrap(middleware::Logger::default())
            .configure(config_service),
    )
    .await;

    let mut wallet = Wallet::new(wallet_address.to_owned(), 1);
    wallet.save(&pool).await.unwrap();

    let request = test::TestRequest::post()
        .uri("/auth/start")
        .set_json(WalletAddress {
            address: wallet_address.clone(),
        })
        .to_request();
    let challenge: Challenge = test::call_and_read_body_json(&app, request).await;

    let nonce = to_lower_hex(&keccak256(wallet_address.as_bytes()));
    let message: String = format!(
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
        wallet_address, CHALLENGE_TEMPLATE, nonce,
    )
    .chars()
    .filter(|c| c != &'\r' && c != &'\n' && c != &'\t')
    .collect::<String>();
    assert_eq!(challenge.challenge, message);

    // Sign message
    let typed_data: TypedData = serde_json::from_str(&message).unwrap();
    let hash_msg = typed_data.encode_eip712().unwrap();
    let message = Message::from_slice(&hash_msg).unwrap();
    let sig_r = secp.sign_ecdsa_recoverable(&message, &secret_key);
    let (rec_id, sig) = sig_r.serialize_compact();

    // Create recoverable_signature array
    let mut sig_arr = [0; 65];
    sig_arr[0..64].copy_from_slice(&sig[0..64]);
    sig_arr[64] = rec_id.to_i32() as u8;

    let request = test::TestRequest::post()
        .uri("/auth")
        .set_json(WalletSignature {
            address: wallet_address.clone(),
            signature: to_lower_hex(&sig_arr),
            nonce: String::from("test"),
        })
        .to_request();
    let response = test::call_service(&app, request).await;
    assert!(response.status().is_success());

    // TODO: validate OIDC token
}
