use clap::Parser;
use log::LevelFilter;
use openidconnect::url::Url;

#[derive(Clone, Parser)]
pub struct Config {
    #[clap(long, env = "AG_ISSUER_URL", value_parser = Url::parse, default_value = "http://localhost:8080")]
    pub issuer_url: Url,

    #[clap(long, env = "AG_CLIENT_ID", default_value = "client_id")]
    pub client_id: String,

    #[clap(long, env = "AG_CLIENT_SECRET", default_value = "client_secret")]
    pub client_secret: String,

    #[clap(
        long,
        env = "AG_CLIENT_ORIGIN_URL",
        default_value = "http://localhost:8000"
    )]
    pub client_origin_url: String,

    #[clap(long, env = "AG_LISTEN_PORT", default_value_t = 8080)]
    pub listen_port: u16,

    #[clap(long, env = "AG_DB_HOST", default_value = "localhost")]
    pub db_host: String,

    #[clap(long, env = "AG_DB_PORT", default_value_t = 5432)]
    pub db_port: u16,

    #[clap(long, env = "AG_DB_NAME", default_value = "avanguard")]
    pub db_name: String,

    #[clap(long, env = "AG_DB_USER", default_value = "avanguard")]
    pub db_user: String,

    #[clap(long, env = "AG_DB_PASSWORD", default_value = "")]
    pub db_password: String,

    #[clap(long, env = "AG_LOG_LEVEL", default_value_t = LevelFilter::Info)]
    pub log_level: LevelFilter,
}
