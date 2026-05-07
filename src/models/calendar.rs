use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

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

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateCalendarRequest {
    pub from: NaiveDate,
    pub to: NaiveDate,
    pub status: CalendarStatus,
    pub price: Decimal,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateCalendarPriceRequest {
    pub from: NaiveDate,
    pub to: NaiveDate,
    pub price: Decimal,
}
