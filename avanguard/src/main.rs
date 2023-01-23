use actix_web::{middleware, web, App, HttpServer};
use anyhow::Result;
use avanguard::{config_service, db::init_db, state::AppState, Config};
use clap::Parser;
use env_logger::Builder;
use std::net::{IpAddr, Ipv4Addr};

#[macro_use]
extern crate log;

#[actix_web::main]
async fn main() -> Result<()> {
    let config = Config::parse();
    Builder::new().filter_level(config.log_level).init();
    info!("AvanGuard HTTP server starting...");

    // Initialize DB connection
    let pool = init_db(
        &config.db_host,
        config.db_port,
        &config.db_name,
        &config.db_user,
        &config.db_password,
    )
    .await;
    let listen_port = config.listen_port;
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(AppState::new(config.clone(), pool.clone())))
            .wrap(middleware::Logger::default())
            .configure(config_service)
    })
    .bind((IpAddr::V4(Ipv4Addr::UNSPECIFIED), listen_port))?
    .run()
    .await?;

    Ok(())
}
