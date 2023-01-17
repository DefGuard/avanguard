use actix_web::{middleware, App, HttpServer};
use anyhow::Result;
use clap::Parser;
use env_logger::Builder;
use avanguard::{Config, config_service};
use std::net::{IpAddr, Ipv4Addr};

#[macro_use]
extern crate log;

#[tokio::main]
async fn main() -> Result<()> {
    let config = Config::parse();
    Builder::new().filter_level(config.log_level).init();

    info!("AvanGuard HTTP server starting...");
    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .configure(config_service)
    })
    .bind((IpAddr::V4(Ipv4Addr::UNSPECIFIED), config.listen_port))?
    .run()
    .await?;

    Ok(())
}
