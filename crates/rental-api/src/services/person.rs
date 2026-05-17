use sqlx::PgPool;

use crate::errors::AppError;
use crate::models::person::{CreatePersonRequest, Person, UpdatePersonRequest};
use crate::repositories::person as repo;

#[tracing::instrument(skip(pool), fields(layer = "service"))]
pub async fn list(pool: &PgPool, limit: Option<i64>, offset: Option<i64>) -> Result<Vec<Person>, AppError> {
    repo::find_all(pool, limit, offset).await
}

#[tracing::instrument(skip(pool), fields(layer = "service"))]
pub async fn get(pool: &PgPool, id: i64) -> Result<Person, AppError> {
    repo::find_by_id(pool, id).await
}

#[tracing::instrument(skip(pool, req), fields(layer = "service"))]
pub async fn create(pool: &PgPool, req: &CreatePersonRequest) -> Result<Person, AppError> {
    repo::create(pool, req).await
}

#[tracing::instrument(skip(pool, req), fields(layer = "service"))]
pub async fn update(pool: &PgPool, id: i64, req: &UpdatePersonRequest) -> Result<Person, AppError> {
    repo::update(pool, id, req).await
}

#[tracing::instrument(skip(pool), fields(layer = "service"))]
pub async fn delete(pool: &PgPool, id: i64) -> Result<(), AppError> {
    repo::delete(pool, id).await
}
