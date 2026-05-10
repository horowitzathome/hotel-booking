use chrono::{Duration, NaiveDate};
use goose::prelude::*;
use rand::Rng;
use rand::seq::IndexedRandom;
use serde::Deserialize;
use serde::de::DeserializeOwned;
use std::collections::HashSet;
use std::sync::OnceLock;

// ---------------------------------------------------------------------------
// Fixtures: fetched once during test_start, shared across every goose user.
// All measured scenarios pick from this pool — no bulk-list calls happen
// while the metrics clock is running.
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct IdOnly {
    id: i64,
}

#[derive(Deserialize)]
struct BookingForFixtures {
    id: i64,
    person: IdOnly,
}

struct Fixtures {
    house_ids: Vec<i64>,
    manager_ids: Vec<i64>,
    person_ids: Vec<i64>,
    booking_ids: Vec<i64>,
}

static FIXTURES: OnceLock<Fixtures> = OnceLock::new();

fn fixtures() -> &'static Fixtures {
    FIXTURES.get().expect("fixtures must be populated by test_start before scenarios run")
}

async fn get_json<T: DeserializeOwned>(user: &mut GooseUser, path: &str) -> Result<T, Box<TransactionError>> {
    let response = user.get(path).await?.response.map_err(|e| Box::new(e.into()))?;
    response.json::<T>().await.map_err(|e| Box::new(e.into()))
}

async fn fetch_fixtures(user: &mut GooseUser) -> TransactionResult {
    println!("[loadtest] fetching fixtures (off the metrics clock)...");

    let house_ids: Vec<i64> = get_json::<Vec<IdOnly>>(user, "/api/v1/houses").await?.into_iter().map(|h| h.id).collect();
    let manager_ids: Vec<i64> = get_json::<Vec<IdOnly>>(user, "/api/v1/managers").await?.into_iter().map(|m| m.id).collect();

    // Sample bookings via 50 random houses — gives us booking IDs and
    // (transitively) a varied pool of person IDs without ever calling
    // GET /api/v1/persons or GET /api/v1/bookings (which would return 100k+ rows).
    let sample_size = house_ids.len().min(50);
    let sample: Vec<i64> = house_ids.choose_multiple(&mut rand::rng(), sample_size).copied().collect();

    let mut booking_ids = Vec::new();
    let mut person_set: HashSet<i64> = HashSet::new();
    for h in sample {
        let bookings: Vec<BookingForFixtures> = get_json(user, &format!("/api/v1/bookings?house_id={h}")).await?;
        for b in bookings {
            booking_ids.push(b.id);
            person_set.insert(b.person.id);
        }
    }
    let person_ids: Vec<i64> = person_set.into_iter().collect();

    println!(
        "[loadtest] fixtures ready: {} houses, {} managers, {} persons, {} bookings",
        house_ids.len(),
        manager_ids.len(),
        person_ids.len(),
        booking_ids.len()
    );

    FIXTURES
        .set(Fixtures {
            house_ids,
            manager_ids,
            person_ids,
            booking_ids,
        })
        .map_err(|_| ())
        .expect("fixtures already initialized — test_start ran twice?");

    Ok(())
}

// ---------------------------------------------------------------------------
// Read scenarios (Pass A). Weights add to 95 — goose normalizes regardless.
// ---------------------------------------------------------------------------

async fn view_house(user: &mut GooseUser) -> TransactionResult {
    let id = pick(&fixtures().house_ids);
    user.get(&format!("/api/v1/houses/{id}")).await?;
    Ok(())
}

async fn view_calendar(user: &mut GooseUser) -> TransactionResult {
    let id = pick(&fixtures().house_ids);
    // Calendar covers 2025-01-01 .. 2034-12-29; pick a random ~30-day window inside.
    // Scope the rng so ThreadRng (!Send) is dropped before the await below.
    let (from, to) = {
        let mut rng = rand::rng();
        let year = rng.random_range(2025..=2034);
        let month = rng.random_range(1..=12);
        let from = NaiveDate::from_ymd_opt(year, month, 1).expect("valid first-of-month");
        (from, from + Duration::days(29))
    };
    user.get(&format!("/api/v1/houses/{id}/calendar?from={from}&to={to}")).await?;
    Ok(())
}

async fn view_bookings_by_house(user: &mut GooseUser) -> TransactionResult {
    let id = pick(&fixtures().house_ids);
    user.get(&format!("/api/v1/bookings?house_id={id}")).await?;
    Ok(())
}

async fn view_bookings_by_person(user: &mut GooseUser) -> TransactionResult {
    let id = pick(&fixtures().person_ids);
    user.get(&format!("/api/v1/bookings?person_id={id}")).await?;
    Ok(())
}

async fn view_booking_detail(user: &mut GooseUser) -> TransactionResult {
    let id = pick(&fixtures().booking_ids);
    user.get(&format!("/api/v1/bookings/{id}")).await?;
    Ok(())
}

async fn view_manager(user: &mut GooseUser) -> TransactionResult {
    let id = pick(&fixtures().manager_ids);
    user.get(&format!("/api/v1/managers/{id}")).await?;
    Ok(())
}

async fn list_countries(user: &mut GooseUser) -> TransactionResult {
    user.get("/api/v1/countries").await?;
    Ok(())
}

fn pick(pool: &[i64]) -> i64 {
    *pool.choose(&mut rand::rng()).expect("fixture pool was empty")
}

#[tokio::main]
async fn main() -> Result<(), GooseError> {
    GooseAttack::initialize()?
        .test_start(transaction!(fetch_fixtures))
        .register_scenario(
            scenario!("Browse")
                .register_transaction(transaction!(view_house).set_name("view_house").set_weight(25)?)
                .register_transaction(transaction!(view_calendar).set_name("view_calendar").set_weight(20)?)
                .register_transaction(transaction!(view_bookings_by_person).set_name("view_bookings_by_person").set_weight(15)?)
                .register_transaction(transaction!(view_bookings_by_house).set_name("view_bookings_by_house").set_weight(10)?)
                .register_transaction(transaction!(view_booking_detail).set_name("view_booking_detail").set_weight(10)?)
                .register_transaction(transaction!(view_manager).set_name("view_manager").set_weight(8)?)
                .register_transaction(transaction!(list_countries).set_name("list_countries").set_weight(7)?),
        )
        .set_default(GooseDefault::Host, "http://localhost:8080")?
        .execute()
        .await?;
    Ok(())
}
