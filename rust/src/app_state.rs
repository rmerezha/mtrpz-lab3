use deadpool_postgres::Pool;

use crate::config::Config;

#[derive(Clone)]
pub struct AppState {
    pub cfg: Config,
    pub connection_pool: Pool,
}

impl AppState {
    pub fn db_pool(&self) -> &Pool {
        &self.connection_pool
    }
}
