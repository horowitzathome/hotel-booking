use actix_web::{HttpResponse, web};
use validator::Validate;

use crate::AppState;
use crate::errors::AppError;
use crate::models::address::{CreateAddressRequest, UpdateAddressRequest};
use crate::services::address as svc;

#[utoipa::path(
    get,
    path = "/api/v1/addresses/{id}",
    tag = "addresses",
    operation_id = "get_address",
    params(("id" = i64, Path, description = "Address id")),
    responses(
        (status = 200, description = "Address with embedded country", body = crate::models::address::Address),
        (status = 404, description = "Address not found")
    )
)]
pub async fn get(state: web::Data<AppState>, path: web::Path<i64>) -> Result<HttpResponse, AppError> {
    let address = svc::get(&state.pool, path.into_inner()).await?;
    Ok(HttpResponse::Ok().json(address))
}

#[utoipa::path(
    post,
    path = "/api/v1/addresses",
    tag = "addresses",
    operation_id = "create_address",
    request_body = CreateAddressRequest,
    responses(
        (status = 201, description = "Address created", body = crate::models::address::Address),
        (status = 409, description = "Unknown country_id")
    )
)]
pub async fn create(state: web::Data<AppState>, body: web::Json<CreateAddressRequest>) -> Result<HttpResponse, AppError> {
    body.validate()?;
    let address = svc::create(&state.pool, &body).await?;
    let location = format!("/api/v1/addresses/{}", address.id);
    Ok(HttpResponse::Created().insert_header(("Location", location)).json(address))
}

#[utoipa::path(
    put,
    path = "/api/v1/addresses/{id}",
    tag = "addresses",
    operation_id = "update_address",
    params(("id" = i64, Path, description = "Address id")),
    request_body = UpdateAddressRequest,
    responses(
        (status = 200, description = "Address updated", body = crate::models::address::Address),
        (status = 404, description = "Address not found"),
        (status = 409, description = "Unknown country_id")
    )
)]
pub async fn update(state: web::Data<AppState>, path: web::Path<i64>, body: web::Json<UpdateAddressRequest>) -> Result<HttpResponse, AppError> {
    body.validate()?;
    let address = svc::update(&state.pool, path.into_inner(), &body).await?;
    Ok(HttpResponse::Ok().json(address))
}

#[utoipa::path(
    delete,
    path = "/api/v1/addresses/{id}",
    tag = "addresses",
    operation_id = "delete_address",
    params(("id" = i64, Path, description = "Address id")),
    responses(
        (status = 204, description = "Address deleted"),
        (status = 404, description = "Address not found"),
        (status = 409, description = "Address is referenced by a house")
    )
)]
pub async fn delete(state: web::Data<AppState>, path: web::Path<i64>) -> Result<HttpResponse, AppError> {
    svc::delete(&state.pool, path.into_inner()).await?;
    Ok(HttpResponse::NoContent().finish())
}
