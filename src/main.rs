use claude_test::{config, db, handlers, openapi::ApiDoc, routes, AppState};

use actix_web::{web, App, HttpServer};
use actix_web_prom::PrometheusMetricsBuilder;
use anyhow::Result;
use tracing_actix_web::TracingLogger;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

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

    let prometheus = PrometheusMetricsBuilder::new("api")
        .endpoint("/metrics")
        .build()
        .expect("failed to build prometheus metrics");

    tracing::info!(address = %addr, "listening");

    let openapi = ApiDoc::openapi();

    HttpServer::new(move || {
        App::new()
            .wrap(prometheus.clone())
            .wrap(TracingLogger::default())
            .app_data(state.clone())
            .route("/health", web::get().to(handlers::health::health))
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}")
                    .url("/api-docs/openapi.json", openapi.clone()),
            )
            .configure(routes::configure)
    })
    .bind(&addr)?
    .run()
    .await?;

    Ok(())
}
