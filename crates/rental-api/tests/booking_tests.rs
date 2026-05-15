mod common;

use rental_api::errors::AppError;
use rental_api::models::booking::{BookingStatus, CreateBookingRequest, RecordPaymentRequest};
use rental_api::models::calendar::{CalendarStatus, CreateCalendarRequest};
use rental_api::services::{booking as booking_svc, calendar as cal_svc};
use sqlx::PgPool;

// ---------------------------------------------------------------------------
// create: flips all calendar entries to Rented on success
// ---------------------------------------------------------------------------

#[sqlx::test(migrations = "../../migrations")]
async fn create_booking_should_flip_days_to_rented(pool: PgPool) {
    let house_id = common::create_test_house(&pool).await;
    let person_id = common::create_test_person(&pool).await;

    cal_svc::create(
        &pool,
        house_id,
        &CreateCalendarRequest {
            from: common::d("2024-06-10"),
            to: common::d("2024-06-12"),
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
            from: common::d("2024-06-10"),
            to: common::d("2024-06-12"),
        },
    )
    .await
    .unwrap();

    let entries = cal_svc::list(&pool, house_id, Some(common::d("2024-06-10")), Some(common::d("2024-06-12"))).await.unwrap();
    assert!(entries.iter().all(|e| e.status == CalendarStatus::Rented));
}

// ---------------------------------------------------------------------------
// create: returns expected_total_price computed from daily prices
// ---------------------------------------------------------------------------

#[sqlx::test(migrations = "../../migrations")]
async fn create_booking_should_return_expected_total_price(pool: PgPool) {
    let house_id = common::create_test_house(&pool).await;
    let person_id = common::create_test_person(&pool).await;

    let day_price = common::price(15000); // 150.00 per day

    cal_svc::create(
        &pool,
        house_id,
        &CreateCalendarRequest {
            from: common::d("2024-06-01"),
            to: common::d("2024-06-03"),
            status: CalendarStatus::Rentable,
            price: day_price,
        },
    )
    .await
    .unwrap();

    let booking = booking_svc::create(
        &pool,
        &CreateBookingRequest {
            house_id,
            person_id,
            from: common::d("2024-06-01"),
            to: common::d("2024-06-03"),
        },
    )
    .await
    .unwrap();

    assert_eq!(booking.expected_total_price, Some(day_price * rust_decimal::Decimal::from(3)));
}

// ---------------------------------------------------------------------------
// create: fails when at least one day is NotRentable
// ---------------------------------------------------------------------------

#[sqlx::test(migrations = "../../migrations")]
async fn create_booking_should_fail_when_day_not_rentable(pool: PgPool) {
    let house_id = common::create_test_house(&pool).await;
    let person_id = common::create_test_person(&pool).await;

    cal_svc::create(
        &pool,
        house_id,
        &CreateCalendarRequest {
            from: common::d("2024-07-01"),
            to: common::d("2024-07-02"),
            status: CalendarStatus::Rentable,
            price: common::price(10000),
        },
    )
    .await
    .unwrap();
    cal_svc::create(
        &pool,
        house_id,
        &CreateCalendarRequest {
            from: common::d("2024-07-03"),
            to: common::d("2024-07-03"),
            status: CalendarStatus::NotRentable,
            price: common::price(10000),
        },
    )
    .await
    .unwrap();
    cal_svc::create(
        &pool,
        house_id,
        &CreateCalendarRequest {
            from: common::d("2024-07-04"),
            to: common::d("2024-07-05"),
            status: CalendarStatus::Rentable,
            price: common::price(10000),
        },
    )
    .await
    .unwrap();

    let result = booking_svc::create(
        &pool,
        &CreateBookingRequest {
            house_id,
            person_id,
            from: common::d("2024-07-01"),
            to: common::d("2024-07-05"),
        },
    )
    .await;

    assert!(matches!(result, Err(AppError::UnprocessableEntity(_))), "expected UnprocessableEntity, got {result:?}");
}

// ---------------------------------------------------------------------------
// create: fails when calendar entries are missing for some days in the range
// ---------------------------------------------------------------------------

#[sqlx::test(migrations = "../../migrations")]
async fn create_booking_should_fail_when_entries_are_missing(pool: PgPool) {
    let house_id = common::create_test_house(&pool).await;
    let person_id = common::create_test_person(&pool).await;

    cal_svc::create(
        &pool,
        house_id,
        &CreateCalendarRequest {
            from: common::d("2024-08-01"),
            to: common::d("2024-08-03"),
            status: CalendarStatus::Rentable,
            price: common::price(10000),
        },
    )
    .await
    .unwrap();

    let result = booking_svc::create(
        &pool,
        &CreateBookingRequest {
            house_id,
            person_id,
            from: common::d("2024-08-01"),
            to: common::d("2024-08-05"),
        },
    )
    .await;

    assert!(matches!(result, Err(AppError::UnprocessableEntity(_))), "expected UnprocessableEntity, got {result:?}");
}

// ---------------------------------------------------------------------------
// cancel: flips calendar entries back to Rentable
// ---------------------------------------------------------------------------

#[sqlx::test(migrations = "../../migrations")]
async fn cancel_booking_should_flip_days_back_to_rentable(pool: PgPool) {
    let house_id = common::create_test_house(&pool).await;
    let person_id = common::create_test_person(&pool).await;

    cal_svc::create(
        &pool,
        house_id,
        &CreateCalendarRequest {
            from: common::d("2024-09-01"),
            to: common::d("2024-09-03"),
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
            from: common::d("2024-09-01"),
            to: common::d("2024-09-03"),
        },
    )
    .await
    .unwrap();

    booking_svc::cancel(&pool, booking.id).await.unwrap();

    let entries = cal_svc::list(&pool, house_id, Some(common::d("2024-09-01")), Some(common::d("2024-09-03"))).await.unwrap();
    assert!(entries.iter().all(|e| e.status == CalendarStatus::Rentable));
}

// ---------------------------------------------------------------------------
// cancel: fails when booking is already cancelled
// ---------------------------------------------------------------------------

#[sqlx::test(migrations = "../../migrations")]
async fn cancel_booking_should_fail_when_already_cancelled(pool: PgPool) {
    let house_id = common::create_test_house(&pool).await;
    let person_id = common::create_test_person(&pool).await;

    cal_svc::create(
        &pool,
        house_id,
        &CreateCalendarRequest {
            from: common::d("2024-10-01"),
            to: common::d("2024-10-02"),
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
            from: common::d("2024-10-01"),
            to: common::d("2024-10-02"),
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
async fn record_payment_should_fail_on_cancelled_booking(pool: PgPool) {
    let house_id = common::create_test_house(&pool).await;
    let person_id = common::create_test_person(&pool).await;

    cal_svc::create(
        &pool,
        house_id,
        &CreateCalendarRequest {
            from: common::d("2024-11-01"),
            to: common::d("2024-11-02"),
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
            from: common::d("2024-11-01"),
            to: common::d("2024-11-02"),
        },
    )
    .await
    .unwrap();

    booking_svc::cancel(&pool, booking.id).await.unwrap();

    let result = booking_svc::record_payment(
        &pool,
        booking.id,
        &RecordPaymentRequest {
            paid_at: common::d("2024-11-01"),
            total_paid: common::price(20000),
        },
    )
    .await;

    assert!(matches!(result, Err(AppError::UnprocessableEntity(_))), "expected UnprocessableEntity, got {result:?}");
}

// ---------------------------------------------------------------------------
// record_payment: stores paid_at and total_paid on an active booking
// ---------------------------------------------------------------------------

#[sqlx::test(migrations = "../../migrations")]
async fn record_payment_should_store_payment_fields(pool: PgPool) {
    let house_id = common::create_test_house(&pool).await;
    let person_id = common::create_test_person(&pool).await;

    let from = common::d("2024-11-01");
    let to = common::d("2024-11-02");
    let paid_at = from;
    let total_paid = common::price(20000);

    cal_svc::create(
        &pool,
        house_id,
        &CreateCalendarRequest {
            from,
            to,
            status: CalendarStatus::Rentable,
            price: common::price(10000),
        },
    )
    .await
    .unwrap();

    let booking = booking_svc::create(&pool, &CreateBookingRequest { house_id, person_id, from, to }).await.unwrap();

    let result = booking_svc::record_payment(&pool, booking.id, &RecordPaymentRequest { paid_at, total_paid }).await.unwrap();

    assert_eq!(result.status, BookingStatus::Active);
    assert_eq!(result.house.id, house_id);
    assert_eq!(result.person.id, person_id);
    assert_eq!(result.from, from);
    assert_eq!(result.to, to);
    assert_eq!(result.paid_at, Some(paid_at));
    assert_eq!(result.total_paid, Some(total_paid));
}
