use actix_web::{HttpResponse, Responder, web};
use serde_json::json;

use crate::AppState;

#[utoipa::path(
    get,
    path = "/health",
    tag = "health",
    responses(
        (status = 200, description = "Service is up and database is reachable"),
        (status = 503, description = "Database is unreachable")
    )
)]
pub async fn health(state: web::Data<AppState>) -> impl Responder {
    match state.pool.acquire().await {
        Ok(_) => HttpResponse::Ok().json(json!({ "status": "ok" })),
        Err(_) => HttpResponse::ServiceUnavailable().json(json!({ "status": "unavailable" })),
    }
}
