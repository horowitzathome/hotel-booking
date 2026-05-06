use rust_decimal::Decimal;
use sqlx::PgPool;

use crate::errors::AppError;
use crate::models::booking::{Booking, BookingHouse, BookingPerson, CreateBookingRequest, RecordPaymentRequest};

pub async fn find_all(
    pool: &PgPool,
    house_id: Option<i64>,
    person_id: Option<i64>,
) -> Result<Vec<Booking>, AppError> {
    let rows = sqlx::query!(
        r#"
        SELECT
            b.id,
            b.house_id,
            h.name           AS house_name,
            b.person_id,
            p.first_name     AS person_first_name,
            p.last_name      AS person_last_name,
            c_from.date      AS from_date,
            c_to.date        AS to_date,
            b.status,
            b.paid_at,
            b.total_paid
        FROM bookings b
        JOIN houses h        ON h.id      = b.house_id
        JOIN persons p       ON p.id      = b.person_id
        JOIN calendar c_from ON c_from.id = b.from_calendar_id
        JOIN calendar c_to   ON c_to.id   = b.to_calendar_id
        WHERE ($1::bigint IS NULL OR b.house_id = $1)
          AND ($2::bigint IS NULL OR b.person_id = $2)
        ORDER BY b.id
        "#,
        house_id,
        person_id,
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::from)?;

    Ok(rows
        .into_iter()
        .map(|r| Booking {
            id: r.id,
            house: BookingHouse { id: r.house_id, name: r.house_name },
            person: BookingPerson {
                id: r.person_id,
                first_name: r.person_first_name,
                last_name: r.person_last_name,
            },
            from: r.from_date,
            to: r.to_date,
            status: r.status,
            expected_total_price: None,
            paid_at: r.paid_at,
            total_paid: r.total_paid,
        })
        .collect())
}

pub async fn find_by_id(pool: &PgPool, id: i64) -> Result<Booking, AppError> {
    let r = sqlx::query!(
        r#"
        SELECT
            b.id,
            b.house_id,
            h.name           AS house_name,
            b.person_id,
            p.first_name     AS person_first_name,
            p.last_name      AS person_last_name,
            c_from.date      AS from_date,
            c_to.date        AS to_date,
            b.status,
            b.paid_at,
            b.total_paid
        FROM bookings b
        JOIN houses h        ON h.id      = b.house_id
        JOIN persons p       ON p.id      = b.person_id
        JOIN calendar c_from ON c_from.id = b.from_calendar_id
        JOIN calendar c_to   ON c_to.id   = b.to_calendar_id
        WHERE b.id = $1
        "#,
        id,
    )
    .fetch_one(pool)
    .await
    .map_err(|e| match e {
        sqlx::Error::RowNotFound => AppError::NotFound(format!("booking {id} not found")),
        other => AppError::from(other),
    })?;

    Ok(Booking {
        id: r.id,
        house: BookingHouse { id: r.house_id, name: r.house_name },
        person: BookingPerson {
            id: r.person_id,
            first_name: r.person_first_name,
            last_name: r.person_last_name,
        },
        from: r.from_date,
        to: r.to_date,
        status: r.status,
        expected_total_price: None,
        paid_at: r.paid_at,
        total_paid: r.total_paid,
    })
}

pub async fn create(
    pool: &PgPool,
    req: &CreateBookingRequest,
) -> Result<(Booking, Decimal), AppError> {
    let mut tx = pool.begin().await.map_err(AppError::from)?;

    let entries = sqlx::query!(
        r#"
        SELECT id, date, status, price
        FROM calendar
        WHERE house_id = $1 AND date BETWEEN $2 AND $3
        ORDER BY date
        FOR UPDATE
        "#,
        req.house_id,
        req.from,
        req.to,
    )
    .fetch_all(&mut *tx)
    .await
    .map_err(AppError::from)?;

    let expected_days = (req.to - req.from).num_days() + 1;
    if entries.len() as i64 != expected_days {
        return Err(AppError::UnprocessableEntity(
            "not all days in the requested range have calendar entries".into(),
        ));
    }

    if entries.iter().any(|e| e.status != "Rentable") {
        return Err(AppError::UnprocessableEntity(
            "all days in range must be in status 'Rentable'".into(),
        ));
    }

    let from_calendar_id = entries.first().unwrap().id;
    let to_calendar_id = entries.last().unwrap().id;
    let expected_total_price: Decimal = entries.iter().map(|e| e.price).sum();

    let booking_id: i64 = sqlx::query_scalar!(
        r#"
        INSERT INTO bookings (house_id, person_id, from_calendar_id, to_calendar_id, status)
        VALUES ($1, $2, $3, $4, 'Active')
        RETURNING id
        "#,
        req.house_id,
        req.person_id,
        from_calendar_id,
        to_calendar_id,
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(AppError::from)?;

    sqlx::query!(
        "UPDATE calendar SET status = 'Rented' WHERE house_id = $1 AND date BETWEEN $2 AND $3",
        req.house_id,
        req.from,
        req.to,
    )
    .execute(&mut *tx)
    .await
    .map_err(AppError::from)?;

    tx.commit().await.map_err(AppError::from)?;

    let booking = find_by_id(pool, booking_id).await?;
    Ok((booking, expected_total_price))
}

pub async fn cancel(pool: &PgPool, id: i64) -> Result<Booking, AppError> {
    let mut tx = pool.begin().await.map_err(AppError::from)?;

    let row = sqlx::query!(
        r#"
        SELECT b.status, b.house_id, c_from.date AS from_date, c_to.date AS to_date
        FROM bookings b
        JOIN calendar c_from ON c_from.id = b.from_calendar_id
        JOIN calendar c_to   ON c_to.id   = b.to_calendar_id
        WHERE b.id = $1
        FOR UPDATE OF b
        "#,
        id,
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| match e {
        sqlx::Error::RowNotFound => AppError::NotFound(format!("booking {id} not found")),
        other => AppError::from(other),
    })?;

    if row.status == "Cancelled" {
        return Err(AppError::UnprocessableEntity(
            "booking is already cancelled".into(),
        ));
    }

    sqlx::query!(
        "UPDATE bookings SET status = 'Cancelled', paid_at = NULL, total_paid = NULL WHERE id = $1",
        id,
    )
    .execute(&mut *tx)
    .await
    .map_err(AppError::from)?;

    sqlx::query!(
        "UPDATE calendar SET status = 'Rentable' WHERE house_id = $1 AND date BETWEEN $2 AND $3",
        row.house_id,
        row.from_date,
        row.to_date,
    )
    .execute(&mut *tx)
    .await
    .map_err(AppError::from)?;

    tx.commit().await.map_err(AppError::from)?;

    find_by_id(pool, id).await
}

pub async fn record_payment(
    pool: &PgPool,
    id: i64,
    req: &RecordPaymentRequest,
) -> Result<Booking, AppError> {
    let status = sqlx::query_scalar!(
        "SELECT status FROM bookings WHERE id = $1",
        id,
    )
    .fetch_one(pool)
    .await
    .map_err(|e| match e {
        sqlx::Error::RowNotFound => AppError::NotFound(format!("booking {id} not found")),
        other => AppError::from(other),
    })?;

    if status == "Cancelled" {
        return Err(AppError::UnprocessableEntity(
            "cannot record payment for a cancelled booking".into(),
        ));
    }

    sqlx::query!(
        "UPDATE bookings SET paid_at = $1, total_paid = $2 WHERE id = $3",
        req.paid_at,
        req.total_paid,
        id,
    )
    .execute(pool)
    .await
    .map_err(AppError::from)?;

    find_by_id(pool, id).await
}
