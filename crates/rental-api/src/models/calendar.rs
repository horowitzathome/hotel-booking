use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::{Validate, ValidationError};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema, sqlx::Type)]
#[sqlx(type_name = "calendar_status")]
pub enum CalendarStatus {
    NotRentable,
    Rentable,
    Rented,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CalendarEntry {
    pub id: i64,
    pub date: NaiveDate,
    pub status: CalendarStatus,
    pub price: Decimal,
}

#[derive(Debug, Deserialize, ToSchema, Validate)]
pub struct CreateCalendarRequest {
    pub from: NaiveDate,
    pub to: NaiveDate,
    pub status: CalendarStatus,
    #[validate(custom(function = "validate_non_negative_decimal"))]
    pub price: Decimal,
}

#[derive(Debug, Deserialize, ToSchema, Validate)]
pub struct UpdateCalendarPriceRequest {
    pub from: NaiveDate,
    pub to: NaiveDate,
    #[validate(custom(function = "validate_non_negative_decimal"))]
    pub price: Decimal,
}

pub(crate) fn validate_non_negative_decimal(value: &Decimal) -> Result<(), ValidationError> {
    if value.is_sign_negative() {
        return Err(ValidationError::new("must_be_non_negative"));
    }
    Ok(())
}
