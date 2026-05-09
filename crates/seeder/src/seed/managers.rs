use crate::seed::BATCH_SIZE;
use anyhow::Result;
use fake::Fake;
use fake::faker::name::en::{FirstName, LastName};
use fake::faker::phone_number::en::PhoneNumber;
use sqlx::PgPool;

pub async fn seed(pool: &PgPool, count: usize) -> Result<Vec<i64>> {
    let mut ids = Vec::with_capacity(count);
    let mut tx = pool.begin().await?;

    let mut idx = 0;
    while idx < count {
        let batch = (count - idx).min(BATCH_SIZE);
        let mut first_names = Vec::with_capacity(batch);
        let mut last_names = Vec::with_capacity(batch);
        let mut emails = Vec::with_capacity(batch);
        let mut phones = Vec::with_capacity(batch);

        for i in idx..idx + batch {
            let first: String = FirstName().fake();
            let last: String = LastName().fake();
            let phone: String = PhoneNumber().fake();
            // Synthesize a unique email — fake's email is random and would collide.
            let email = format!("{}.{}.{i}@managers.example.com", first.to_lowercase(), last.to_lowercase());
            first_names.push(first);
            last_names.push(last);
            emails.push(email);
            phones.push(phone);
        }

        let batch_ids: Vec<i64> = sqlx::query_scalar(
            "INSERT INTO managers (first_name, last_name, email, phone)
             SELECT * FROM UNNEST($1::text[], $2::text[], $3::text[], $4::text[])
             RETURNING id",
        )
        .bind(&first_names)
        .bind(&last_names)
        .bind(&emails)
        .bind(&phones)
        .fetch_all(&mut *tx)
        .await?;

        ids.extend(batch_ids);
        idx += batch;
    }

    tx.commit().await?;
    Ok(ids)
}
