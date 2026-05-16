use anyhow::{Result, anyhow};
use chrono::{Duration, NaiveDate};
use rand::RngExt;
use rand::seq::IndexedRandom;
use rust_decimal::Decimal;
use sqlx::PgPool;

// Cap how many bookings we send in one INSERT — keeps each protocol message a
// few hundred kB rather than tens of MB. The whole table is still one transaction.
const INSERT_CHUNK: usize = 50_000;

struct PlannedBooking {
    house_id: i64,
    person_id: i64,
    from_date: NaiveDate,
    to_date: NaiveDate,
    paid_at: Option<NaiveDate>,
    total_paid: Option<Decimal>,
}

/// Plan and insert ~`target` bookings spread across `house_ids`. Bookings within
/// a house are non-overlapping by construction (deterministic stride). Returns
/// the number of bookings actually inserted.
pub async fn seed(pool: &PgPool, house_ids: &[i64], person_ids: &[i64], start_date: NaiveDate, years: u32, target: usize) -> Result<u64> {
    if house_ids.is_empty() {
        return Err(anyhow!("bookings.seed needs at least one house"));
    }
    if person_ids.is_empty() {
        return Err(anyhow!("bookings.seed needs at least one person"));
    }

    let plan = plan_bookings(house_ids, person_ids, start_date, years, target);
    if plan.is_empty() {
        return Ok(0);
    }

    let mut tx = pool.begin().await?;

    // 1. Bulk INSERT bookings (no calendar IDs yet).
    for chunk in plan.chunks(INSERT_CHUNK) {
        let house_arr: Vec<i64> = chunk.iter().map(|b| b.house_id).collect();
        let person_arr: Vec<i64> = chunk.iter().map(|b| b.person_id).collect();
        let from_arr: Vec<NaiveDate> = chunk.iter().map(|b| b.from_date).collect();
        let to_arr: Vec<NaiveDate> = chunk.iter().map(|b| b.to_date).collect();
        let paid_at_arr: Vec<Option<NaiveDate>> = chunk.iter().map(|b| b.paid_at).collect();
        let total_paid_arr: Vec<Option<Decimal>> = chunk.iter().map(|b| b.total_paid).collect();

        sqlx::query(
            "INSERT INTO bookings (house_id, person_id, from_date, to_date, status, paid_at, total_paid)
             SELECT h, p, f, t, 'Active', pa, tp
             FROM UNNEST($1::bigint[], $2::bigint[], $3::date[], $4::date[], $5::date[], $6::numeric[]) AS u(h, p, f, t, pa, tp)",
        )
        .bind(&house_arr)
        .bind(&person_arr)
        .bind(&from_arr)
        .bind(&to_arr)
        .bind(&paid_at_arr)
        .bind(&total_paid_arr)
        .execute(&mut *tx)
        .await?;
    }

    // 2. Flip every covered calendar entry Rentable -> Rented in one statement.
    //    Filter on Rentable so re-runs are idempotent.
    sqlx::query(
        "UPDATE calendar SET status = 'Rented'
         FROM bookings b
         WHERE calendar.house_id = b.house_id
           AND calendar.date BETWEEN b.from_date AND b.to_date
           AND calendar.status = 'Rentable'",
    )
    .execute(&mut *tx)
    .await?;

    // 3. Back-fill from_calendar_id / to_calendar_id by joining on (house_id, date).
    //    Only touches the bookings we just inserted (rest already have IDs).
    sqlx::query(
        "UPDATE bookings SET
            from_calendar_id = c_from.id,
            to_calendar_id   = c_to.id
         FROM calendar c_from, calendar c_to
         WHERE c_from.house_id = bookings.house_id
           AND c_from.date     = bookings.from_date
           AND c_to.house_id   = bookings.house_id
           AND c_to.date       = bookings.to_date
           AND bookings.from_calendar_id IS NULL",
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(plan.len() as u64)
}

fn plan_bookings(house_ids: &[i64], person_ids: &[i64], start_date: NaiveDate, years: u32, target: usize) -> Vec<PlannedBooking> {
    let mut rng = rand::rng();
    let house_days = years as usize * 365;
    let target_per_house = target.div_ceil(house_ids.len()).max(1);
    // Stride must be > max booking length (14) so successive bookings cannot overlap.
    let stride = (house_days / target_per_house).max(20);

    let mut plan = Vec::with_capacity(target);
    for &house_id in house_ids {
        // Random initial phase per house so seasons don't all line up across houses.
        let initial = rng.random_range(0..stride);
        for i in 0..target_per_house {
            let offset = initial + i * stride;
            let length = rng.random_range(3..=14);
            if offset + length > house_days {
                break;
            }
            let from_date = start_date + Duration::days(offset as i64);
            let to_date = start_date + Duration::days((offset + length - 1) as i64);
            let person_id = *person_ids.choose(&mut rng).expect("non-empty persons");

            // 60% of bookings are paid: paid 7-30 days before the stay, total ≈ length × $150.
            let (paid_at, total_paid) = if rng.random_bool(0.6) {
                let pay_offset = rng.random_range(7..=30) as i64;
                let nightly = rng.random_range(120..=180) as i64;
                (Some(from_date - Duration::days(pay_offset)), Some(Decimal::new(length as i64 * nightly * 100, 2)))
            } else {
                (None, None)
            };

            plan.push(PlannedBooking {
                house_id,
                person_id,
                from_date,
                to_date,
                paid_at,
                total_paid,
            });
        }
    }
    plan
}
