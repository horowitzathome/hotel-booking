mod config;
mod db;
mod errors;

use actix_web::{web, App, HttpServer};
use anyhow::Result;
use tracing_actix_web::TracingLogger;

pub struct AppState {
    pub pool: sqlx::PgPool,
}

fn init_tracing(is_development: bool) {
    use tracing_subscriber::EnvFilter;

    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,sqlx=warn,actix_web=info"));

    if is_development {
        tracing_subscriber::fmt().with_env_filter(filter).init();
    } else {
        tracing_subscriber::fmt()
            .json()
            .with_env_filter(filter)
            .init();
    }
}

#[actix_web::main]
async fn main() -> Result<()> {
    let config = config::AppConfig::from_env()?;

    init_tracing(config.is_development());

    tracing::info!(
        app = %config.app_name,
        env = %config.app_env,
        "starting application"
    );

    let pool = db::create_pool(&config).await?;
    let state = web::Data::new(AppState { pool });
    let addr = config.server_addr();

    tracing::info!(address = %addr, "listening");

    HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .app_data(state.clone())
    })
    .bind(&addr)?
    .run()
    .await?;

    Ok(())
}
