use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema, sqlx::FromRow)]
pub struct Person {
    pub id: i64,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub phone: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreatePersonRequest {
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub phone: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdatePersonRequest {
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub phone: String,
}
