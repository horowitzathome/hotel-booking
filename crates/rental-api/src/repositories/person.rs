use sqlx::PgPool;

use crate::errors::AppError;
use crate::models::person::{CreatePersonRequest, Person, UpdatePersonRequest};

#[tracing::instrument(skip(pool), fields(layer = "repository"))]
pub async fn find_all(pool: &PgPool) -> Result<Vec<Person>, AppError> {
    let rows = sqlx::query_as!(Person, "SELECT id, first_name, last_name, email, phone FROM persons ORDER BY last_name, first_name")
        .fetch_all(pool)
        .await?;
    Ok(rows)
}

#[tracing::instrument(skip(pool), fields(layer = "repository"))]
pub async fn find_by_id(pool: &PgPool, id: i64) -> Result<Person, AppError> {
    sqlx::query_as!(Person, "SELECT id, first_name, last_name, email, phone FROM persons WHERE id = $1", id)
        .fetch_one(pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => AppError::NotFound(format!("person {id} not found")),
            other => AppError::from(other),
        })
}

#[tracing::instrument(skip(pool, req), fields(layer = "repository"))]
pub async fn create(pool: &PgPool, req: &CreatePersonRequest) -> Result<Person, AppError> {
    sqlx::query_as!(
        Person,
        "INSERT INTO persons (first_name, last_name, email, phone)
         VALUES ($1, $2, $3, $4)
         RETURNING id, first_name, last_name, email, phone",
        req.first_name,
        req.last_name,
        req.email,
        req.phone,
    )
    .fetch_one(pool)
    .await
    .map_err(AppError::from)
}

#[tracing::instrument(skip(pool, req), fields(layer = "repository"))]
pub async fn update(pool: &PgPool, id: i64, req: &UpdatePersonRequest) -> Result<Person, AppError> {
    sqlx::query_as!(
        Person,
        "UPDATE persons
         SET first_name = $1, last_name = $2, email = $3, phone = $4
         WHERE id = $5
         RETURNING id, first_name, last_name, email, phone",
        req.first_name,
        req.last_name,
        req.email,
        req.phone,
        id,
    )
    .fetch_one(pool)
    .await
    .map_err(|e| match e {
        sqlx::Error::RowNotFound => AppError::NotFound(format!("person {id} not found")),
        other => AppError::from(other),
    })
}

#[tracing::instrument(skip(pool), fields(layer = "repository"))]
pub async fn delete(pool: &PgPool, id: i64) -> Result<(), AppError> {
    let result = sqlx::query!("DELETE FROM persons WHERE id = $1", id).execute(pool).await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("person {id} not found")));
    }
    Ok(())
}
