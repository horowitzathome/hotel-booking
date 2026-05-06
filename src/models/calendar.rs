use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct CalendarEntry {
    pub id: i64,
    pub date: NaiveDate,
    pub status: String,
    pub price: Decimal,
}

#[derive(Debug, Deserialize)]
pub struct CreateCalendarRequest {
    pub from: NaiveDate,
    pub to: NaiveDate,
    pub status: String,
    pub price: Decimal,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCalendarPriceRequest {
    pub from: NaiveDate,
    pub to: NaiveDate,
    pub price: Decimal,
}
