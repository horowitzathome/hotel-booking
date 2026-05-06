use actix_web::{web, HttpResponse};
use serde::Deserialize;

use crate::errors::AppError;
use crate::models::booking::{CreateBookingRequest, RecordPaymentRequest};
use crate::services::booking as svc;
use crate::AppState;

#[derive(Deserialize)]
pub struct BookingListQuery {
    pub house_id: Option<i64>,
    pub person_id: Option<i64>,
}

pub async fn list(
    state: web::Data<AppState>,
    query: web::Query<BookingListQuery>,
) -> Result<HttpResponse, AppError> {
    let bookings = svc::list(&state.pool, query.house_id, query.person_id).await?;
    Ok(HttpResponse::Ok().json(bookings))
}

pub async fn get(
    state: web::Data<AppState>,
    path: web::Path<i64>,
) -> Result<HttpResponse, AppError> {
    let booking = svc::get(&state.pool, path.into_inner()).await?;
    Ok(HttpResponse::Ok().json(booking))
}

pub async fn create(
    state: web::Data<AppState>,
    body: web::Json<CreateBookingRequest>,
) -> Result<HttpResponse, AppError> {
    let booking = svc::create(&state.pool, &body).await?;
    let location = format!("/api/v1/bookings/{}", booking.id);
    Ok(HttpResponse::Created()
        .insert_header(("Location", location))
        .json(booking))
}

pub async fn cancel(
    state: web::Data<AppState>,
    path: web::Path<i64>,
) -> Result<HttpResponse, AppError> {
    let booking = svc::cancel(&state.pool, path.into_inner()).await?;
    Ok(HttpResponse::Ok().json(booking))
}

pub async fn record_payment(
    state: web::Data<AppState>,
    path: web::Path<i64>,
    body: web::Json<RecordPaymentRequest>,
) -> Result<HttpResponse, AppError> {
    let booking = svc::record_payment(&state.pool, path.into_inner(), &body).await?;
    Ok(HttpResponse::Ok().json(booking))
}
