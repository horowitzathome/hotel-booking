mod common;

use rental_api::models::calendar::{CalendarStatus, CreateCalendarRequest};
use rental_api::models::country::CreateCountryRequest;
use rental_api::services::{calendar as cal_svc, country as country_svc};
use sqlx::PgPool;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

async fn insert_countries(pool: &PgPool, n: usize) {
    for i in 0..n {
        let iso_code = format!("{}{}", (b'A' + (i / 26) as u8) as char, (b'A' + (i % 26) as u8) as char);
        country_svc::create(
            pool,
            &CreateCountryRequest {
                name: format!("Country {i:02}"),
                iso_code,
            },
        )
        .await
        .unwrap();
    }
}

// ---------------------------------------------------------------------------
// limit
// ---------------------------------------------------------------------------

#[sqlx::test(migrations = "../../migrations")]
async fn list_with_limit_returns_at_most_limit_rows(pool: PgPool) {
    insert_countries(&pool, 5).await;

    let results = country_svc::list(&pool, Some(2), None).await.unwrap();

    assert_eq!(results.len(), 2);
}

// ---------------------------------------------------------------------------
// offset
// ---------------------------------------------------------------------------

#[sqlx::test(migrations = "../../migrations")]
async fn list_with_offset_skips_leading_rows(pool: PgPool) {
    insert_countries(&pool, 5).await;

    let all = country_svc::list(&pool, None, None).await.unwrap();
    let paged = country_svc::list(&pool, None, Some(2)).await.unwrap();

    assert_eq!(paged.len(), 3);
    assert_eq!(paged[0].name, all[2].name);
}

// ---------------------------------------------------------------------------
// limit + offset — correct middle page
// ---------------------------------------------------------------------------

#[sqlx::test(migrations = "../../migrations")]
async fn list_with_limit_and_offset_returns_correct_page(pool: PgPool) {
    insert_countries(&pool, 5).await;

    let all = country_svc::list(&pool, None, None).await.unwrap();
    let page = country_svc::list(&pool, Some(2), Some(1)).await.unwrap();

    assert_eq!(page.len(), 2);
    assert_eq!(page[0].name, all[1].name);
    assert_eq!(page[1].name, all[2].name);
}

// ---------------------------------------------------------------------------
// no params → backward-compatible: all rows returned
// ---------------------------------------------------------------------------

#[sqlx::test(migrations = "../../migrations")]
async fn list_without_pagination_returns_all_rows(pool: PgPool) {
    insert_countries(&pool, 5).await;

    let results = country_svc::list(&pool, None, None).await.unwrap();

    assert_eq!(results.len(), 5);
}

// ---------------------------------------------------------------------------
// offset beyond row count → empty
// ---------------------------------------------------------------------------

#[sqlx::test(migrations = "../../migrations")]
async fn list_with_offset_beyond_row_count_returns_empty(pool: PgPool) {
    insert_countries(&pool, 3).await;

    let results = country_svc::list(&pool, None, Some(100)).await.unwrap();

    assert!(results.is_empty());
}

// ---------------------------------------------------------------------------
// calendar: date filter + limit interact correctly
// ---------------------------------------------------------------------------

#[sqlx::test(migrations = "../../migrations")]
async fn calendar_list_date_filter_and_limit_are_applied_together(pool: PgPool) {
    let house_id = common::create_test_house(&pool).await;

    cal_svc::create(
        &pool,
        house_id,
        &CreateCalendarRequest {
            from: common::d("2024-06-01"),
            to: common::d("2024-06-10"),
            status: CalendarStatus::Rentable,
            price: common::price(10000),
        },
    )
    .await
    .unwrap();

    // Without limit: all 10 days in the date window
    let all = cal_svc::list(&pool, house_id, Some(common::d("2024-06-01")), Some(common::d("2024-06-10")), None, None).await.unwrap();
    assert_eq!(all.len(), 10);

    // With limit=3: first 3 days in date order
    let limited = cal_svc::list(&pool, house_id, Some(common::d("2024-06-01")), Some(common::d("2024-06-10")), Some(3), None)
        .await
        .unwrap();
    assert_eq!(limited.len(), 3);
    assert_eq!(limited[0].date, common::d("2024-06-01"));
    assert_eq!(limited[2].date, common::d("2024-06-03"));

    // With offset=7: last 3 days
    let tail = cal_svc::list(&pool, house_id, Some(common::d("2024-06-01")), Some(common::d("2024-06-10")), None, Some(7))
        .await
        .unwrap();
    assert_eq!(tail.len(), 3);
    assert_eq!(tail[0].date, common::d("2024-06-08"));
}
