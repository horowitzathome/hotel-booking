use chrono::NaiveDate;
use sqlx::PgPool;

use crate::errors::AppError;
use crate::models::booking::{Booking, BookingStatus, CreateBookingRequest, CreateBookingResponse, RecordPaymentRequest};
use crate::models::calendar::CalendarStatus;
use crate::repositories::booking as repo;

fn validate_date_range(from: NaiveDate, to: NaiveDate) -> Result<(), AppError> {
    if from > to {
        return Err(AppError::UnprocessableEntity("'from' must not be after 'to'".into()));
    }
    Ok(())
}

#[tracing::instrument(skip(pool), fields(layer = "service"))]
pub async fn list(pool: &PgPool, house_id: Option<i64>, person_id: Option<i64>, limit: Option<i64>, offset: Option<i64>) -> Result<Vec<Booking>, AppError> {
    repo::find_all(pool, house_id, person_id, limit, offset).await
}

#[tracing::instrument(skip(pool), fields(layer = "service"))]
pub async fn get(pool: &PgPool, id: i64) -> Result<Booking, AppError> {
    repo::find_by_id(pool, id).await
}

#[tracing::instrument(skip(pool, req), fields(layer = "service", house_id = req.house_id, person_id = req.person_id))]
pub async fn create(pool: &PgPool, req: &CreateBookingRequest) -> Result<CreateBookingResponse, AppError> {
    validate_date_range(req.from, req.to)?;

    let mut tx = pool.begin().await?;
    let entries = repo::fetch_calendar_for_booking(&mut tx, req.house_id, req.from, req.to).await?;

    let expected_days = (req.to - req.from).num_days() + 1;
    if entries.len() as i64 != expected_days {
        return Err(AppError::UnprocessableEntity("not all days in the requested range have calendar entries".into()));
    }
    if entries.iter().any(|e| e.status != CalendarStatus::Rentable) {
        return Err(AppError::UnprocessableEntity("all days in range must be in status 'Rentable'".into()));
    }

    let (booking_id, expected_total_price) = repo::do_create_booking(&mut tx, req, &entries).await?;
    tx.commit().await?;

    let booking = repo::find_by_id(pool, booking_id).await?;
    Ok(CreateBookingResponse { booking, expected_total_price })
}

#[tracing::instrument(skip(pool), fields(layer = "service"))]
pub async fn cancel(pool: &PgPool, id: i64) -> Result<Booking, AppError> {
    let mut tx = pool.begin().await?;
    let locked = repo::lock_for_write(&mut tx, id).await?;

    if locked.status == BookingStatus::Cancelled {
        return Err(AppError::UnprocessableEntity("booking is already cancelled".into()));
    }

    repo::do_cancel(&mut tx, &locked).await?;
    tx.commit().await?;

    repo::find_by_id(pool, id).await
}

#[tracing::instrument(skip(pool, req), fields(layer = "service"))]
pub async fn record_payment(pool: &PgPool, id: i64, req: &RecordPaymentRequest) -> Result<Booking, AppError> {
    let mut tx = pool.begin().await?;
    let locked = repo::lock_for_write(&mut tx, id).await?;

    if locked.status == BookingStatus::Cancelled {
        return Err(AppError::UnprocessableEntity("cannot record payment for a cancelled booking".into()));
    }

    repo::do_record_payment(&mut tx, id, req).await?;
    tx.commit().await?;

    repo::find_by_id(pool, id).await
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(s: &str) -> NaiveDate {
        NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap()
    }

    #[test]
    fn create_should_fail_when_from_is_after_to() {
        assert!(matches!(validate_date_range(d("2024-07-10"), d("2024-07-01")), Err(AppError::UnprocessableEntity(_))));
    }

    #[test]
    fn create_should_accept_same_day_range() {
        assert!(validate_date_range(d("2024-07-01"), d("2024-07-01")).is_ok());
    }

    #[test]
    fn create_should_accept_valid_range() {
        assert!(validate_date_range(d("2024-07-01"), d("2024-07-31")).is_ok());
    }
}
