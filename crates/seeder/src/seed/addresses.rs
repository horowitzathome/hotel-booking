use crate::seed::BATCH_SIZE;
use anyhow::{Context, Result, anyhow};
use fake::Fake;
use fake::faker::address::en::{BuildingNumber, CityName, PostCode, StateName, StreetName};
use rand::Rng;
use rand::seq::IndexedRandom;
use sqlx::PgPool;

pub async fn seed(pool: &PgPool, count: usize, country_ids: &[i64]) -> Result<Vec<i64>> {
    if country_ids.is_empty() {
        return Err(anyhow!("addresses.seed needs at least one country"));
    }

    let mut rng = rand::rng();
    let mut ids = Vec::with_capacity(count);
    let mut tx = pool.begin().await?;

    let mut idx = 0;
    while idx < count {
        let batch = (count - idx).min(BATCH_SIZE);
        let mut streets = Vec::with_capacity(batch);
        let mut numbers = Vec::with_capacity(batch);
        let mut postcodes = Vec::with_capacity(batch);
        let mut cities = Vec::with_capacity(batch);
        let mut provinces: Vec<Option<String>> = Vec::with_capacity(batch);
        let mut countries = Vec::with_capacity(batch);

        for _ in 0..batch {
            streets.push(StreetName().fake::<String>());
            numbers.push(BuildingNumber().fake::<String>());
            postcodes.push(PostCode().fake::<String>());
            cities.push(CityName().fake::<String>());
            // ~30% of addresses have a province set.
            provinces.push(if rng.random_bool(0.3) { Some(StateName().fake::<String>()) } else { None });
            countries.push(*country_ids.choose(&mut rng).context("country_ids was empty")?);
        }

        let batch_ids: Vec<i64> = sqlx::query_scalar(
            "INSERT INTO addresses (street, number, postcode, city, province, country_id)
             SELECT * FROM UNNEST($1::text[], $2::text[], $3::text[], $4::text[], $5::text[], $6::bigint[])
             RETURNING id",
        )
        .bind(&streets)
        .bind(&numbers)
        .bind(&postcodes)
        .bind(&cities)
        .bind(&provinces)
        .bind(&countries)
        .fetch_all(&mut *tx)
        .await?;

        ids.extend(batch_ids);
        idx += batch;
    }

    tx.commit().await?;
    Ok(ids)
}
