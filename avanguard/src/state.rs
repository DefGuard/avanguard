use crate::{db::DbPool, Config};

pub struct AppState {
    pub config: Config,
    pub pool: DbPool,
}

impl AppState {
    #[must_use]
    pub fn new(config: Config, pool: DbPool) -> Self {
        Self { config, pool }
    }
}
