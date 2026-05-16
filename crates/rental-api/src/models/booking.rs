use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

use crate::models::calendar::validate_non_negative_decimal;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema, sqlx::Type)]
#[sqlx(type_name = "booking_status")]
pub enum BookingStatus {
    Active,
    Cancelled,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct BookingHouse {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct BookingPerson {
    pub id: i64,
    pub first_name: String,
    pub last_name: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct Booking {
    pub id: i64,
    pub house: BookingHouse,
    pub person: BookingPerson,
    pub from: NaiveDate,
    pub to: NaiveDate,
    pub status: BookingStatus,
    pub paid_at: Option<NaiveDate>,
    pub total_paid: Option<Decimal>,
}

/// Response for POST /bookings only: the base booking plus the computed price total.
/// `expected_total_price` is derived at booking time and not stored in the database.
#[derive(Debug, Serialize, ToSchema)]
pub struct CreateBookingResponse {
    #[serde(flatten)]
    pub booking: Booking,
    pub expected_total_price: Decimal,
}

impl std::ops::Deref for CreateBookingResponse {
    type Target = Booking;
    fn deref(&self) -> &Self::Target {
        &self.booking
    }
}

#[derive(Debug, Deserialize, ToSchema, Validate)]
pub struct CreateBookingRequest {
    #[validate(range(min = 1))]
    pub house_id: i64,
    #[validate(range(min = 1))]
    pub person_id: i64,
    pub from: NaiveDate,
    pub to: NaiveDate,
}

#[derive(Debug, Deserialize, ToSchema, Validate)]
pub struct RecordPaymentRequest {
    pub paid_at: NaiveDate,
    #[validate(custom(function = "validate_non_negative_decimal"))]
    pub total_paid: Decimal,
}
