use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

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

#[derive(Debug, Deserialize, ToSchema, Validate)]
pub struct CreateAddressRequest {
    #[validate(length(min = 1, max = 255))]
    pub street: String,
    #[validate(length(min = 1, max = 20))]
    pub number: String,
    #[validate(length(min = 1, max = 20))]
    pub postcode: String,
    #[validate(length(min = 1, max = 100))]
    pub city: String,
    #[validate(length(min = 1, max = 100))]
    pub province: Option<String>,
    #[validate(range(min = 1))]
    pub country_id: i64,
}

#[derive(Debug, Deserialize, ToSchema, Validate)]
pub struct UpdateAddressRequest {
    #[validate(length(min = 1, max = 255))]
    pub street: String,
    #[validate(length(min = 1, max = 20))]
    pub number: String,
    #[validate(length(min = 1, max = 20))]
    pub postcode: String,
    #[validate(length(min = 1, max = 100))]
    pub city: String,
    #[validate(length(min = 1, max = 100))]
    pub province: Option<String>,
    #[validate(range(min = 1))]
    pub country_id: i64,
}
