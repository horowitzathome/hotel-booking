use sqlx::PgPool;

use crate::errors::AppError;
use crate::models::manager::{CreateManagerRequest, Manager, UpdateManagerRequest};
use crate::repositories::manager as repo;

#[tracing::instrument(skip(pool), fields(layer = "service"))]
pub async fn list(pool: &PgPool, limit: Option<i64>, offset: Option<i64>) -> Result<Vec<Manager>, AppError> {
    repo::find_all(pool, limit, offset).await
}

#[tracing::instrument(skip(pool), fields(layer = "service"))]
pub async fn get(pool: &PgPool, id: i64) -> Result<Manager, AppError> {
    repo::find_by_id(pool, id).await
}

#[tracing::instrument(skip(pool, req), fields(layer = "service"))]
pub async fn create(pool: &PgPool, req: &CreateManagerRequest) -> Result<Manager, AppError> {
    repo::create(pool, req).await
}

#[tracing::instrument(skip(pool, req), fields(layer = "service"))]
pub async fn update(pool: &PgPool, id: i64, req: &UpdateManagerRequest) -> Result<Manager, AppError> {
    repo::update(pool, id, req).await
}

#[tracing::instrument(skip(pool), fields(layer = "service"))]
pub async fn delete(pool: &PgPool, id: i64) -> Result<(), AppError> {
    repo::delete(pool, id).await
}
