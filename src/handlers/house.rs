use actix_web::{web, HttpResponse};

use crate::errors::AppError;
use crate::models::house::{CreateHouseRequest, UpdateHouseRequest};
use crate::services::house as svc;
use crate::AppState;

#[utoipa::path(
    get,
    path = "/api/v1/houses",
    tag = "houses",
    operation_id = "list_houses",
    responses((status = 200, description = "List of houses", body = [crate::models::house::House]))
)]
pub async fn list(state: web::Data<AppState>) -> Result<HttpResponse, AppError> {
    let houses = svc::list(&state.pool).await?;
    Ok(HttpResponse::Ok().json(houses))
}

#[utoipa::path(
    get,
    path = "/api/v1/houses/{id}",
    tag = "houses",
    operation_id = "get_house",
    params(("id" = i64, Path, description = "House id")),
    responses(
        (status = 200, description = "House", body = crate::models::house::House),
        (status = 404, description = "House not found")
    )
)]
pub async fn get(
    state: web::Data<AppState>,
    path: web::Path<i64>,
) -> Result<HttpResponse, AppError> {
    let house = svc::get(&state.pool, path.into_inner()).await?;
    Ok(HttpResponse::Ok().json(house))
}

#[utoipa::path(
    post,
    path = "/api/v1/houses",
    tag = "houses",
    operation_id = "create_house",
    request_body = CreateHouseRequest,
    responses(
        (status = 201, description = "House created", body = crate::models::house::House),
        (status = 409, description = "Unknown address_id or manager_id")
    )
)]
pub async fn create(
    state: web::Data<AppState>,
    body: web::Json<CreateHouseRequest>,
) -> Result<HttpResponse, AppError> {
    let house = svc::create(&state.pool, &body).await?;
    let location = format!("/api/v1/houses/{}", house.id);
    Ok(HttpResponse::Created()
        .insert_header(("Location", location))
        .json(house))
}

#[utoipa::path(
    put,
    path = "/api/v1/houses/{id}",
    tag = "houses",
    operation_id = "update_house",
    params(("id" = i64, Path, description = "House id")),
    request_body = UpdateHouseRequest,
    responses(
        (status = 200, description = "House updated", body = crate::models::house::House),
        (status = 404, description = "House not found"),
        (status = 409, description = "Unknown address_id or manager_id")
    )
)]
pub async fn update(
    state: web::Data<AppState>,
    path: web::Path<i64>,
    body: web::Json<UpdateHouseRequest>,
) -> Result<HttpResponse, AppError> {
    let house = svc::update(&state.pool, path.into_inner(), &body).await?;
    Ok(HttpResponse::Ok().json(house))
}

#[utoipa::path(
    delete,
    path = "/api/v1/houses/{id}",
    tag = "houses",
    operation_id = "delete_house",
    params(("id" = i64, Path, description = "House id")),
    responses(
        (status = 204, description = "House deleted"),
        (status = 404, description = "House not found"),
        (status = 409, description = "House is referenced by a booking")
    )
)]
pub async fn delete(
    state: web::Data<AppState>,
    path: web::Path<i64>,
) -> Result<HttpResponse, AppError> {
    svc::delete(&state.pool, path.into_inner()).await?;
    Ok(HttpResponse::NoContent().finish())
}
