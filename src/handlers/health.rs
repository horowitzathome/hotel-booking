use actix_web::{HttpResponse, Responder};
use serde_json::json;

#[utoipa::path(
    get,
    path = "/health",
    tag = "health",
    responses((status = 200, description = "Service is up"))
)]
pub async fn health() -> impl Responder {
    HttpResponse::Ok().json(json!({ "status": "ok" }))
}
