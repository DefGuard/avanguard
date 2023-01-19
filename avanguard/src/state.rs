use crate::db::DbPool;

pub struct AppState {
    pub pool: DbPool,
}

impl AppState {
    #[must_use]
    pub fn new(pool: DbPool) -> Self {
        Self {pool}
    }
}
