use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct BookingHouse {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct BookingPerson {
    pub id: i64,
    pub first_name: String,
    pub last_name: String,
}

#[derive(Debug, Serialize)]
pub struct Booking {
    pub id: i64,
    pub house: BookingHouse,
    pub person: BookingPerson,
    pub from: NaiveDate,
    pub to: NaiveDate,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_total_price: Option<Decimal>,
    pub paid_at: Option<NaiveDate>,
    pub total_paid: Option<Decimal>,
}

#[derive(Debug, Deserialize)]
pub struct CreateBookingRequest {
    pub house_id: i64,
    pub person_id: i64,
    pub from: NaiveDate,
    pub to: NaiveDate,
}

#[derive(Debug, Deserialize)]
pub struct RecordPaymentRequest {
    pub paid_at: NaiveDate,
    pub total_paid: Decimal,
}
