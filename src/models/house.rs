use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

use crate::models::address::Address;
use crate::models::manager::Manager;

#[derive(Debug, Serialize, ToSchema)]
pub struct House {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub address: Address,
    pub manager: Manager,
}

#[derive(Debug, Deserialize, ToSchema, Validate)]
pub struct CreateHouseRequest {
    #[validate(length(min = 1, max = 255))]
    pub name: String,
    #[validate(length(max = 2000))]
    pub description: String,
    #[validate(range(min = 1))]
    pub address_id: i64,
    #[validate(range(min = 1))]
    pub manager_id: i64,
}

#[derive(Debug, Deserialize, ToSchema, Validate)]
pub struct UpdateHouseRequest {
    #[validate(length(min = 1, max = 255))]
    pub name: String,
    #[validate(length(max = 2000))]
    pub description: String,
    #[validate(range(min = 1))]
    pub address_id: i64,
    #[validate(range(min = 1))]
    pub manager_id: i64,
}
