use crate::config::AppConfig;
use anyhow::{Context, Result};
use sqlx::postgres::PgPoolOptions;
use std::time::Duration;

pub async fn create_pool(config: &AppConfig) -> Result<sqlx::PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(config.database_max_connections)
        .min_connections(config.database_min_connections)
        .acquire_timeout(Duration::from_secs(config.database_connect_timeout_secs))
        .connect(&config.database_url)
        .await
        .context("failed to connect to database")?;

    tracing::info!(max_connections = config.database_max_connections, "database pool created");

    sqlx::migrate!("./migrations").run(&pool).await.context("failed to run database migrations")?;

    tracing::info!("database migrations applied");

    Ok(pool)
}
