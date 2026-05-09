mod common;

use chrono::NaiveDate;
use claude_test::errors::AppError;
use claude_test::models::booking::CreateBookingRequest;
use claude_test::models::calendar::{CalendarStatus, CreateCalendarRequest};
use claude_test::services::{booking as booking_svc, calendar as cal_svc};
use rust_decimal::Decimal;
use sqlx::PgPool;

fn d(s: &str) -> NaiveDate {
    NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap()
}

fn price(cents: i64) -> Decimal {
    Decimal::new(cents, 2)
}

// ---------------------------------------------------------------------------
// create: idempotent — skips days that already have an entry
// ---------------------------------------------------------------------------

#[sqlx::test(migrations = "./migrations")]
async fn create_skips_existing_entries(pool: PgPool) {
    let house_id = common::create_test_house(&pool).await;

    let req1 = CreateCalendarRequest {
        from: d("2024-07-01"),
        to: d("2024-07-05"),
        status: CalendarStatus::Rentable,
        price: price(10000),
    };
    let first = cal_svc::create(&pool, house_id, &req1).await.unwrap();
    assert_eq!(first.len(), 5);

    // Range overlaps with the first batch — days 3–5 already exist; only 6–7 are new.
    let req2 = CreateCalendarRequest {
        from: d("2024-07-03"),
        to: d("2024-07-07"),
        status: CalendarStatus::Rentable,
        price: price(12000),
    };
    let second = cal_svc::create(&pool, house_id, &req2).await.unwrap();
    assert_eq!(second.len(), 2);
    assert_eq!(second[0].date, d("2024-07-06"));
    assert_eq!(second[1].date, d("2024-07-07"));
}

// ---------------------------------------------------------------------------
// create: same range twice returns empty on second call
// ---------------------------------------------------------------------------

#[sqlx::test(migrations = "./migrations")]
async fn create_fully_overlapping_range_returns_empty(pool: PgPool) {
    let house_id = common::create_test_house(&pool).await;

    let req = CreateCalendarRequest {
        from: d("2024-08-01"),
        to: d("2024-08-03"),
        status: CalendarStatus::NotRentable,
        price: price(5000),
    };
    cal_svc::create(&pool, house_id, &req).await.unwrap();

    let second = cal_svc::create(&pool, house_id, &req).await.unwrap();
    assert!(second.is_empty());
}

// ---------------------------------------------------------------------------
// delete: blocked when an active booking covers any entry in the range
// ---------------------------------------------------------------------------

#[sqlx::test(migrations = "./migrations")]
async fn delete_blocked_by_active_booking(pool: PgPool) {
    let house_id = common::create_test_house(&pool).await;
    let person_id = common::create_test_person(&pool).await;

    cal_svc::create(
        &pool,
        house_id,
        &CreateCalendarRequest {
            from: d("2024-07-01"),
            to: d("2024-07-10"),
            status: CalendarStatus::Rentable,
            price: price(10000),
        },
    )
    .await
    .unwrap();

    booking_svc::create(
        &pool,
        &CreateBookingRequest {
            house_id,
            person_id,
            from: d("2024-07-03"),
            to: d("2024-07-06"),
        },
    )
    .await
    .unwrap();

    // Attempt to delete a range that includes the booked days.
    let result = cal_svc::delete(&pool, house_id, d("2024-07-01"), d("2024-07-10")).await;
    assert!(matches!(result, Err(AppError::Conflict(_))), "expected Conflict, got {result:?}");
}

// ---------------------------------------------------------------------------
// delete: succeeds once the booking over those entries is cancelled
// ---------------------------------------------------------------------------

#[sqlx::test(migrations = "./migrations")]
async fn delete_succeeds_after_booking_cancelled(pool: PgPool) {
    let house_id = common::create_test_house(&pool).await;
    let person_id = common::create_test_person(&pool).await;

    cal_svc::create(
        &pool,
        house_id,
        &CreateCalendarRequest {
            from: d("2024-07-01"),
            to: d("2024-07-05"),
            status: CalendarStatus::Rentable,
            price: price(10000),
        },
    )
    .await
    .unwrap();

    let booking = booking_svc::create(
        &pool,
        &CreateBookingRequest {
            house_id,
            person_id,
            from: d("2024-07-02"),
            to: d("2024-07-04"),
        },
    )
    .await
    .unwrap();

    booking_svc::cancel(&pool, booking.id).await.unwrap();

    let result = cal_svc::delete(&pool, house_id, d("2024-07-01"), d("2024-07-05")).await;
    assert!(result.is_ok(), "delete after cancel should succeed, got {result:?}");
}

// ---------------------------------------------------------------------------
// delete: non-booked entries outside any booking range can always be removed
// ---------------------------------------------------------------------------

#[sqlx::test(migrations = "./migrations")]
async fn delete_unboooked_entries_succeeds(pool: PgPool) {
    let house_id = common::create_test_house(&pool).await;

    cal_svc::create(
        &pool,
        house_id,
        &CreateCalendarRequest {
            from: d("2024-09-01"),
            to: d("2024-09-05"),
            status: CalendarStatus::NotRentable,
            price: price(8000),
        },
    )
    .await
    .unwrap();

    let result = cal_svc::delete(&pool, house_id, d("2024-09-01"), d("2024-09-05")).await;
    assert!(result.is_ok());

    // Keep DB
    // assert!(false, "keep DB");
}
