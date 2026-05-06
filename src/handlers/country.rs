use actix_web::{web, HttpResponse};

use crate::errors::AppError;
use crate::models::country::{CreateCountryRequest, UpdateCountryRequest};
use crate::services::country as svc;
use crate::AppState;

pub async fn list(state: web::Data<AppState>) -> Result<HttpResponse, AppError> {
    let countries = svc::list(&state.pool).await?;
    Ok(HttpResponse::Ok().json(countries))
}

pub async fn get(
    state: web::Data<AppState>,
    path: web::Path<i64>,
) -> Result<HttpResponse, AppError> {
    let country = svc::get(&state.pool, path.into_inner()).await?;
    Ok(HttpResponse::Ok().json(country))
}

pub async fn create(
    state: web::Data<AppState>,
    body: web::Json<CreateCountryRequest>,
) -> Result<HttpResponse, AppError> {
    let country = svc::create(&state.pool, &body).await?;
    let location = format!("/api/v1/countries/{}", country.id);
    Ok(HttpResponse::Created()
        .insert_header(("Location", location))
        .json(country))
}

pub async fn update(
    state: web::Data<AppState>,
    path: web::Path<i64>,
    body: web::Json<UpdateCountryRequest>,
) -> Result<HttpResponse, AppError> {
    let country = svc::update(&state.pool, path.into_inner(), &body).await?;
    Ok(HttpResponse::Ok().json(country))
}

pub async fn delete(
    state: web::Data<AppState>,
    path: web::Path<i64>,
) -> Result<HttpResponse, AppError> {
    svc::delete(&state.pool, path.into_inner()).await?;
    Ok(HttpResponse::NoContent().finish())
}
