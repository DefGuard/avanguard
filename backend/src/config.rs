use clap::Parser;
use log::LevelFilter;

#[derive(Clone, Parser)]
pub struct Config {
    #[clap(long, env = "AG_LISTEN_PORT", default_value_t = 8081)]
    pub listen_port: u16,

    #[clap(long, env = "AG_LOG_LEVEL", default_value_t = LevelFilter::Info)]
    pub log_level: LevelFilter,

    #[clap(long, env = "OPENID_CLIENT_ID", default_value = "client_id")]
    pub client_id: String,

    #[clap(long, env = "OPENID_CLIENT_SECRET", default_value = "client_secret")]
    pub client_secret: String,

    #[clap(
        long,
        env = "OPENID_PROVIDER_URL",
        default_value = "http://localhost:8080"
    )]
    pub client_url: String,
}
