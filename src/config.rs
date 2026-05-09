use anyhow::{Context, Result};
use std::env;

pub struct AppConfig {
    pub database_url: String,
    pub database_max_connections: u32,
    pub database_min_connections: u32,
    pub database_connect_timeout_secs: u64,
    pub server_host: String,
    pub server_port: u16,
    pub app_name: String,
    pub app_env: String,
}

impl AppConfig {
    pub fn from_env() -> Result<Self> {
        dotenvy::dotenv().ok();

        Ok(AppConfig {
            database_url: required("DATABASE_URL")?,
            database_max_connections: parse("DATABASE_MAX_CONNECTIONS", 5)?,
            database_min_connections: parse("DATABASE_MIN_CONNECTIONS", 1)?,
            database_connect_timeout_secs: parse("DATABASE_CONNECT_TIMEOUT_SECS", 5)?,
            server_host: optional("SERVER_HOST", "0.0.0.0"),
            server_port: parse("SERVER_PORT", 8080)?,
            app_name: optional("APP_NAME", "rental-api"),
            app_env: optional("APP_ENV", "development"),
        })
    }

    pub fn server_addr(&self) -> String {
        format!("{}:{}", self.server_host, self.server_port)
    }

    pub fn is_development(&self) -> bool {
        self.app_env == "development"
    }
}

fn required(key: &str) -> Result<String> {
    env::var(key).with_context(|| format!("missing required env var: {key}"))
}

fn optional(key: &str, default: &str) -> String {
    env::var(key).unwrap_or_else(|_| default.to_string())
}

fn parse<T: std::str::FromStr>(key: &str, default: T) -> Result<T>
where
    T::Err: std::error::Error + Send + Sync + 'static,
{
    match env::var(key) {
        Ok(val) => val.parse().with_context(|| format!("invalid value for env var: {key}")),
        Err(_) => Ok(default),
    }
}
