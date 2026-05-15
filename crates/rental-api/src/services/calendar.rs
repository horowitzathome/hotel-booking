use chrono::NaiveDate;
use sqlx::PgPool;

use crate::errors::AppError;
use crate::models::calendar::{CalendarEntry, CalendarStatus, CreateCalendarRequest, UpdateCalendarPriceRequest};
use crate::repositories::calendar as repo;

fn validate_create_status(status: CalendarStatus) -> Result<(), AppError> {
    if status == CalendarStatus::Rented {
        return Err(AppError::UnprocessableEntity("status 'Rented' cannot be set via calendar endpoints".into()));
    }
    Ok(())
}

fn validate_date_range(from: NaiveDate, to: NaiveDate) -> Result<(), AppError> {
    if from > to {
        return Err(AppError::UnprocessableEntity("'from' must not be after 'to'".into()));
    }
    Ok(())
}

#[tracing::instrument(skip(pool), fields(layer = "service"))]
pub async fn list(pool: &PgPool, house_id: i64, from: Option<NaiveDate>, to: Option<NaiveDate>) -> Result<Vec<CalendarEntry>, AppError> {
    repo::find_all(pool, house_id, from, to).await
}

#[tracing::instrument(skip(pool), fields(layer = "service"))]
pub async fn get(pool: &PgPool, house_id: i64, id: i64) -> Result<CalendarEntry, AppError> {
    repo::find_by_id(pool, house_id, id).await
}

#[tracing::instrument(skip(pool, req), fields(layer = "service"))]
pub async fn create(pool: &PgPool, house_id: i64, req: &CreateCalendarRequest) -> Result<Vec<CalendarEntry>, AppError> {
    validate_create_status(req.status)?;
    validate_date_range(req.from, req.to)?;
    repo::create(pool, house_id, req).await
}

#[tracing::instrument(skip(pool, req), fields(layer = "service"))]
pub async fn update_price(pool: &PgPool, house_id: i64, req: &UpdateCalendarPriceRequest) -> Result<Vec<CalendarEntry>, AppError> {
    validate_date_range(req.from, req.to)?;
    repo::update_price(pool, house_id, req).await
}

#[tracing::instrument(skip(pool), fields(layer = "service"))]
pub async fn delete(pool: &PgPool, house_id: i64, from: NaiveDate, to: NaiveDate) -> Result<(), AppError> {
    validate_date_range(from, to)?;
    repo::delete(pool, house_id, from, to).await
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(s: &str) -> NaiveDate {
        NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap()
    }

    #[test]
    fn create_rejects_rented_status() {
        assert!(matches!(validate_create_status(CalendarStatus::Rented), Err(AppError::UnprocessableEntity(_))));
    }

    #[test]
    fn create_accepts_valid_statuses() {
        assert!(validate_create_status(CalendarStatus::Rentable).is_ok());
        assert!(validate_create_status(CalendarStatus::NotRentable).is_ok());
    }

    #[test]
    fn date_range_rejects_inverted() {
        assert!(matches!(validate_date_range(d("2024-07-10"), d("2024-07-01")), Err(AppError::UnprocessableEntity(_))));
    }

    #[test]
    fn date_range_accepts_same_day() {
        assert!(validate_date_range(d("2024-07-01"), d("2024-07-01")).is_ok());
    }

    #[test]
    fn date_range_accepts_valid_range() {
        assert!(validate_date_range(d("2024-07-01"), d("2024-07-31")).is_ok());
    }
}
