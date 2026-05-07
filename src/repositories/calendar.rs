use chrono::{Duration, NaiveDate};
use sqlx::PgPool;

use crate::errors::AppError;
use crate::models::calendar::{CalendarEntry, CalendarStatus, CreateCalendarRequest, UpdateCalendarPriceRequest};

pub async fn find_all(
    pool: &PgPool,
    house_id: i64,
    from: Option<NaiveDate>,
    to: Option<NaiveDate>,
) -> Result<Vec<CalendarEntry>, AppError> {
    let rows = sqlx::query!(
        r#"
        SELECT id, date, status AS "status: CalendarStatus", price
        FROM calendar
        WHERE house_id = $1
          AND ($2::date IS NULL OR date >= $2)
          AND ($3::date IS NULL OR date <= $3)
        ORDER BY date
        "#,
        house_id,
        from as Option<NaiveDate>,
        to as Option<NaiveDate>,
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::from)?;

    Ok(rows
        .into_iter()
        .map(|r| CalendarEntry {
            id: r.id,
            date: r.date,
            status: r.status,
            price: r.price,
        })
        .collect())
}

pub async fn find_by_id(pool: &PgPool, house_id: i64, id: i64) -> Result<CalendarEntry, AppError> {
    let row = sqlx::query!(
        r#"SELECT id, date, status AS "status: CalendarStatus", price FROM calendar WHERE id = $1 AND house_id = $2"#,
        id,
        house_id,
    )
    .fetch_one(pool)
    .await
    .map_err(|e| match e {
        sqlx::Error::RowNotFound => {
            AppError::NotFound(format!("calendar entry {id} not found for house {house_id}"))
        }
        other => AppError::from(other),
    })?;

    Ok(CalendarEntry {
        id: row.id,
        date: row.date,
        status: row.status,
        price: row.price,
    })
}

pub async fn create(
    pool: &PgPool,
    house_id: i64,
    req: &CreateCalendarRequest,
) -> Result<Vec<CalendarEntry>, AppError> {
    let mut tx = pool.begin().await.map_err(AppError::from)?;
    let mut created = Vec::new();

    let mut date = req.from;
    while date <= req.to {
        let row = sqlx::query!(
            r#"
            INSERT INTO calendar (house_id, date, status, price)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (house_id, date) DO NOTHING
            RETURNING id, date, status AS "status: CalendarStatus", price
            "#,
            house_id,
            date,
            req.status as CalendarStatus,
            req.price,
        )
        .fetch_optional(&mut *tx)
        .await
        .map_err(AppError::from)?;

        if let Some(r) = row {
            created.push(CalendarEntry {
                id: r.id,
                date: r.date,
                status: r.status,
                price: r.price,
            });
        }

        date += Duration::days(1);
    }

    tx.commit().await.map_err(AppError::from)?;
    Ok(created)
}

pub async fn update_price(
    pool: &PgPool,
    house_id: i64,
    req: &UpdateCalendarPriceRequest,
) -> Result<Vec<CalendarEntry>, AppError> {
    let mut rows = sqlx::query!(
        r#"
        UPDATE calendar
        SET price = $1
        WHERE house_id = $2 AND date BETWEEN $3 AND $4
        RETURNING id, date, status AS "status: CalendarStatus", price
        "#,
        req.price,
        house_id,
        req.from,
        req.to,
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::from)?;

    rows.sort_by_key(|r| r.date);

    Ok(rows
        .into_iter()
        .map(|r| CalendarEntry {
            id: r.id,
            date: r.date,
            status: r.status,
            price: r.price,
        })
        .collect())
}

pub async fn delete(
    pool: &PgPool,
    house_id: i64,
    from: NaiveDate,
    to: NaiveDate,
) -> Result<(), AppError> {
    // Check if any entry in the range belongs to a non-cancelled booking
    let count = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*)
        FROM calendar c
        JOIN bookings b       ON b.house_id         = c.house_id
        JOIN calendar c_from  ON c_from.id          = b.from_calendar_id
        JOIN calendar c_to    ON c_to.id            = b.to_calendar_id
        WHERE c.house_id = $1
          AND c.date BETWEEN $2 AND $3
          AND b.status != 'Cancelled'
          AND c.date BETWEEN c_from.date AND c_to.date
        "#,
        house_id,
        from,
        to,
    )
    .fetch_one(pool)
    .await
    .map_err(AppError::from)?;

    if count.unwrap_or(0) > 0 {
        return Err(AppError::Conflict(
            "one or more calendar entries are referenced by active bookings".into(),
        ));
    }

    sqlx::query!(
        "DELETE FROM calendar WHERE house_id = $1 AND date BETWEEN $2 AND $3",
        house_id,
        from,
        to,
    )
    .execute(pool)
    .await
    .map_err(AppError::from)?;

    Ok(())
}
