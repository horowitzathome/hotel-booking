use sqlx::PgPool;

use crate::errors::AppError;
use crate::models::address::{Address, CreateAddressRequest, UpdateAddressRequest};
use crate::models::country::Country;

pub async fn find_by_id(pool: &PgPool, id: i64) -> Result<Address, AppError> {
    let row = sqlx::query!(
        r#"
        SELECT a.id, a.street, a.number, a.postcode, a.city, a.province,
               c.id    AS country_id,
               c.name  AS country_name,
               c.iso_code AS country_iso_code
        FROM   addresses a
        JOIN   countries c ON c.id = a.country_id
        WHERE  a.id = $1
        "#,
        id
    )
    .fetch_one(pool)
    .await
    .map_err(|e| match e {
        sqlx::Error::RowNotFound => AppError::NotFound(format!("address {id} not found")),
        other => AppError::from(other),
    })?;

    Ok(Address {
        id: row.id,
        street: row.street,
        number: row.number,
        postcode: row.postcode,
        city: row.city,
        province: row.province,
        country: Country {
            id: row.country_id,
            name: row.country_name,
            iso_code: row.country_iso_code,
        },
    })
}

pub async fn create(pool: &PgPool, req: &CreateAddressRequest) -> Result<Address, AppError> {
    let id: i64 = sqlx::query_scalar!(
        "INSERT INTO addresses (street, number, postcode, city, province, country_id) \
         VALUES ($1, $2, $3, $4, $5, $6) RETURNING id",
        req.street,
        req.number,
        req.postcode,
        req.city,
        req.province,
        req.country_id,
    )
    .fetch_one(pool)
    .await
    .map_err(AppError::from)?;

    find_by_id(pool, id).await
}

pub async fn update(pool: &PgPool, id: i64, req: &UpdateAddressRequest) -> Result<Address, AppError> {
    let rows = sqlx::query!(
        "UPDATE addresses \
         SET street = $1, number = $2, postcode = $3, city = $4, province = $5, country_id = $6 \
         WHERE id = $7",
        req.street,
        req.number,
        req.postcode,
        req.city,
        req.province,
        req.country_id,
        id,
    )
    .execute(pool)
    .await
    .map_err(AppError::from)?;

    if rows.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("address {id} not found")));
    }

    find_by_id(pool, id).await
}

pub async fn delete(pool: &PgPool, id: i64) -> Result<(), AppError> {
    let result = sqlx::query!("DELETE FROM addresses WHERE id = $1", id).execute(pool).await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("address {id} not found")));
    }
    Ok(())
}
