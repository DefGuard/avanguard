use actix_web::{middleware, web, App, HttpServer};
use anyhow::Result;
use backend::state::AppState;
use backend::{config_service, Config};
use clap::Parser;
use env_logger::Builder;
use std::net::{IpAddr, Ipv4Addr};

#[macro_use]
extern crate log;

#[actix_web::main]
async fn main() -> Result<()> {
    let config = Config::parse();
    Builder::new().filter_level(config.log_level).init();

    info!("Backend HTTP server starting...");
    let listen_port = config.listen_port;
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(AppState::new(config.clone())))
            .wrap(middleware::Logger::default())
            .configure(config_service)
    })
    .bind((IpAddr::V4(Ipv4Addr::UNSPECIFIED), listen_port))?
    .run()
    .await?;

    Ok(())
}
