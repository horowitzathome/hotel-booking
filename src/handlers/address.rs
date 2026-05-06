use actix_web::{web, HttpResponse};

use crate::errors::AppError;
use crate::models::address::{CreateAddressRequest, UpdateAddressRequest};
use crate::services::address as svc;
use crate::AppState;

pub async fn get(
    state: web::Data<AppState>,
    path: web::Path<i64>,
) -> Result<HttpResponse, AppError> {
    let address = svc::get(&state.pool, path.into_inner()).await?;
    Ok(HttpResponse::Ok().json(address))
}

pub async fn create(
    state: web::Data<AppState>,
    body: web::Json<CreateAddressRequest>,
) -> Result<HttpResponse, AppError> {
    let address = svc::create(&state.pool, &body).await?;
    let location = format!("/api/v1/addresses/{}", address.id);
    Ok(HttpResponse::Created()
        .insert_header(("Location", location))
        .json(address))
}

pub async fn update(
    state: web::Data<AppState>,
    path: web::Path<i64>,
    body: web::Json<UpdateAddressRequest>,
) -> Result<HttpResponse, AppError> {
    let address = svc::update(&state.pool, path.into_inner(), &body).await?;
    Ok(HttpResponse::Ok().json(address))
}

pub async fn delete(
    state: web::Data<AppState>,
    path: web::Path<i64>,
) -> Result<HttpResponse, AppError> {
    svc::delete(&state.pool, path.into_inner()).await?;
    Ok(HttpResponse::NoContent().finish())
}
