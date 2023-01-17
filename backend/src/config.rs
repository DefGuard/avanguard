use clap::Parser;
use log::LevelFilter;

#[derive(Clone, Parser)]
pub struct Config {
    #[clap(long, env = "VG_LISTEN_PORT", default_value_t = 8081)]
    pub listen_port: u16,

    #[clap(long, env = "VG_LOG_LEVEL", default_value_t = LevelFilter::Info)]
    pub log_level: LevelFilter,
}
