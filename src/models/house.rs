use serde::{Deserialize, Serialize};

use crate::models::address::Address;
use crate::models::manager::Manager;

#[derive(Debug, Serialize)]
pub struct House {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub address: Address,
    pub manager: Manager,
}

#[derive(Debug, Deserialize)]
pub struct CreateHouseRequest {
    pub name: String,
    pub description: String,
    pub address_id: i64,
    pub manager_id: i64,
}

#[derive(Debug, Deserialize)]
pub struct UpdateHouseRequest {
    pub name: String,
    pub description: String,
    pub address_id: i64,
    pub manager_id: i64,
}
