use chrono::NaiveDate;
use sqlx::PgPool;

use crate::errors::AppError;
use crate::models::calendar::{CalendarEntry, CreateCalendarRequest, UpdateCalendarPriceRequest};
use crate::repositories::calendar as repo;

pub async fn list(
    pool: &PgPool,
    house_id: i64,
    from: Option<NaiveDate>,
    to: Option<NaiveDate>,
) -> Result<Vec<CalendarEntry>, AppError> {
    repo::find_all(pool, house_id, from, to).await
}

pub async fn get(pool: &PgPool, house_id: i64, id: i64) -> Result<CalendarEntry, AppError> {
    repo::find_by_id(pool, house_id, id).await
}

pub async fn create(
    pool: &PgPool,
    house_id: i64,
    req: &CreateCalendarRequest,
) -> Result<Vec<CalendarEntry>, AppError> {
    if req.status == "Rented" {
        return Err(AppError::UnprocessableEntity(
            "status 'Rented' cannot be set via calendar endpoints".into(),
        ));
    }
    if !matches!(req.status.as_str(), "NotRentable" | "Rentable") {
        return Err(AppError::UnprocessableEntity(format!(
            "invalid status '{}'; must be 'NotRentable' or 'Rentable'",
            req.status
        )));
    }
    if req.from > req.to {
        return Err(AppError::UnprocessableEntity(
            "'from' must not be after 'to'".into(),
        ));
    }
    repo::create(pool, house_id, req).await
}

pub async fn update_price(
    pool: &PgPool,
    house_id: i64,
    req: &UpdateCalendarPriceRequest,
) -> Result<Vec<CalendarEntry>, AppError> {
    if req.from > req.to {
        return Err(AppError::UnprocessableEntity(
            "'from' must not be after 'to'".into(),
        ));
    }
    repo::update_price(pool, house_id, req).await
}

pub async fn delete(
    pool: &PgPool,
    house_id: i64,
    from: NaiveDate,
    to: NaiveDate,
) -> Result<(), AppError> {
    if from > to {
        return Err(AppError::UnprocessableEntity(
            "'from' must not be after 'to'".into(),
        ));
    }
    repo::delete(pool, house_id, from, to).await
}
