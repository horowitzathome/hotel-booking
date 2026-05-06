use actix_web::{web, HttpResponse};
use chrono::NaiveDate;
use serde::Deserialize;

use crate::errors::AppError;
use crate::models::calendar::{CreateCalendarRequest, UpdateCalendarPriceRequest};
use crate::services::calendar as svc;
use crate::AppState;

#[derive(Deserialize)]
pub struct CalendarListQuery {
    pub from: Option<NaiveDate>,
    pub to: Option<NaiveDate>,
}

#[derive(Deserialize)]
pub struct CalendarDeleteQuery {
    pub from: NaiveDate,
    pub to: NaiveDate,
}

pub async fn list(
    state: web::Data<AppState>,
    path: web::Path<i64>,
    query: web::Query<CalendarListQuery>,
) -> Result<HttpResponse, AppError> {
    let entries = svc::list(&state.pool, path.into_inner(), query.from, query.to).await?;
    Ok(HttpResponse::Ok().json(entries))
}

pub async fn get(
    state: web::Data<AppState>,
    path: web::Path<(i64, i64)>,
) -> Result<HttpResponse, AppError> {
    let (house_id, id) = path.into_inner();
    let entry = svc::get(&state.pool, house_id, id).await?;
    Ok(HttpResponse::Ok().json(entry))
}

pub async fn create(
    state: web::Data<AppState>,
    path: web::Path<i64>,
    body: web::Json<CreateCalendarRequest>,
) -> Result<HttpResponse, AppError> {
    let house_id = path.into_inner();
    let entries = svc::create(&state.pool, house_id, &body).await?;
    let location = format!("/api/v1/houses/{}/calendar", house_id);
    Ok(HttpResponse::Created()
        .insert_header(("Location", location))
        .json(entries))
}

pub async fn update_price(
    state: web::Data<AppState>,
    path: web::Path<i64>,
    body: web::Json<UpdateCalendarPriceRequest>,
) -> Result<HttpResponse, AppError> {
    let house_id = path.into_inner();
    let entries = svc::update_price(&state.pool, house_id, &body).await?;
    Ok(HttpResponse::Ok().json(entries))
}

pub async fn delete(
    state: web::Data<AppState>,
    path: web::Path<i64>,
    query: web::Query<CalendarDeleteQuery>,
) -> Result<HttpResponse, AppError> {
    svc::delete(&state.pool, path.into_inner(), query.from, query.to).await?;
    Ok(HttpResponse::NoContent().finish())
}
