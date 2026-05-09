use sqlx::PgPool;

use crate::errors::AppError;
use crate::models::country::{Country, CreateCountryRequest, UpdateCountryRequest};
use crate::repositories::country as repo;

pub async fn list(pool: &PgPool) -> Result<Vec<Country>, AppError> {
    repo::find_all(pool).await
}

pub async fn get(pool: &PgPool, id: i64) -> Result<Country, AppError> {
    repo::find_by_id(pool, id).await
}

pub async fn create(pool: &PgPool, req: &CreateCountryRequest) -> Result<Country, AppError> {
    repo::create(pool, req).await
}

pub async fn update(pool: &PgPool, id: i64, req: &UpdateCountryRequest) -> Result<Country, AppError> {
    repo::update(pool, id, req).await
}

pub async fn delete(pool: &PgPool, id: i64) -> Result<(), AppError> {
    repo::delete(pool, id).await
}
