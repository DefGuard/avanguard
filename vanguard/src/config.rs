use clap::Parser;
use log::LevelFilter;

#[derive(Clone, Parser)]
pub struct Config {
    #[clap(long, env = "VG_LOG_LEVEL", default_value_t = LevelFilter::Info)]
    pub log_level: LevelFilter,
}
