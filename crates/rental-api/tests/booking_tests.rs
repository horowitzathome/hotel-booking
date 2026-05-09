mod common;

use chrono::NaiveDate;
use rental_api::errors::AppError;
use rental_api::models::booking::{BookingStatus, CreateBookingRequest, RecordPaymentRequest};
use rental_api::models::calendar::{CalendarStatus, CreateCalendarRequest};
use rental_api::services::{booking as booking_svc, calendar as cal_svc};
use rust_decimal::Decimal;
use sqlx::PgPool;

fn d(s: &str) -> NaiveDate {
    NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap()
}

fn price(cents: i64) -> Decimal {
    Decimal::new(cents, 2)
}

// ---------------------------------------------------------------------------
// create: flips all calendar entries to Rented on success
// ---------------------------------------------------------------------------

#[sqlx::test(migrations = "../../migrations")]
async fn booking_create_flips_days_to_rented(pool: PgPool) {
    let house_id = common::create_test_house(&pool).await;
    let person_id = common::create_test_person(&pool).await;

    cal_svc::create(
        &pool,
        house_id,
        &CreateCalendarRequest {
            from: d("2024-06-10"),
            to: d("2024-06-12"),
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
            from: d("2024-06-10"),
            to: d("2024-06-12"),
        },
    )
    .await
    .unwrap();

    let entries = cal_svc::list(&pool, house_id, Some(d("2024-06-10")), Some(d("2024-06-12"))).await.unwrap();
    assert!(entries.iter().all(|e| e.status == CalendarStatus::Rented));
}

// ---------------------------------------------------------------------------
// create: returns expected_total_price computed from daily prices
// ---------------------------------------------------------------------------

#[sqlx::test(migrations = "../../migrations")]
async fn booking_create_returns_expected_total_price(pool: PgPool) {
    let house_id = common::create_test_house(&pool).await;
    let person_id = common::create_test_person(&pool).await;

    cal_svc::create(
        &pool,
        house_id,
        &CreateCalendarRequest {
            from: d("2024-06-01"),
            to: d("2024-06-03"),
            status: CalendarStatus::Rentable,
            price: price(15000), // 150.00 per day
        },
    )
    .await
    .unwrap();

    let booking = booking_svc::create(
        &pool,
        &CreateBookingRequest {
            house_id,
            person_id,
            from: d("2024-06-01"),
            to: d("2024-06-03"),
        },
    )
    .await
    .unwrap();

    // 3 days × 150.00 = 450.00
    assert_eq!(booking.expected_total_price, Some(Decimal::new(45000, 2)));
}

// ---------------------------------------------------------------------------
// create: fails when at least one day is NotRentable
// ---------------------------------------------------------------------------

#[sqlx::test(migrations = "../../migrations")]
async fn booking_create_fails_when_day_not_rentable(pool: PgPool) {
    let house_id = common::create_test_house(&pool).await;
    let person_id = common::create_test_person(&pool).await;

    // Two rentable days surrounding one NotRentable day.
    cal_svc::create(
        &pool,
        house_id,
        &CreateCalendarRequest {
            from: d("2024-07-01"),
            to: d("2024-07-02"),
            status: CalendarStatus::Rentable,
            price: price(10000),
        },
    )
    .await
    .unwrap();
    cal_svc::create(
        &pool,
        house_id,
        &CreateCalendarRequest {
            from: d("2024-07-03"),
            to: d("2024-07-03"),
            status: CalendarStatus::NotRentable,
            price: price(10000),
        },
    )
    .await
    .unwrap();
    cal_svc::create(
        &pool,
        house_id,
        &CreateCalendarRequest {
            from: d("2024-07-04"),
            to: d("2024-07-05"),
            status: CalendarStatus::Rentable,
            price: price(10000),
        },
    )
    .await
    .unwrap();

    let result = booking_svc::create(
        &pool,
        &CreateBookingRequest {
            house_id,
            person_id,
            from: d("2024-07-01"),
            to: d("2024-07-05"),
        },
    )
    .await;

    assert!(matches!(result, Err(AppError::UnprocessableEntity(_))), "expected UnprocessableEntity, got {result:?}");
}

// ---------------------------------------------------------------------------
// create: fails when calendar entries are missing for some days in the range
// ---------------------------------------------------------------------------

#[sqlx::test(migrations = "../../migrations")]
async fn booking_create_fails_when_entries_missing(pool: PgPool) {
    let house_id = common::create_test_house(&pool).await;
    let person_id = common::create_test_person(&pool).await;

    // Only days 1–3 exist; booking asks for 1–5.
    cal_svc::create(
        &pool,
        house_id,
        &CreateCalendarRequest {
            from: d("2024-08-01"),
            to: d("2024-08-03"),
            status: CalendarStatus::Rentable,
            price: price(10000),
        },
    )
    .await
    .unwrap();

    let result = booking_svc::create(
        &pool,
        &CreateBookingRequest {
            house_id,
            person_id,
            from: d("2024-08-01"),
            to: d("2024-08-05"),
        },
    )
    .await;

    assert!(matches!(result, Err(AppError::UnprocessableEntity(_))), "expected UnprocessableEntity, got {result:?}");
}

// ---------------------------------------------------------------------------
// cancel: flips calendar entries back to Rentable
// ---------------------------------------------------------------------------

#[sqlx::test(migrations = "../../migrations")]
async fn booking_cancel_flips_days_back_to_rentable(pool: PgPool) {
    let house_id = common::create_test_house(&pool).await;
    let person_id = common::create_test_person(&pool).await;

    cal_svc::create(
        &pool,
        house_id,
        &CreateCalendarRequest {
            from: d("2024-09-01"),
            to: d("2024-09-03"),
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
            from: d("2024-09-01"),
            to: d("2024-09-03"),
        },
    )
    .await
    .unwrap();

    booking_svc::cancel(&pool, booking.id).await.unwrap();

    let entries = cal_svc::list(&pool, house_id, Some(d("2024-09-01")), Some(d("2024-09-03"))).await.unwrap();
    assert!(entries.iter().all(|e| e.status == CalendarStatus::Rentable));
}

// ---------------------------------------------------------------------------
// cancel: fails when booking is already cancelled
// ---------------------------------------------------------------------------

#[sqlx::test(migrations = "../../migrations")]
async fn booking_cancel_fails_when_already_cancelled(pool: PgPool) {
    let house_id = common::create_test_house(&pool).await;
    let person_id = common::create_test_person(&pool).await;

    cal_svc::create(
        &pool,
        house_id,
        &CreateCalendarRequest {
            from: d("2024-10-01"),
            to: d("2024-10-02"),
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
            from: d("2024-10-01"),
            to: d("2024-10-02"),
        },
    )
    .await
    .unwrap();

    booking_svc::cancel(&pool, booking.id).await.unwrap();

    let result = booking_svc::cancel(&pool, booking.id).await;
    assert!(matches!(result, Err(AppError::UnprocessableEntity(_))), "expected UnprocessableEntity on double-cancel, got {result:?}");
}

// ---------------------------------------------------------------------------
// record_payment: fails when booking is cancelled
// ---------------------------------------------------------------------------

#[sqlx::test(migrations = "../../migrations")]
async fn payment_fails_on_cancelled_booking(pool: PgPool) {
    let house_id = common::create_test_house(&pool).await;
    let person_id = common::create_test_person(&pool).await;

    cal_svc::create(
        &pool,
        house_id,
        &CreateCalendarRequest {
            from: d("2024-11-01"),
            to: d("2024-11-02"),
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
            from: d("2024-11-01"),
            to: d("2024-11-02"),
        },
    )
    .await
    .unwrap();

    booking_svc::cancel(&pool, booking.id).await.unwrap();

    let result = booking_svc::record_payment(
        &pool,
        booking.id,
        &RecordPaymentRequest {
            paid_at: d("2024-11-01"),
            total_paid: price(20000),
        },
    )
    .await;

    assert!(matches!(result, Err(AppError::UnprocessableEntity(_))), "expected UnprocessableEntity, got {result:?}");
}

// ---------------------------------------------------------------------------
// record_payment: do a successful payment
// ---------------------------------------------------------------------------

#[sqlx::test(migrations = "../../migrations")]
async fn payment_successful(pool: PgPool) {
    let house_id = common::create_test_house(&pool).await;
    let person_id = common::create_test_person(&pool).await;

    let from = d("2024-11-01");
    let to = d("2024-11-02");

    cal_svc::create(
        &pool,
        house_id,
        &CreateCalendarRequest {
            from,
            to,
            status: CalendarStatus::Rentable,
            price: price(10000),
        },
    )
    .await
    .unwrap();

    let booking = booking_svc::create(&pool, &CreateBookingRequest { house_id, person_id, from, to }).await.unwrap();

    let price = price(20000);

    let result = booking_svc::record_payment(&pool, booking.id, &RecordPaymentRequest { paid_at: from, total_paid: price }).await;

    assert!(result.is_ok());

    let booking = result.unwrap();
    assert!(booking.person.id == person_id);
    assert!(booking.total_paid.unwrap() == price);
    assert!(booking.from == from);
    assert!(booking.to == to);
    assert!(booking.house.id == house_id);
    assert!(booking.status == BookingStatus::Active);
}
