use actix_web::{get, HttpResponse, Responder, web};

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

/// Configure Actix Web server.
pub fn config_service(config: &mut web::ServiceConfig) {
    config.service(health_check).service(hello);
}
