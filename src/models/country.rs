use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema, sqlx::FromRow)]
pub struct Country {
    pub id: i64,
    pub name: String,
    pub iso_code: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateCountryRequest {
    pub name: String,
    pub iso_code: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateCountryRequest {
    pub name: String,
    pub iso_code: String,
}
