use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::models::country::Country;

#[derive(Debug, Serialize, ToSchema)]
pub struct Address {
    pub id: i64,
    pub street: String,
    pub number: String,
    pub postcode: String,
    pub city: String,
    pub province: Option<String>,
    pub country: Country,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateAddressRequest {
    pub street: String,
    pub number: String,
    pub postcode: String,
    pub city: String,
    pub province: Option<String>,
    pub country_id: i64,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateAddressRequest {
    pub street: String,
    pub number: String,
    pub postcode: String,
    pub city: String,
    pub province: Option<String>,
    pub country_id: i64,
}
