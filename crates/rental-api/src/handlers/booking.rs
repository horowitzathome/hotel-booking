use actix_web::{HttpResponse, web};
use serde::Deserialize;
use validator::Validate;

use crate::AppState;
use crate::errors::AppError;
use crate::models::booking::{CreateBookingRequest, CreateBookingResponse, RecordPaymentRequest};
use crate::services::booking as svc;

#[derive(Deserialize)]
pub struct BookingListQuery {
    pub house_id: Option<i64>,
    pub person_id: Option<i64>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[utoipa::path(
    get,
    path = "/api/v1/bookings",
    tag = "bookings",
    operation_id = "list_bookings",
    params(
        ("house_id" = Option<i64>, Query, description = "Filter by house id"),
        ("person_id" = Option<i64>, Query, description = "Filter by person id"),
        ("limit" = Option<i64>, Query, description = "Maximum number of results to return"),
        ("offset" = Option<i64>, Query, description = "Number of results to skip"),
    ),
    responses((status = 200, description = "List of bookings", body = [crate::models::booking::Booking]))
)]
pub async fn list(state: web::Data<AppState>, query: web::Query<BookingListQuery>) -> Result<HttpResponse, AppError> {
    let bookings = svc::list(&state.pool, query.house_id, query.person_id, query.limit, query.offset).await?;
    Ok(HttpResponse::Ok().json(bookings))
}

#[utoipa::path(
    get,
    path = "/api/v1/bookings/{id}",
    tag = "bookings",
    operation_id = "get_booking",
    params(("id" = i64, Path, description = "Booking id")),
    responses(
        (status = 200, description = "Booking", body = crate::models::booking::Booking),
        (status = 404, description = "Booking not found")
    )
)]
pub async fn get(state: web::Data<AppState>, path: web::Path<i64>) -> Result<HttpResponse, AppError> {
    let booking = svc::get(&state.pool, path.into_inner()).await?;
    Ok(HttpResponse::Ok().json(booking))
}

#[utoipa::path(
    post,
    path = "/api/v1/bookings",
    tag = "bookings",
    operation_id = "create_booking",
    request_body = CreateBookingRequest,
    responses(
        (status = 201, description = "Booking created (includes expected_total_price)", body = CreateBookingResponse),
        (status = 409, description = "Unknown house_id or person_id"),
        (status = 422, description = "from > to or any day not Rentable")
    )
)]
pub async fn create(state: web::Data<AppState>, body: web::Json<CreateBookingRequest>) -> Result<HttpResponse, AppError> {
    body.validate()?;
    let booking = svc::create(&state.pool, &body).await?;
    let location = format!("/api/v1/bookings/{}", booking.id);
    Ok(HttpResponse::Created().insert_header(("Location", location)).json(booking))
}

#[utoipa::path(
    post,
    path = "/api/v1/bookings/{id}/cancel",
    tag = "bookings",
    operation_id = "cancel_booking",
    params(("id" = i64, Path, description = "Booking id")),
    responses(
        (status = 200, description = "Booking cancelled", body = crate::models::booking::Booking),
        (status = 404, description = "Booking not found"),
        (status = 422, description = "Booking already cancelled")
    )
)]
pub async fn cancel(state: web::Data<AppState>, path: web::Path<i64>) -> Result<HttpResponse, AppError> {
    let booking = svc::cancel(&state.pool, path.into_inner()).await?;
    Ok(HttpResponse::Ok().json(booking))
}

#[utoipa::path(
    post,
    path = "/api/v1/bookings/{id}/payment",
    tag = "bookings",
    operation_id = "record_booking_payment",
    params(("id" = i64, Path, description = "Booking id")),
    request_body = RecordPaymentRequest,
    responses(
        (status = 200, description = "Payment recorded", body = crate::models::booking::Booking),
        (status = 404, description = "Booking not found"),
        (status = 422, description = "Booking is cancelled")
    )
)]
pub async fn record_payment(state: web::Data<AppState>, path: web::Path<i64>, body: web::Json<RecordPaymentRequest>) -> Result<HttpResponse, AppError> {
    body.validate()?;
    let booking = svc::record_payment(&state.pool, path.into_inner(), &body).await?;
    Ok(HttpResponse::Ok().json(booking))
}
