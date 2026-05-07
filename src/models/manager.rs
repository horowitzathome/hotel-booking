use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema, sqlx::FromRow)]
pub struct Manager {
    pub id: i64,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub phone: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateManagerRequest {
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub phone: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateManagerRequest {
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub phone: String,
}
