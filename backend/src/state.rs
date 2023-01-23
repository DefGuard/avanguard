use crate::Config;

pub struct AppState {
    pub config: Config,
}

impl AppState {
    #[must_use]
    pub fn new(config: Config) -> Self {
        Self { config }
    }
}
