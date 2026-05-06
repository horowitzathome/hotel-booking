use actix_web::{web, HttpResponse};

use crate::errors::AppError;
use crate::models::person::{CreatePersonRequest, UpdatePersonRequest};
use crate::services::person as svc;
use crate::AppState;

pub async fn list(state: web::Data<AppState>) -> Result<HttpResponse, AppError> {
    let persons = svc::list(&state.pool).await?;
    Ok(HttpResponse::Ok().json(persons))
}

pub async fn get(
    state: web::Data<AppState>,
    path: web::Path<i64>,
) -> Result<HttpResponse, AppError> {
    let person = svc::get(&state.pool, path.into_inner()).await?;
    Ok(HttpResponse::Ok().json(person))
}

pub async fn create(
    state: web::Data<AppState>,
    body: web::Json<CreatePersonRequest>,
) -> Result<HttpResponse, AppError> {
    let person = svc::create(&state.pool, &body).await?;
    let location = format!("/api/v1/persons/{}", person.id);
    Ok(HttpResponse::Created()
        .insert_header(("Location", location))
        .json(person))
}

pub async fn update(
    state: web::Data<AppState>,
    path: web::Path<i64>,
    body: web::Json<UpdatePersonRequest>,
) -> Result<HttpResponse, AppError> {
    let person = svc::update(&state.pool, path.into_inner(), &body).await?;
    Ok(HttpResponse::Ok().json(person))
}

pub async fn delete(
    state: web::Data<AppState>,
    path: web::Path<i64>,
) -> Result<HttpResponse, AppError> {
    svc::delete(&state.pool, path.into_inner()).await?;
    Ok(HttpResponse::NoContent().finish())
}
