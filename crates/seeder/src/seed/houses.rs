use crate::seed::BATCH_SIZE;
use anyhow::{Context, Result, anyhow};
use fake::Fake;
use fake::faker::lorem::en::{Sentence, Word};
use rand::seq::IndexedRandom;
use sqlx::PgPool;

pub async fn seed(pool: &PgPool, count: usize, address_ids: &[i64], manager_ids: &[i64]) -> Result<Vec<i64>> {
    if address_ids.is_empty() {
        return Err(anyhow!("houses.seed needs at least one address"));
    }
    if manager_ids.is_empty() {
        return Err(anyhow!("houses.seed needs at least one manager"));
    }

    let mut rng = rand::rng();
    let mut ids = Vec::with_capacity(count);
    let mut tx = pool.begin().await?;

    let mut idx = 0;
    while idx < count {
        let batch = (count - idx).min(BATCH_SIZE);
        let mut names = Vec::with_capacity(batch);
        let mut descriptions = Vec::with_capacity(batch);
        let mut addresses = Vec::with_capacity(batch);
        let mut managers = Vec::with_capacity(batch);

        for _ in 0..batch {
            let word: String = Word().fake();
            names.push(format!("Villa {}", capitalize(&word)));
            descriptions.push(Sentence(8..16).fake::<String>());
            addresses.push(*address_ids.choose(&mut rng).context("address_ids was empty")?);
            managers.push(*manager_ids.choose(&mut rng).context("manager_ids was empty")?);
        }

        let batch_ids: Vec<i64> = sqlx::query_scalar(
            "INSERT INTO houses (name, description, address_id, manager_id)
             SELECT * FROM UNNEST($1::text[], $2::text[], $3::bigint[], $4::bigint[])
             RETURNING id",
        )
        .bind(&names)
        .bind(&descriptions)
        .bind(&addresses)
        .bind(&managers)
        .fetch_all(&mut *tx)
        .await?;

        ids.extend(batch_ids);
        idx += batch;
    }

    tx.commit().await?;
    Ok(ids)
}

fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) => c.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}
