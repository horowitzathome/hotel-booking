use sqlx::PgPool;

use crate::errors::AppError;
use crate::models::address::Address;
use crate::models::country::Country;
use crate::models::house::{CreateHouseRequest, House, UpdateHouseRequest};
use crate::models::manager::Manager;

#[tracing::instrument(skip(pool), fields(layer = "repository"))]
pub async fn find_all(pool: &PgPool) -> Result<Vec<House>, AppError> {
    let rows = sqlx::query!(
        r#"
        SELECT
            h.id          AS house_id,
            h.name        AS house_name,
            h.description AS house_description,
            a.id          AS addr_id,
            a.street      AS addr_street,
            a.number      AS addr_number,
            a.postcode    AS addr_postcode,
            a.city        AS addr_city,
            a.province    AS addr_province,
            c.id          AS country_id,
            c.name        AS country_name,
            c.iso_code    AS country_iso_code,
            m.id          AS mgr_id,
            m.first_name  AS mgr_first_name,
            m.last_name   AS mgr_last_name,
            m.email       AS mgr_email,
            m.phone       AS mgr_phone
        FROM  houses   h
        JOIN  addresses a ON a.id = h.address_id
        JOIN  countries c ON c.id = a.country_id
        JOIN  managers  m ON m.id = h.manager_id
        ORDER BY h.name
        "#
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::from)?;

    Ok(rows
        .into_iter()
        .map(|row| House {
            id: row.house_id,
            name: row.house_name,
            description: row.house_description,
            address: Address {
                id: row.addr_id,
                street: row.addr_street,
                number: row.addr_number,
                postcode: row.addr_postcode,
                city: row.addr_city,
                province: row.addr_province,
                country: Country {
                    id: row.country_id,
                    name: row.country_name,
                    iso_code: row.country_iso_code,
                },
            },
            manager: Manager {
                id: row.mgr_id,
                first_name: row.mgr_first_name,
                last_name: row.mgr_last_name,
                email: row.mgr_email,
                phone: row.mgr_phone,
            },
        })
        .collect())
}

#[tracing::instrument(skip(pool), fields(layer = "repository"))]
pub async fn find_by_id(pool: &PgPool, id: i64) -> Result<House, AppError> {
    let row = sqlx::query!(
        r#"
        SELECT
            h.id          AS house_id,
            h.name        AS house_name,
            h.description AS house_description,
            a.id          AS addr_id,
            a.street      AS addr_street,
            a.number      AS addr_number,
            a.postcode    AS addr_postcode,
            a.city        AS addr_city,
            a.province    AS addr_province,
            c.id          AS country_id,
            c.name        AS country_name,
            c.iso_code    AS country_iso_code,
            m.id          AS mgr_id,
            m.first_name  AS mgr_first_name,
            m.last_name   AS mgr_last_name,
            m.email       AS mgr_email,
            m.phone       AS mgr_phone
        FROM  houses   h
        JOIN  addresses a ON a.id = h.address_id
        JOIN  countries c ON c.id = a.country_id
        JOIN  managers  m ON m.id = h.manager_id
        WHERE h.id = $1
        "#,
        id
    )
    .fetch_one(pool)
    .await
    .map_err(|e| match e {
        sqlx::Error::RowNotFound => AppError::NotFound(format!("house {id} not found")),
        other => AppError::from(other),
    })?;

    Ok(House {
        id: row.house_id,
        name: row.house_name,
        description: row.house_description,
        address: Address {
            id: row.addr_id,
            street: row.addr_street,
            number: row.addr_number,
            postcode: row.addr_postcode,
            city: row.addr_city,
            province: row.addr_province,
            country: Country {
                id: row.country_id,
                name: row.country_name,
                iso_code: row.country_iso_code,
            },
        },
        manager: Manager {
            id: row.mgr_id,
            first_name: row.mgr_first_name,
            last_name: row.mgr_last_name,
            email: row.mgr_email,
            phone: row.mgr_phone,
        },
    })
}

#[tracing::instrument(skip(pool, req), fields(layer = "repository"))]
pub async fn create(pool: &PgPool, req: &CreateHouseRequest) -> Result<House, AppError> {
    let id: i64 = sqlx::query_scalar!(
        "INSERT INTO houses (name, description, address_id, manager_id) \
         VALUES ($1, $2, $3, $4) RETURNING id",
        req.name,
        req.description,
        req.address_id,
        req.manager_id,
    )
    .fetch_one(pool)
    .await
    .map_err(AppError::from)?;

    find_by_id(pool, id).await
}

#[tracing::instrument(skip(pool, req), fields(layer = "repository"))]
pub async fn update(pool: &PgPool, id: i64, req: &UpdateHouseRequest) -> Result<House, AppError> {
    let rows = sqlx::query!(
        "UPDATE houses \
         SET name = $1, description = $2, address_id = $3, manager_id = $4 \
         WHERE id = $5",
        req.name,
        req.description,
        req.address_id,
        req.manager_id,
        id,
    )
    .execute(pool)
    .await
    .map_err(AppError::from)?;

    if rows.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("house {id} not found")));
    }

    find_by_id(pool, id).await
}

#[tracing::instrument(skip(pool), fields(layer = "repository"))]
pub async fn delete(pool: &PgPool, id: i64) -> Result<(), AppError> {
    let result = sqlx::query!("DELETE FROM houses WHERE id = $1", id).execute(pool).await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("house {id} not found")));
    }
    Ok(())
}
