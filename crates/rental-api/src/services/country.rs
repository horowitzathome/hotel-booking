use sqlx::PgPool;

use crate::errors::AppError;
use crate::models::country::{Country, CreateCountryRequest, UpdateCountryRequest};
use crate::repositories::country as repo;

#[tracing::instrument(skip(pool), fields(layer = "service"))]
pub async fn list(pool: &PgPool, limit: Option<i64>, offset: Option<i64>) -> Result<Vec<Country>, AppError> {
    repo::find_all(pool, limit, offset).await
}

#[tracing::instrument(skip(pool), fields(layer = "service"))]
pub async fn get(pool: &PgPool, id: i64) -> Result<Country, AppError> {
    repo::find_by_id(pool, id).await
}

#[tracing::instrument(skip(pool, req), fields(layer = "service"))]
pub async fn create(pool: &PgPool, req: &CreateCountryRequest) -> Result<Country, AppError> {
    repo::create(pool, req).await
}

#[tracing::instrument(skip(pool, req), fields(layer = "service"))]
pub async fn update(pool: &PgPool, id: i64, req: &UpdateCountryRequest) -> Result<Country, AppError> {
    repo::update(pool, id, req).await
}

#[tracing::instrument(skip(pool), fields(layer = "service"))]
pub async fn delete(pool: &PgPool, id: i64) -> Result<(), AppError> {
    repo::delete(pool, id).await
}
