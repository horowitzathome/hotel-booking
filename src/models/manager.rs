use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Manager {
    pub id: i64,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub phone: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateManagerRequest {
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub phone: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateManagerRequest {
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub phone: String,
}
