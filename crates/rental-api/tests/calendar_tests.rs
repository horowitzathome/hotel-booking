mod common;

use rental_api::errors::AppError;
use rental_api::models::booking::CreateBookingRequest;
use rental_api::models::calendar::{CalendarStatus, CreateCalendarRequest};
use rental_api::services::{booking as booking_svc, calendar as cal_svc};
use sqlx::PgPool;

// ---------------------------------------------------------------------------
// create: idempotent — skips days that already have an entry
// ---------------------------------------------------------------------------

#[sqlx::test(migrations = "../../migrations")]
async fn create_calendar_should_skip_existing_entries(pool: PgPool) {
    let house_id = common::create_test_house(&pool).await;

    let req1 = CreateCalendarRequest {
        from: common::d("2024-07-01"),
        to: common::d("2024-07-05"),
        status: CalendarStatus::Rentable,
        price: common::price(10000),
    };
    let first = cal_svc::create(&pool, house_id, &req1).await.unwrap();
    assert_eq!(first.len(), 5);

    let req2 = CreateCalendarRequest {
        from: common::d("2024-07-03"),
        to: common::d("2024-07-07"),
        status: CalendarStatus::Rentable,
        price: common::price(12000),
    };
    let second = cal_svc::create(&pool, house_id, &req2).await.unwrap();
    assert_eq!(second.len(), 2);
    assert_eq!(second[0].date, common::d("2024-07-06"));
    assert_eq!(second[1].date, common::d("2024-07-07"));
}

// ---------------------------------------------------------------------------
// create: same range twice returns empty on second call
// ---------------------------------------------------------------------------

#[sqlx::test(migrations = "../../migrations")]
async fn create_calendar_should_return_empty_when_range_fully_overlaps(pool: PgPool) {
    let house_id = common::create_test_house(&pool).await;

    let req = CreateCalendarRequest {
        from: common::d("2024-08-01"),
        to: common::d("2024-08-03"),
        status: CalendarStatus::NotRentable,
        price: common::price(5000),
    };
    cal_svc::create(&pool, house_id, &req).await.unwrap();

    let second = cal_svc::create(&pool, house_id, &req).await.unwrap();
    assert!(second.is_empty());
}

// ---------------------------------------------------------------------------
// delete: blocked when an active booking covers any entry in the range
// ---------------------------------------------------------------------------

#[sqlx::test(migrations = "../../migrations")]
async fn delete_calendar_should_fail_when_active_booking_covers_range(pool: PgPool) {
    let house_id = common::create_test_house(&pool).await;
    let person_id = common::create_test_person(&pool).await;

    cal_svc::create(
        &pool,
        house_id,
        &CreateCalendarRequest {
            from: common::d("2024-07-01"),
            to: common::d("2024-07-10"),
            status: CalendarStatus::Rentable,
            price: common::price(10000),
        },
    )
    .await
    .unwrap();

    booking_svc::create(
        &pool,
        &CreateBookingRequest {
            house_id,
            person_id,
            from: common::d("2024-07-03"),
            to: common::d("2024-07-06"),
        },
    )
    .await
    .unwrap();

    let result = cal_svc::delete(&pool, house_id, common::d("2024-07-01"), common::d("2024-07-10")).await;
    assert!(matches!(result, Err(AppError::Conflict(_))), "expected Conflict, got {result:?}");
}

// ---------------------------------------------------------------------------
// delete: succeeds once the booking over those entries is cancelled
// ---------------------------------------------------------------------------

#[sqlx::test(migrations = "../../migrations")]
async fn delete_calendar_should_succeed_after_covering_booking_is_cancelled(pool: PgPool) {
    let house_id = common::create_test_house(&pool).await;
    let person_id = common::create_test_person(&pool).await;

    cal_svc::create(
        &pool,
        house_id,
        &CreateCalendarRequest {
            from: common::d("2024-07-01"),
            to: common::d("2024-07-05"),
            status: CalendarStatus::Rentable,
            price: common::price(10000),
        },
    )
    .await
    .unwrap();

    let booking = booking_svc::create(
        &pool,
        &CreateBookingRequest {
            house_id,
            person_id,
            from: common::d("2024-07-02"),
            to: common::d("2024-07-04"),
        },
    )
    .await
    .unwrap();

    booking_svc::cancel(&pool, booking.id).await.unwrap();

    cal_svc::delete(&pool, house_id, common::d("2024-07-01"), common::d("2024-07-05")).await.unwrap();
}

// ---------------------------------------------------------------------------
// delete: non-booked entries outside any booking range can always be removed
// ---------------------------------------------------------------------------

#[sqlx::test(migrations = "../../migrations")]
async fn delete_calendar_should_succeed_when_no_bookings_cover_range(pool: PgPool) {
    let house_id = common::create_test_house(&pool).await;

    cal_svc::create(
        &pool,
        house_id,
        &CreateCalendarRequest {
            from: common::d("2024-09-01"),
            to: common::d("2024-09-05"),
            status: CalendarStatus::NotRentable,
            price: common::price(8000),
        },
    )
    .await
    .unwrap();

    cal_svc::delete(&pool, house_id, common::d("2024-09-01"), common::d("2024-09-05")).await.unwrap();
}
