use anyhow::{Context, Result};
use chrono::NaiveDate;
use rand::seq::IndexedRandom;
use sqlx::PgPool;

// COPY BINARY header: 11-byte signature + 4-byte flags (0) + 4-byte header extension length (0).
const COPY_HEADER: &[u8] = b"PGCOPY\n\xFF\r\n\0\0\0\0\0\0\0\0\0";

// ENUM wire format is just the text label.
const STATUS_RENTABLE: &[u8] = b"Rentable";

// Send to the COPY sink in 4 MiB chunks. Smaller chunks cost more syscalls;
// much larger chunks gain little once the network/Postgres are saturated.
const FLUSH_THRESHOLD: usize = 4 << 20;

/// Seed `years` of `Rentable` calendar entries for every house in `house_ids`,
/// starting on `start_date`. Each house picks one nightly price from a small
/// pre-encoded pool. Returns the number of rows inserted.
pub async fn seed(pool: &PgPool, house_ids: &[i64], start_date: NaiveDate, years: u32) -> Result<u64> {
    let days_per_house = years as i64 * 365;

    // Prices: $50–$300 in $10 increments → 26 values, each 10 bytes of binary NUMERIC.
    let price_pool: Vec<Vec<u8>> = (50..=300).step_by(10).map(encode_numeric_dollars).collect();

    let pg_epoch = NaiveDate::from_ymd_opt(2000, 1, 1).expect("valid epoch");
    let first_pg_date = (start_date - pg_epoch).num_days() as i32;

    let mut conn = pool.acquire().await.context("failed to acquire connection")?;
    let mut copy = conn
        .copy_in_raw("COPY calendar (house_id, date, status, price) FROM STDIN WITH (FORMAT BINARY)")
        .await
        .context("failed to start COPY")?;

    let mut buf: Vec<u8> = Vec::with_capacity(FLUSH_THRESHOLD + 64);
    buf.extend_from_slice(COPY_HEADER);

    let mut rng = rand::rng();
    for &house_id in house_ids {
        let price_bytes: &[u8] = price_pool.choose(&mut rng).expect("non-empty price pool");

        for offset in 0..days_per_house {
            let pg_date = first_pg_date + offset as i32;

            // 4 fields per tuple
            buf.extend_from_slice(&4i16.to_be_bytes());

            // house_id  BIGINT (8 bytes)
            buf.extend_from_slice(&8i32.to_be_bytes());
            buf.extend_from_slice(&house_id.to_be_bytes());

            // date  DATE (4 bytes, days since 2000-01-01)
            buf.extend_from_slice(&4i32.to_be_bytes());
            buf.extend_from_slice(&pg_date.to_be_bytes());

            // status  ENUM (text label as bytes)
            buf.extend_from_slice(&(STATUS_RENTABLE.len() as i32).to_be_bytes());
            buf.extend_from_slice(STATUS_RENTABLE);

            // price  NUMERIC
            buf.extend_from_slice(&(price_bytes.len() as i32).to_be_bytes());
            buf.extend_from_slice(price_bytes);

            if buf.len() >= FLUSH_THRESHOLD {
                copy.send(buf.as_slice()).await.context("COPY send failed")?;
                buf.clear();
            }
        }
    }

    // Trailer (-1 as i16) marks end of data.
    buf.extend_from_slice(&(-1i16).to_be_bytes());
    copy.send(buf.as_slice()).await.context("final COPY send failed")?;
    let inserted = copy.finish().await.context("COPY finish failed")?;

    Ok(inserted)
}

/// Binary NUMERIC encoding for a positive whole-dollar amount (1..=9999),
/// stored with `dscale = 2` so the displayed value reads "150.00" etc.
fn encode_numeric_dollars(price_dollars: i32) -> Vec<u8> {
    let mut buf = Vec::with_capacity(10);
    buf.extend_from_slice(&1i16.to_be_bytes()); // ndigits
    buf.extend_from_slice(&0i16.to_be_bytes()); // weight (one base-10000 group at 10^0)
    buf.extend_from_slice(&0i16.to_be_bytes()); // sign (positive)
    buf.extend_from_slice(&2i16.to_be_bytes()); // dscale
    buf.extend_from_slice(&(price_dollars as i16).to_be_bytes()); // digit[0]
    buf
}
