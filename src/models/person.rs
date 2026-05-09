use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Serialize, ToSchema, sqlx::FromRow)]
pub struct Person {
    pub id: i64,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub phone: String,
}

#[derive(Debug, Deserialize, ToSchema, Validate)]
pub struct CreatePersonRequest {
    #[validate(length(min = 1, max = 100))]
    pub first_name: String,
    #[validate(length(min = 1, max = 100))]
    pub last_name: String,
    #[validate(email, length(max = 255))]
    pub email: String,
    #[validate(length(min = 1, max = 50))]
    pub phone: String,
}

#[derive(Debug, Deserialize, ToSchema, Validate)]
pub struct UpdatePersonRequest {
    #[validate(length(min = 1, max = 100))]
    pub first_name: String,
    #[validate(length(min = 1, max = 100))]
    pub last_name: String,
    #[validate(email, length(max = 255))]
    pub email: String,
    #[validate(length(min = 1, max = 50))]
    pub phone: String,
}
