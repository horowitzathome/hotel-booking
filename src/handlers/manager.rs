use actix_web::{web, HttpResponse};
use validator::Validate;

use crate::errors::AppError;
use crate::models::manager::{CreateManagerRequest, UpdateManagerRequest};
use crate::services::manager as svc;
use crate::AppState;

#[utoipa::path(
    get,
    path = "/api/v1/managers",
    tag = "managers",
    operation_id = "list_managers",
    responses((status = 200, description = "List of managers", body = [crate::models::manager::Manager]))
)]
pub async fn list(state: web::Data<AppState>) -> Result<HttpResponse, AppError> {
    let managers = svc::list(&state.pool).await?;
    Ok(HttpResponse::Ok().json(managers))
}

#[utoipa::path(
    get,
    path = "/api/v1/managers/{id}",
    tag = "managers",
    operation_id = "get_manager",
    params(("id" = i64, Path, description = "Manager id")),
    responses(
        (status = 200, description = "Manager", body = crate::models::manager::Manager),
        (status = 404, description = "Manager not found")
    )
)]
pub async fn get(
    state: web::Data<AppState>,
    path: web::Path<i64>,
) -> Result<HttpResponse, AppError> {
    let manager = svc::get(&state.pool, path.into_inner()).await?;
    Ok(HttpResponse::Ok().json(manager))
}

#[utoipa::path(
    post,
    path = "/api/v1/managers",
    tag = "managers",
    operation_id = "create_manager",
    request_body = CreateManagerRequest,
    responses(
        (status = 201, description = "Manager created", body = crate::models::manager::Manager),
        (status = 409, description = "Duplicate email")
    )
)]
pub async fn create(
    state: web::Data<AppState>,
    body: web::Json<CreateManagerRequest>,
) -> Result<HttpResponse, AppError> {
    body.validate()?;
    let manager = svc::create(&state.pool, &body).await?;
    let location = format!("/api/v1/managers/{}", manager.id);
    Ok(HttpResponse::Created()
        .insert_header(("Location", location))
        .json(manager))
}

#[utoipa::path(
    put,
    path = "/api/v1/managers/{id}",
    tag = "managers",
    operation_id = "update_manager",
    params(("id" = i64, Path, description = "Manager id")),
    request_body = UpdateManagerRequest,
    responses(
        (status = 200, description = "Manager updated", body = crate::models::manager::Manager),
        (status = 404, description = "Manager not found"),
        (status = 409, description = "Unique constraint conflict")
    )
)]
pub async fn update(
    state: web::Data<AppState>,
    path: web::Path<i64>,
    body: web::Json<UpdateManagerRequest>,
) -> Result<HttpResponse, AppError> {
    body.validate()?;
    let manager = svc::update(&state.pool, path.into_inner(), &body).await?;
    Ok(HttpResponse::Ok().json(manager))
}

#[utoipa::path(
    delete,
    path = "/api/v1/managers/{id}",
    tag = "managers",
    operation_id = "delete_manager",
    params(("id" = i64, Path, description = "Manager id")),
    responses(
        (status = 204, description = "Manager deleted"),
        (status = 404, description = "Manager not found"),
        (status = 409, description = "Manager is referenced by a house")
    )
)]
pub async fn delete(
    state: web::Data<AppState>,
    path: web::Path<i64>,
) -> Result<HttpResponse, AppError> {
    svc::delete(&state.pool, path.into_inner()).await?;
    Ok(HttpResponse::NoContent().finish())
}
