use crate::{
    state::AppState,
    token::{validate_token_response, Claims},
};

use actix_web::{
    get, post,
    web::{self, Json},
    HttpResponse, Responder,
};

#[derive(Serialize, Deserialize)]
pub struct TokenRequest {
    pub token: String,
    pub nonce: String,
}

/// Simple HTTP server health check.
#[get("/api/health")]
async fn health_check() -> &'static str {
    "alive"
}

/// Simple GET endpoint
#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

/// Simple GET endpoint
#[post("api/v1/token")]
async fn validate_token(
    app_state: web::Data<AppState>,
    data: Json<TokenRequest>,
) -> Result<Json<Claims>, actix_web::Error> {
    let token_claims = validate_token_response(&app_state.config, &data.token, &data.nonce)
        .await
        .unwrap();
    Ok(Json(token_claims))
}

/// Configure Actix Web server.
pub fn config_service(config: &mut web::ServiceConfig) {
    config
        .service(health_check)
        .service(hello)
        .service(validate_token);
}
