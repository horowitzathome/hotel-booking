use actix_web::{web, HttpResponse};

use crate::errors::AppError;
use crate::models::house::{CreateHouseRequest, UpdateHouseRequest};
use crate::services::house as svc;
use crate::AppState;

pub async fn list(state: web::Data<AppState>) -> Result<HttpResponse, AppError> {
    let houses = svc::list(&state.pool).await?;
    Ok(HttpResponse::Ok().json(houses))
}

pub async fn get(
    state: web::Data<AppState>,
    path: web::Path<i64>,
) -> Result<HttpResponse, AppError> {
    let house = svc::get(&state.pool, path.into_inner()).await?;
    Ok(HttpResponse::Ok().json(house))
}

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

pub async fn update(
    state: web::Data<AppState>,
    path: web::Path<i64>,
    body: web::Json<UpdateHouseRequest>,
) -> Result<HttpResponse, AppError> {
    let house = svc::update(&state.pool, path.into_inner(), &body).await?;
    Ok(HttpResponse::Ok().json(house))
}

pub async fn delete(
    state: web::Data<AppState>,
    path: web::Path<i64>,
) -> Result<HttpResponse, AppError> {
    svc::delete(&state.pool, path.into_inner()).await?;
    Ok(HttpResponse::NoContent().finish())
}
