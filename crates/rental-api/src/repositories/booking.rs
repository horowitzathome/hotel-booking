use chrono::NaiveDate;
use rust_decimal::Decimal;
use sqlx::{PgPool, Postgres, Transaction};

use crate::errors::AppError;
use crate::models::booking::{Booking, BookingHouse, BookingPerson, BookingStatus, CreateBookingRequest, RecordPaymentRequest};
use crate::models::calendar::CalendarStatus;

/// Booking row locked for write; carries the fields the service needs for business-rule checks.
pub struct LockedBooking {
    pub id: i64,
    pub status: BookingStatus,
    pub house_id: i64,
    pub from_date: NaiveDate,
    pub to_date: NaiveDate,
}

/// Calendar row fetched under lock; carries the fields needed to validate and price a booking.
pub struct CalendarEntryRow {
    pub id: i64,
    pub status: CalendarStatus,
    pub price: Decimal,
}

#[tracing::instrument(skip(pool), fields(layer = "repository"))]
pub async fn find_all(pool: &PgPool, house_id: Option<i64>, person_id: Option<i64>, limit: Option<i64>, offset: Option<i64>) -> Result<Vec<Booking>, AppError> {
    let rows = sqlx::query!(
        r#"
        SELECT
            b.id,
            b.house_id,
            h.name           AS house_name,
            b.person_id,
            p.first_name     AS person_first_name,
            p.last_name      AS person_last_name,
            b.from_date,
            b.to_date,
            b.status         AS "status: BookingStatus",
            b.paid_at,
            b.total_paid
        FROM bookings b
        JOIN houses h  ON h.id = b.house_id
        JOIN persons p ON p.id = b.person_id
        WHERE ($1::bigint IS NULL OR b.house_id = $1)
          AND ($2::bigint IS NULL OR b.person_id = $2)
        ORDER BY b.id
        LIMIT $3::bigint OFFSET COALESCE($4::bigint, 0)
        "#,
        house_id,
        person_id,
        limit,
        offset,
    )
    .fetch_all(pool)
    .await?;

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
            paid_at: r.paid_at,
            total_paid: r.total_paid,
        })
        .collect())
}

#[tracing::instrument(skip(pool), fields(layer = "repository"))]
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
            b.from_date,
            b.to_date,
            b.status         AS "status: BookingStatus",
            b.paid_at,
            b.total_paid
        FROM bookings b
        JOIN houses h  ON h.id = b.house_id
        JOIN persons p ON p.id = b.person_id
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
        paid_at: r.paid_at,
        total_paid: r.total_paid,
    })
}

/// Locks the booking row for update and returns the fields the service needs to validate.
/// Returns 404 if the booking does not exist.
#[tracing::instrument(skip(tx), fields(layer = "repository"))]
pub async fn lock_for_write(tx: &mut Transaction<'_, Postgres>, id: i64) -> Result<LockedBooking, AppError> {
    let r = sqlx::query!(
        r#"
        SELECT id, status AS "status: BookingStatus", house_id, from_date, to_date
        FROM bookings
        WHERE id = $1
        FOR UPDATE
        "#,
        id,
    )
    .fetch_one(&mut **tx)
    .await
    .map_err(|e| match e {
        sqlx::Error::RowNotFound => AppError::NotFound(format!("booking {id} not found")),
        other => AppError::from(other),
    })?;

    Ok(LockedBooking {
        id: r.id,
        status: r.status,
        house_id: r.house_id,
        from_date: r.from_date,
        to_date: r.to_date,
    })
}

/// Cancels the booking and releases its calendar days back to Rentable.
/// Caller must have locked the row via `lock_for_write` in the same transaction.
#[tracing::instrument(skip(tx, locked), fields(layer = "repository", booking_id = locked.id))]
pub async fn do_cancel(tx: &mut Transaction<'_, Postgres>, locked: &LockedBooking) -> Result<(), AppError> {
    sqlx::query!("UPDATE bookings SET status = 'Cancelled', paid_at = NULL, total_paid = NULL WHERE id = $1", locked.id,)
        .execute(&mut **tx)
        .await?;

    sqlx::query!(
        "UPDATE calendar SET status = 'Rentable' WHERE house_id = $1 AND date BETWEEN $2 AND $3",
        locked.house_id,
        locked.from_date,
        locked.to_date,
    )
    .execute(&mut **tx)
    .await?;

    Ok(())
}

/// Fetches and locks the calendar entries covering [from, to] for a given house.
/// Returns the rows in date order; the service validates them before calling `do_create_booking`.
#[tracing::instrument(skip(tx), fields(layer = "repository", house_id, from = %from, to = %to))]
pub async fn fetch_calendar_for_booking(tx: &mut Transaction<'_, Postgres>, house_id: i64, from: NaiveDate, to: NaiveDate) -> Result<Vec<CalendarEntryRow>, AppError> {
    let rows = sqlx::query!(
        r#"
        SELECT id, status AS "status: CalendarStatus", price
        FROM calendar
        WHERE house_id = $1 AND date BETWEEN $2 AND $3
        ORDER BY date
        FOR UPDATE
        "#,
        house_id,
        from,
        to,
    )
    .fetch_all(&mut **tx)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| CalendarEntryRow {
            id: r.id,
            status: r.status,
            price: r.price,
        })
        .collect())
}

/// Inserts the booking record and marks the calendar days as Rented.
/// Caller must have validated `entries` (non-empty, all Rentable) before calling this.
#[tracing::instrument(skip(tx, req, entries), fields(layer = "repository", house_id = req.house_id, person_id = req.person_id))]
pub async fn do_create_booking(tx: &mut Transaction<'_, Postgres>, req: &CreateBookingRequest, entries: &[CalendarEntryRow]) -> Result<(i64, Decimal), AppError> {
    let from_calendar_id = entries.first().expect("invariant: entries non-empty, validated by caller").id;
    let to_calendar_id = entries.last().expect("invariant: entries non-empty, validated by caller").id;
    let expected_total_price: Decimal = entries.iter().map(|e| e.price).sum();

    let booking_id: i64 = sqlx::query_scalar!(
        r#"
        INSERT INTO bookings (house_id, person_id, from_calendar_id, to_calendar_id, from_date, to_date, status)
        VALUES ($1, $2, $3, $4, $5, $6, 'Active')
        RETURNING id
        "#,
        req.house_id,
        req.person_id,
        from_calendar_id,
        to_calendar_id,
        req.from,
        req.to,
    )
    .fetch_one(&mut **tx)
    .await?;

    sqlx::query!("UPDATE calendar SET status = 'Rented' WHERE house_id = $1 AND date BETWEEN $2 AND $3", req.house_id, req.from, req.to,)
        .execute(&mut **tx)
        .await?;

    Ok((booking_id, expected_total_price))
}

/// Records payment fields on the booking.
/// Caller must have validated that the booking is not cancelled before calling this.
#[tracing::instrument(skip(tx, req), fields(layer = "repository", booking_id = id))]
pub async fn do_record_payment(tx: &mut Transaction<'_, Postgres>, id: i64, req: &RecordPaymentRequest) -> Result<(), AppError> {
    sqlx::query!("UPDATE bookings SET paid_at = $1, total_paid = $2 WHERE id = $3", req.paid_at, req.total_paid, id,)
        .execute(&mut **tx)
        .await?;

    Ok(())
}
