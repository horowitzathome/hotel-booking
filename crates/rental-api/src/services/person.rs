use sqlx::PgPool;

use crate::errors::AppError;
use crate::models::person::{CreatePersonRequest, Person, UpdatePersonRequest};
use crate::repositories::person as repo;

pub async fn list(pool: &PgPool) -> Result<Vec<Person>, AppError> {
    repo::find_all(pool).await
}

pub async fn get(pool: &PgPool, id: i64) -> Result<Person, AppError> {
    repo::find_by_id(pool, id).await
}

pub async fn create(pool: &PgPool, req: &CreatePersonRequest) -> Result<Person, AppError> {
    repo::create(pool, req).await
}

pub async fn update(pool: &PgPool, id: i64, req: &UpdatePersonRequest) -> Result<Person, AppError> {
    repo::update(pool, id, req).await
}

pub async fn delete(pool: &PgPool, id: i64) -> Result<(), AppError> {
    repo::delete(pool, id).await
}
