use actix_web::{web, HttpResponse};

use crate::errors::AppError;
use crate::models::manager::{CreateManagerRequest, UpdateManagerRequest};
use crate::services::manager as svc;
use crate::AppState;

pub async fn list(state: web::Data<AppState>) -> Result<HttpResponse, AppError> {
    let managers = svc::list(&state.pool).await?;
    Ok(HttpResponse::Ok().json(managers))
}

pub async fn get(
    state: web::Data<AppState>,
    path: web::Path<i64>,
) -> Result<HttpResponse, AppError> {
    let manager = svc::get(&state.pool, path.into_inner()).await?;
    Ok(HttpResponse::Ok().json(manager))
}

pub async fn create(
    state: web::Data<AppState>,
    body: web::Json<CreateManagerRequest>,
) -> Result<HttpResponse, AppError> {
    let manager = svc::create(&state.pool, &body).await?;
    let location = format!("/api/v1/managers/{}", manager.id);
    Ok(HttpResponse::Created()
        .insert_header(("Location", location))
        .json(manager))
}

pub async fn update(
    state: web::Data<AppState>,
    path: web::Path<i64>,
    body: web::Json<UpdateManagerRequest>,
) -> Result<HttpResponse, AppError> {
    let manager = svc::update(&state.pool, path.into_inner(), &body).await?;
    Ok(HttpResponse::Ok().json(manager))
}

pub async fn delete(
    state: web::Data<AppState>,
    path: web::Path<i64>,
) -> Result<HttpResponse, AppError> {
    svc::delete(&state.pool, path.into_inner()).await?;
    Ok(HttpResponse::NoContent().finish())
}
