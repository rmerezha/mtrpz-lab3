use std::path::Path;

use figment::providers::{Env, Format, Serialized, Toml};
use figment::Figment;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub app_log: String,
    pub server: String,
    pub http_tracing: bool,

    pub db_host: String,
    pub db_port: u16,
    pub db_name: String,
    pub db_user: String,
    pub db_password: String,
    pub db_pool_max_size: usize,

    pub session_lifetime: i64,
    pub refresh_token_lifetime: i64,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            app_log: "info".to_string(),
            server: "0.0.0.0:8080".to_string(),
            http_tracing: true,

            db_host: "postgres".to_string(),
            db_port: 5432,
            db_name: "test".to_string(),
            db_user: "test".to_string(),
            db_password: "password".to_string(),
            db_pool_max_size: 100,

            session_lifetime: 3600,
            refresh_token_lifetime: 3600 * 24 * 100,
        }
    }
}


#[derive(Debug, Clone, Default)]
pub struct AppConfig {
    pub config: Config,
}

impl AppConfig {
    pub fn new() -> Self {
        let mut figment = Figment::new().merge(Serialized::defaults(Config::default()));

        if Path::new("app.toml").exists() {
            figment = figment
                .merge(Toml::file("app.toml"))
                .merge(Env::prefixed("APP_"));
        } else {
            figment = figment.merge(Env::prefixed("APP_"));
        }

        let config: Config = figment.extract().unwrap();

        AppConfig { config }
    }
}
