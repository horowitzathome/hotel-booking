use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Serialize, ToSchema, sqlx::FromRow)]
pub struct Country {
    pub id: i64,
    pub name: String,
    pub iso_code: String,
}

#[derive(Debug, Deserialize, ToSchema, Validate)]
pub struct CreateCountryRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: String,
    #[validate(length(equal = 2))]
    pub iso_code: String,
}

#[derive(Debug, Deserialize, ToSchema, Validate)]
pub struct UpdateCountryRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: String,
    #[validate(length(equal = 2))]
    pub iso_code: String,
}
