use chrono::{Duration, NaiveDate};
use goose::prelude::*;
use rand::RngExt;
use rand::seq::IndexedRandom;
use serde::Deserialize;
use serde::de::DeserializeOwned;
use std::collections::HashSet;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicUsize, Ordering};

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
    status: String,
    paid_at: Option<NaiveDate>,
}

#[derive(Deserialize)]
struct CalendarEntryForFixtures {
    date: NaiveDate,
    status: String,
}

struct FreeWindow {
    house_id: i64,
    from: NaiveDate,
    to: NaiveDate,
}

struct Fixtures {
    house_ids: Vec<i64>,
    manager_ids: Vec<i64>,
    person_ids: Vec<i64>,
    booking_ids: Vec<i64>,
    /// Active bookings with no payment recorded — write target for record_payment.
    /// Each goose worker grabs the next unused index via PAYMENT_CURSOR; never collides.
    unpaid_booking_ids: Vec<i64>,
    /// Pre-discovered (house, from, to) windows on currently-Rentable days —
    /// write target for create_booking. Same cursor pattern via CREATE_CURSOR.
    free_windows: Vec<FreeWindow>,
}

// Global cursors that hand out fresh fixture indices to the write scenarios.
// Each `fetch_add` is atomic, so even though many goose workers run concurrently
// no two of them ever pay/book the same target.
static PAYMENT_CURSOR: AtomicUsize = AtomicUsize::new(0);
static CREATE_CURSOR: AtomicUsize = AtomicUsize::new(0);

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
    let mut unpaid_booking_ids = Vec::new();
    let mut person_set: HashSet<i64> = HashSet::new();
    for h in sample {
        let bookings: Vec<BookingForFixtures> = get_json(user, &format!("/api/v1/bookings?house_id={h}")).await?;
        for b in bookings {
            booking_ids.push(b.id);
            person_set.insert(b.person.id);
            if b.status == "Active" && b.paid_at.is_none() {
                unpaid_booking_ids.push(b.id);
            }
        }
    }
    let person_ids: Vec<i64> = person_set.into_iter().collect();

    // Pre-compute free (Rentable) date windows on a sample of houses, used by
    // create_booking. One full year per house yields plenty of 3-day windows.
    let cal_sample_size = house_ids.len().min(100);
    let cal_sample: Vec<i64> = house_ids.choose_multiple(&mut rand::rng(), cal_sample_size).copied().collect();
    let mut free_windows: Vec<FreeWindow> = Vec::new();
    for h in cal_sample {
        let entries: Vec<CalendarEntryForFixtures> = get_json(user, &format!("/api/v1/houses/{h}/calendar?from=2025-01-01&to=2025-12-31")).await?;
        extract_free_windows(h, &entries, &mut free_windows);
    }

    println!(
        "[loadtest] fixtures ready: {} houses, {} managers, {} persons, {} bookings ({} unpaid), {} free windows",
        house_ids.len(),
        manager_ids.len(),
        person_ids.len(),
        booking_ids.len(),
        unpaid_booking_ids.len(),
        free_windows.len()
    );

    FIXTURES
        .set(Fixtures {
            house_ids,
            manager_ids,
            person_ids,
            booking_ids,
            unpaid_booking_ids,
            free_windows,
        })
        .map_err(|_| ())
        .expect("fixtures already initialized — test_start ran twice?");

    Ok(())
}

/// Walk a single house's calendar entries in date order, find every contiguous
/// run of Rentable days of length ≥ 3, and emit non-overlapping 3-day windows
/// from each run.
fn extract_free_windows(house_id: i64, entries: &[CalendarEntryForFixtures], out: &mut Vec<FreeWindow>) {
    const WIN: usize = 3;
    let mut sorted: Vec<&CalendarEntryForFixtures> = entries.iter().filter(|e| e.status == "Rentable").collect();
    sorted.sort_by_key(|e| e.date);

    let mut run: Vec<NaiveDate> = Vec::new();
    let mut last: Option<NaiveDate> = None;
    for e in sorted {
        let contiguous = matches!(last, Some(prev) if (e.date - prev).num_days() == 1);
        if !contiguous {
            flush_run(house_id, &run, WIN, out);
            run.clear();
        }
        run.push(e.date);
        last = Some(e.date);
    }
    flush_run(house_id, &run, WIN, out);
}

fn flush_run(house_id: i64, run: &[NaiveDate], win: usize, out: &mut Vec<FreeWindow>) {
    let mut i = 0;
    while i + win <= run.len() {
        out.push(FreeWindow {
            house_id,
            from: run[i],
            to: run[i + win - 1],
        });
        i += win;
    }
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

// ---------------------------------------------------------------------------
// Write scenarios (Pass B). Targets come from atomic-cursor pools so two goose
// users never mutate the same row, even under heavy concurrency. Once a pool
// is exhausted the scenario silently no-ops — better than logging spurious
// 422s against an already-paid / already-Rented target.
// ---------------------------------------------------------------------------

async fn record_payment(user: &mut GooseUser) -> TransactionResult {
    let pool = &fixtures().unpaid_booking_ids;
    let idx = PAYMENT_CURSOR.fetch_add(1, Ordering::Relaxed);
    if idx >= pool.len() {
        return Ok(());
    }
    let booking_id = pool[idx];

    let body = serde_json::json!({
        "paid_at": "2025-06-15",
        "total_paid": 750.00,
    });
    let path = format!("/api/v1/bookings/{booking_id}/payment");
    let req_builder = user.get_request_builder(&GooseMethod::Post, &path)?.json(&body);
    let goose_request = GooseRequest::builder().method(GooseMethod::Post).name("record_payment").set_request_builder(req_builder).build();
    user.request(goose_request).await?;
    Ok(())
}

async fn create_booking(user: &mut GooseUser) -> TransactionResult {
    let f = fixtures();
    let pool = &f.free_windows;
    let idx = CREATE_CURSOR.fetch_add(1, Ordering::Relaxed);
    if idx >= pool.len() {
        return Ok(());
    }
    let win = &pool[idx];
    let person_id = pick(&f.person_ids);

    let body = serde_json::json!({
        "house_id": win.house_id,
        "person_id": person_id,
        "from": win.from,
        "to": win.to,
    });
    let req_builder = user.get_request_builder(&GooseMethod::Post, "/api/v1/bookings")?.json(&body);
    let goose_request = GooseRequest::builder().method(GooseMethod::Post).name("create_booking").set_request_builder(req_builder).build();
    user.request(goose_request).await?;
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
                .register_transaction(transaction!(list_countries).set_name("list_countries").set_weight(7)?)
                .register_transaction(transaction!(record_payment).set_name("record_payment").set_weight(3)?)
                .register_transaction(transaction!(create_booking).set_name("create_booking").set_weight(2)?),
        )
        .set_default(GooseDefault::Host, "http://localhost:8080")?
        .execute()
        .await?;
    Ok(())
}
