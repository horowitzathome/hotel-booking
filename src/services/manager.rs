use sqlx::PgPool;

use crate::errors::AppError;
use crate::models::manager::{CreateManagerRequest, Manager, UpdateManagerRequest};
use crate::repositories::manager as repo;

pub async fn list(pool: &PgPool) -> Result<Vec<Manager>, AppError> {
    repo::find_all(pool).await
}

pub async fn get(pool: &PgPool, id: i64) -> Result<Manager, AppError> {
    repo::find_by_id(pool, id).await
}

pub async fn create(pool: &PgPool, req: &CreateManagerRequest) -> Result<Manager, AppError> {
    repo::create(pool, req).await
}

pub async fn update(pool: &PgPool, id: i64, req: &UpdateManagerRequest) -> Result<Manager, AppError> {
    repo::update(pool, id, req).await
}

pub async fn delete(pool: &PgPool, id: i64) -> Result<(), AppError> {
    repo::delete(pool, id).await
}
