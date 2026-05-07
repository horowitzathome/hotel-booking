use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "calendar_status")]
pub enum CalendarStatus {
    NotRentable,
    Rentable,
    Rented,
}

#[derive(Debug, Serialize)]
pub struct CalendarEntry {
    pub id: i64,
    pub date: NaiveDate,
    pub status: CalendarStatus,
    pub price: Decimal,
}

#[derive(Debug, Deserialize)]
pub struct CreateCalendarRequest {
    pub from: NaiveDate,
    pub to: NaiveDate,
    pub status: CalendarStatus,
    pub price: Decimal,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCalendarPriceRequest {
    pub from: NaiveDate,
    pub to: NaiveDate,
    pub price: Decimal,
}
