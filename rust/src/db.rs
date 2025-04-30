use crate::config::Config as AppConfig;
use deadpool_postgres::{Manager, ManagerConfig, Pool, RecyclingMethod};
use tokio_postgres::{Config, NoTls};

pub async fn init_db_pool(cfg: &AppConfig) -> Pool {
    let pg_config = to_db_config(cfg);
    let mgr_config = ManagerConfig {
        recycling_method: RecyclingMethod::Fast,
    };
    let mgr = Manager::from_config(pg_config, NoTls, mgr_config);

    Pool::builder(mgr).max_size(cfg.db_pool_max_size).build().unwrap()
}

fn to_db_config(cfg: &AppConfig) -> Config {
    Config::new()
        .host(&cfg.db_host)
        .port(cfg.db_port)
        .user(&cfg.db_user)
        .password(&cfg.db_password)
        .dbname(&cfg.db_name)
        .to_owned()
}
