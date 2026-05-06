use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Country {
    pub id: i64,
    pub name: String,
    pub iso_code: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateCountryRequest {
    pub name: String,
    pub iso_code: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCountryRequest {
    pub name: String,
    pub iso_code: String,
}
