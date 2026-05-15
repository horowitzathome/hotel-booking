use sqlx::PgPool;

use crate::errors::AppError;
use crate::models::address::{Address, CreateAddressRequest, UpdateAddressRequest};
use crate::repositories::address as repo;

#[tracing::instrument(skip(pool), fields(layer = "service"))]
pub async fn get(pool: &PgPool, id: i64) -> Result<Address, AppError> {
    repo::find_by_id(pool, id).await
}

#[tracing::instrument(skip(pool, req), fields(layer = "service"))]
pub async fn create(pool: &PgPool, req: &CreateAddressRequest) -> Result<Address, AppError> {
    repo::create(pool, req).await
}

#[tracing::instrument(skip(pool, req), fields(layer = "service"))]
pub async fn update(pool: &PgPool, id: i64, req: &UpdateAddressRequest) -> Result<Address, AppError> {
    repo::update(pool, id, req).await
}

#[tracing::instrument(skip(pool), fields(layer = "service"))]
pub async fn delete(pool: &PgPool, id: i64) -> Result<(), AppError> {
    repo::delete(pool, id).await
}
