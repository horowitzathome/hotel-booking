use sqlx::PgPool;

use crate::errors::AppError;
use crate::models::house::{CreateHouseRequest, House, UpdateHouseRequest};
use crate::repositories::house as repo;

pub async fn list(pool: &PgPool) -> Result<Vec<House>, AppError> {
    repo::find_all(pool).await
}

pub async fn get(pool: &PgPool, id: i64) -> Result<House, AppError> {
    repo::find_by_id(pool, id).await
}

pub async fn create(pool: &PgPool, req: &CreateHouseRequest) -> Result<House, AppError> {
    repo::create(pool, req).await
}

pub async fn update(pool: &PgPool, id: i64, req: &UpdateHouseRequest) -> Result<House, AppError> {
    repo::update(pool, id, req).await
}

pub async fn delete(pool: &PgPool, id: i64) -> Result<(), AppError> {
    repo::delete(pool, id).await
}
