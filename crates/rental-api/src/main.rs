use rental_api::{AppState, config, db, handlers, openapi::ApiDoc, routes};

use actix_web::dev::Service;
use actix_web::http::header::{HeaderName, HeaderValue};
use actix_web::{App, HttpMessage, HttpServer, web};
use actix_web_prom::PrometheusMetricsBuilder;
use anyhow::Result;
use opentelemetry::trace::TracerProvider as _;
use opentelemetry_otlp::{SpanExporter, WithExportConfig};
use opentelemetry_sdk::Resource;
use opentelemetry_sdk::trace::SdkTracerProvider;
use tracing_actix_web::{RequestId, TracingLogger};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Layer};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

fn init_tracing(is_development: bool, service_name: &str) -> Option<SdkTracerProvider> {
    let default_filter = "info,sqlx=warn,actix_web=info,h2=off,hyper=off,tonic=off,tower=off,reqwest=off";
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(default_filter));

    let fmt_layer = if is_development {
        tracing_subscriber::fmt::layer().boxed()
    } else {
        tracing_subscriber::fmt::layer().json().boxed()
    };

    let provider = std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT").ok().map(|endpoint| build_otel_provider(&endpoint, service_name));

    let otel_layer = provider.as_ref().map(|p| {
        let tracer = p.tracer(service_name.to_string());
        tracing_opentelemetry::layer().with_tracer(tracer)
    });

    tracing_subscriber::registry().with(filter).with(fmt_layer).with(otel_layer).init();

    if let Some(p) = provider.as_ref() {
        opentelemetry::global::set_tracer_provider(p.clone());
    }
    provider
}

fn build_otel_provider(endpoint: &str, service_name: &str) -> SdkTracerProvider {
    let exporter = SpanExporter::builder().with_tonic().with_endpoint(endpoint).build().expect("failed to build OTLP span exporter");

    let resource = Resource::builder().with_service_name(service_name.to_string()).build();

    SdkTracerProvider::builder().with_resource(resource).with_batch_exporter(exporter).build()
}

#[actix_web::main]
async fn main() -> Result<()> {
    let config = config::AppConfig::from_env()?;

    let otel_provider = init_tracing(config.is_development(), &config.app_name);

    tracing::info!(
        app = %config.app_name,
        env = %config.app_env,
        otlp = otel_provider.is_some(),
        "starting application"
    );

    let pool = db::create_pool(&config).await?;
    let state = web::Data::new(AppState { pool });
    let addr = config.server_addr();

    let prometheus = PrometheusMetricsBuilder::new("api")
        .endpoint("/metrics")
        .exclude_regex(r"^/swagger-ui(/.*)?$")
        .exclude("/api-docs/openapi.json")
        .build()
        .expect("failed to build prometheus metrics");

    tracing::info!(address = %addr, "listening");

    let openapi = ApiDoc::openapi();

    HttpServer::new(move || {
        App::new()
            .wrap(prometheus.clone())
            .wrap_fn(|req, srv| {
                let request_id = req.extensions().get::<RequestId>().copied();
                let fut = srv.call(req);
                async move {
                    let mut res = fut.await?;
                    if let Some(id) = request_id
                        && let Ok(value) = HeaderValue::from_str(&format!("{id}"))
                    {
                        res.headers_mut().insert(HeaderName::from_static("x-request-id"), value);
                    }
                    Ok(res)
                }
            })
            .wrap(TracingLogger::default())
            .app_data(state.clone())
            .route("/health", web::get().to(handlers::health::health))
            .service(SwaggerUi::new("/swagger-ui/{_:.*}").url("/api-docs/openapi.json", openapi.clone()))
            .configure(routes::configure)
    })
    .bind(&addr)?
    .run()
    .await?;

    if let Some(provider) = otel_provider
        && let Err(err) = provider.shutdown()
    {
        tracing::warn!(error = %err, "OpenTelemetry shutdown reported an error");
    }

    Ok(())
}
