use sqlx::PgPool;

/// Creates the full hierarchy required by a house: country → address → manager → house.
/// Returns the house id.
pub async fn create_test_house(pool: &PgPool) -> i64 {
    let country_id: i64 = sqlx::query_scalar(
        "INSERT INTO countries (name, iso_code) VALUES ('Germany', 'DE') RETURNING id",
    )
    .fetch_one(pool)
    .await
    .unwrap();

    let address_id: i64 = sqlx::query_scalar(
        "INSERT INTO addresses (street, number, postcode, city, country_id) \
         VALUES ('Main St', '1', '10001', 'Berlin', $1) RETURNING id",
    )
    .bind(country_id)
    .fetch_one(pool)
    .await
    .unwrap();

    let manager_id: i64 = sqlx::query_scalar(
        "INSERT INTO managers (first_name, last_name, email, phone) \
         VALUES ('Hans', 'Mueller', 'hans@test.com', '+49123') RETURNING id",
    )
    .fetch_one(pool)
    .await
    .unwrap();

    sqlx::query_scalar(
        "INSERT INTO houses (name, description, address_id, manager_id) \
         VALUES ('Test House', 'Test description', $1, $2) RETURNING id",
    )
    .bind(address_id)
    .bind(manager_id)
    .fetch_one(pool)
    .await
    .unwrap()
}

pub async fn create_test_person(pool: &PgPool) -> i64 {
    sqlx::query_scalar(
        "INSERT INTO persons (first_name, last_name, email, phone) \
         VALUES ('Anna', 'Schmidt', 'anna@test.com', '+49987') RETURNING id",
    )
    .fetch_one(pool)
    .await
    .unwrap()
}
