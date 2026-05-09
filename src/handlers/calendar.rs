use actix_web::{web, HttpResponse};
use chrono::NaiveDate;
use serde::Deserialize;
use validator::Validate;

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

#[utoipa::path(
    get,
    path = "/api/v1/houses/{house_id}/calendar",
    tag = "calendar",
    operation_id = "list_calendar_entries",
    params(
        ("house_id" = i64, Path, description = "House id"),
        ("from" = Option<chrono::NaiveDate>, Query, description = "Start date (inclusive)"),
        ("to" = Option<chrono::NaiveDate>, Query, description = "End date (inclusive)")
    ),
    responses((status = 200, description = "Calendar entries", body = [crate::models::calendar::CalendarEntry]))
)]
pub async fn list(
    state: web::Data<AppState>,
    path: web::Path<i64>,
    query: web::Query<CalendarListQuery>,
) -> Result<HttpResponse, AppError> {
    let entries = svc::list(&state.pool, path.into_inner(), query.from, query.to).await?;
    Ok(HttpResponse::Ok().json(entries))
}

#[utoipa::path(
    get,
    path = "/api/v1/houses/{house_id}/calendar/{id}",
    tag = "calendar",
    operation_id = "get_calendar_entry",
    params(
        ("house_id" = i64, Path, description = "House id"),
        ("id" = i64, Path, description = "Calendar entry id")
    ),
    responses(
        (status = 200, description = "Calendar entry", body = crate::models::calendar::CalendarEntry),
        (status = 404, description = "Entry not found")
    )
)]
pub async fn get(
    state: web::Data<AppState>,
    path: web::Path<(i64, i64)>,
) -> Result<HttpResponse, AppError> {
    let (house_id, id) = path.into_inner();
    let entry = svc::get(&state.pool, house_id, id).await?;
    Ok(HttpResponse::Ok().json(entry))
}

#[utoipa::path(
    post,
    path = "/api/v1/houses/{house_id}/calendar",
    tag = "calendar",
    operation_id = "create_calendar_entries",
    params(("house_id" = i64, Path, description = "House id")),
    request_body = CreateCalendarRequest,
    responses(
        (status = 201, description = "Newly created entries (existing days skipped)", body = [crate::models::calendar::CalendarEntry]),
        (status = 422, description = "Invalid status, Rented disallowed, or from > to")
    )
)]
pub async fn create(
    state: web::Data<AppState>,
    path: web::Path<i64>,
    body: web::Json<CreateCalendarRequest>,
) -> Result<HttpResponse, AppError> {
    body.validate()?;
    let house_id = path.into_inner();
    let entries = svc::create(&state.pool, house_id, &body).await?;
    let location = format!("/api/v1/houses/{}/calendar", house_id);
    Ok(HttpResponse::Created()
        .insert_header(("Location", location))
        .json(entries))
}

#[utoipa::path(
    patch,
    path = "/api/v1/houses/{house_id}/calendar",
    tag = "calendar",
    operation_id = "update_calendar_prices",
    params(("house_id" = i64, Path, description = "House id")),
    request_body = UpdateCalendarPriceRequest,
    responses(
        (status = 200, description = "Updated entries", body = [crate::models::calendar::CalendarEntry]),
        (status = 422, description = "from > to")
    )
)]
pub async fn update_price(
    state: web::Data<AppState>,
    path: web::Path<i64>,
    body: web::Json<UpdateCalendarPriceRequest>,
) -> Result<HttpResponse, AppError> {
    body.validate()?;
    let house_id = path.into_inner();
    let entries = svc::update_price(&state.pool, house_id, &body).await?;
    Ok(HttpResponse::Ok().json(entries))
}

#[utoipa::path(
    delete,
    path = "/api/v1/houses/{house_id}/calendar",
    tag = "calendar",
    operation_id = "delete_calendar_entries",
    params(
        ("house_id" = i64, Path, description = "House id"),
        ("from" = chrono::NaiveDate, Query, description = "Start date (inclusive)"),
        ("to" = chrono::NaiveDate, Query, description = "End date (inclusive)")
    ),
    responses(
        (status = 204, description = "Entries deleted"),
        (status = 409, description = "An entry is referenced by a non-cancelled booking"),
        (status = 422, description = "from > to")
    )
)]
pub async fn delete(
    state: web::Data<AppState>,
    path: web::Path<i64>,
    query: web::Query<CalendarDeleteQuery>,
) -> Result<HttpResponse, AppError> {
    svc::delete(&state.pool, path.into_inner(), query.from, query.to).await?;
    Ok(HttpResponse::NoContent().finish())
}
