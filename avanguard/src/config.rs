use clap::Parser;
use log::LevelFilter;
use openidconnect::url::Url;

#[derive(Clone, Parser)]
pub struct Config {
    #[clap(
        long,
        env = "AG_ISSUER_URL",
        value_parser = Url::parse,
        default_value = "http://localhost:8080",
        help = "URL to be used as issuer in JWT token"
    )]
    pub issuer_url: Url,

    #[clap(
        long,
        env = "AG_CLIENT_ID",
        default_value = "client_id",
        help = "OIDC client id, shared with client application"
    )]
    pub client_id: String,

    #[clap(
        long,
        env = "AG_CLIENT_SECRET",
        default_value = "client_secret",
        help = "OIDC client secret, shared with client application to perform HMAC JWT validation"
    )]
    pub client_secret: String,

    #[clap(
        long,
        env = "AG_CLIENT_ORIGIN_URL",
        default_value = "http://localhost:8000",
        help = "Url from which client requests will come, used to set CORS header"
    )]
    pub client_origin_url: String,

    #[clap(
        long,
        env = "AG_LISTEN_PORT",
        default_value_t = 8080,
        help = "REST API listen port"
    )]
    pub listen_port: u16,

    #[clap(
        long,
        env = "AG_DB_HOST",
        default_value = "localhost",
        help = "Database host"
    )]
    pub db_host: String,

    #[clap(
        long,
        env = "AG_DB_PORT",
        default_value_t = 5432,
        help = "Database port"
    )]
    pub db_port: u16,

    #[clap(
        long,
        env = "AG_DB_NAME",
        default_value = "avanguard",
        help = "Database name"
    )]
    pub db_name: String,

    #[clap(
        long,
        env = "AG_DB_USER",
        default_value = "avanguard",
        help = "Database user"
    )]
    pub db_user: String,

    #[clap(
        long,
        env = "AG_DB_PASSWORD",
        default_value = "",
        help = "Database password"
    )]
    pub db_password: String,

    #[clap(long, env = "AG_LOG_LEVEL", default_value_t = LevelFilter::Info, help = "Log level")]
    pub log_level: LevelFilter,

    #[arg(
        long,
        env = "TOKEN_TIMEOUT",
        default_value_t = 3600 * 4,
        help = "Token timeout"
    )]
    pub token_timeout: i64,

    #[clap(
        long,
        env = "REFRESH_TOKEN_TIMEOUT",
        default_value_t = 3600 * 24,
        help = "Refresh token timeout"
    )]
    pub refresh_token_timeout: i64,
}
