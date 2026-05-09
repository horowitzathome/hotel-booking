use sqlx::PgPool;

use crate::errors::AppError;
use crate::models::booking::{Booking, CreateBookingRequest, RecordPaymentRequest};
use crate::repositories::booking as repo;

#[tracing::instrument(skip(pool), fields(layer = "service"))]
pub async fn list(pool: &PgPool, house_id: Option<i64>, person_id: Option<i64>) -> Result<Vec<Booking>, AppError> {
    repo::find_all(pool, house_id, person_id).await
}

#[tracing::instrument(skip(pool), fields(layer = "service"))]
pub async fn get(pool: &PgPool, id: i64) -> Result<Booking, AppError> {
    repo::find_by_id(pool, id).await
}

#[tracing::instrument(skip(pool, req), fields(layer = "service", house_id = req.house_id, person_id = req.person_id))]
pub async fn create(pool: &PgPool, req: &CreateBookingRequest) -> Result<Booking, AppError> {
    if req.from > req.to {
        return Err(AppError::UnprocessableEntity("'from' must not be after 'to'".into()));
    }
    let (mut booking, expected_total_price) = repo::create(pool, req).await?;
    booking.expected_total_price = Some(expected_total_price);
    Ok(booking)
}

#[tracing::instrument(skip(pool), fields(layer = "service"))]
pub async fn cancel(pool: &PgPool, id: i64) -> Result<Booking, AppError> {
    repo::cancel(pool, id).await
}

#[tracing::instrument(skip(pool, req), fields(layer = "service"))]
pub async fn record_payment(pool: &PgPool, id: i64, req: &RecordPaymentRequest) -> Result<Booking, AppError> {
    repo::record_payment(pool, id, req).await
}
