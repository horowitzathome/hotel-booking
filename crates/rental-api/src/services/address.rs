use sqlx::PgPool;

use crate::errors::AppError;
use crate::models::address::{Address, CreateAddressRequest, UpdateAddressRequest};
use crate::repositories::address as repo;

pub async fn get(pool: &PgPool, id: i64) -> Result<Address, AppError> {
    repo::find_by_id(pool, id).await
}

pub async fn create(pool: &PgPool, req: &CreateAddressRequest) -> Result<Address, AppError> {
    repo::create(pool, req).await
}

pub async fn update(pool: &PgPool, id: i64, req: &UpdateAddressRequest) -> Result<Address, AppError> {
    repo::update(pool, id, req).await
}

pub async fn delete(pool: &PgPool, id: i64) -> Result<(), AppError> {
    repo::delete(pool, id).await
}
