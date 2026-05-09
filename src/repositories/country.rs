use sqlx::PgPool;

use crate::errors::AppError;
use crate::models::country::{Country, CreateCountryRequest, UpdateCountryRequest};

pub async fn find_all(pool: &PgPool) -> Result<Vec<Country>, AppError> {
    let rows = sqlx::query_as!(Country, "SELECT id, name, iso_code FROM countries ORDER BY name").fetch_all(pool).await?;
    Ok(rows)
}

pub async fn find_by_id(pool: &PgPool, id: i64) -> Result<Country, AppError> {
    sqlx::query_as!(Country, "SELECT id, name, iso_code FROM countries WHERE id = $1", id)
        .fetch_one(pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => AppError::NotFound(format!("country {id} not found")),
            other => AppError::from(other),
        })
}

pub async fn create(pool: &PgPool, req: &CreateCountryRequest) -> Result<Country, AppError> {
    sqlx::query_as!(Country, "INSERT INTO countries (name, iso_code) VALUES ($1, $2) RETURNING id, name, iso_code", req.name, req.iso_code,)
        .fetch_one(pool)
        .await
        .map_err(AppError::from)
}

pub async fn update(pool: &PgPool, id: i64, req: &UpdateCountryRequest) -> Result<Country, AppError> {
    sqlx::query_as!(
        Country,
        "UPDATE countries SET name = $1, iso_code = $2 WHERE id = $3 RETURNING id, name, iso_code",
        req.name,
        req.iso_code,
        id,
    )
    .fetch_one(pool)
    .await
    .map_err(|e| match e {
        sqlx::Error::RowNotFound => AppError::NotFound(format!("country {id} not found")),
        other => AppError::from(other),
    })
}

pub async fn delete(pool: &PgPool, id: i64) -> Result<(), AppError> {
    let result = sqlx::query!("DELETE FROM countries WHERE id = $1", id).execute(pool).await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("country {id} not found")));
    }
    Ok(())
}
