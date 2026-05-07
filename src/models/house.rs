use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

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

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateHouseRequest {
    pub name: String,
    pub description: String,
    pub address_id: i64,
    pub manager_id: i64,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateHouseRequest {
    pub name: String,
    pub description: String,
    pub address_id: i64,
    pub manager_id: i64,
}
