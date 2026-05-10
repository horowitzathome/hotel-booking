use anyhow::{Context, Result};
use chrono::NaiveDate;
use clap::{Args, Parser, Subcommand, ValueEnum};
use sqlx::PgPool;
use std::collections::HashSet;
use std::time::Instant;

mod seed;

#[derive(Parser)]
#[command(name = "seeder", about = "Bulk-load dummy data into the rental DB")]
struct Cli {
    #[command(subcommand)]
    command: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Truncate all data tables and restart identity sequences.
    Reset,
    /// Populate the database with dummy data.
    Load(LoadArgs),
    /// Print row counts per table.
    Verify,
}

#[derive(Args)]
struct LoadArgs {
    /// Use a tiny dataset preset for fast iteration / smoke tests.
    /// Per-table flags still override individual volumes when used together.
    #[arg(long)]
    small: bool,

    #[arg(long)]
    managers: Option<usize>,
    #[arg(long)]
    persons: Option<usize>,
    #[arg(long)]
    addresses: Option<usize>,
    #[arg(long)]
    houses: Option<usize>,

    /// Number of years of calendar entries per house.
    #[arg(long)]
    years: Option<u32>,
    /// Calendar start date (YYYY-MM-DD). Default 2025-01-01 — fixed for reproducibility.
    #[arg(long)]
    start_date: Option<NaiveDate>,
    /// Approximate number of bookings to generate (spread across all houses).
    #[arg(long)]
    bookings: Option<usize>,

    /// Run only these steps (comma-separated). Skipped dependencies are read
    /// from the existing rows in the DB. Mutually exclusive with --skip.
    #[arg(long, value_delimiter = ',', conflicts_with = "skip")]
    only: Vec<Step>,

    /// Skip these steps (comma-separated). Skipped dependencies are read from
    /// the existing rows in the DB.
    #[arg(long, value_delimiter = ',')]
    skip: Vec<Step>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, ValueEnum)]
#[value(rename_all = "lowercase")]
enum Step {
    Countries,
    Managers,
    Persons,
    Addresses,
    Houses,
    Calendar,
    Bookings,
}

const ALL_STEPS: &[Step] = &[Step::Countries, Step::Managers, Step::Persons, Step::Addresses, Step::Houses, Step::Calendar, Step::Bookings];

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "seeder=info,sqlx=warn".into()))
        .init();

    let cli = Cli::parse();
    let database_url = std::env::var("DATABASE_URL").context("DATABASE_URL not set")?;
    let pool = PgPool::connect(&database_url).await.context("failed to connect to database")?;

    match cli.command {
        Cmd::Reset => reset(&pool).await?,
        Cmd::Load(args) => load(&pool, args).await?,
        Cmd::Verify => verify(&pool).await?,
    }

    Ok(())
}

async fn reset(pool: &PgPool) -> Result<()> {
    let start = Instant::now();
    sqlx::query("TRUNCATE bookings, calendar, houses, addresses, managers, persons, countries RESTART IDENTITY CASCADE")
        .execute(pool)
        .await
        .context("failed to truncate tables")?;
    println!("reset all tables in {:.2?}", start.elapsed());
    Ok(())
}

async fn load(pool: &PgPool, args: LoadArgs) -> Result<()> {
    // --- volume resolution: explicit flag > preset default ---
    let (def_managers, def_persons, def_addresses, def_houses, def_years, def_bookings) = if args.small { (10, 100, 50, 50, 1, 200) } else { (1_000, 100_000, 10_000, 10_000, 10, 500_000) };
    let n_managers = args.managers.unwrap_or(def_managers);
    let n_persons = args.persons.unwrap_or(def_persons);
    let n_addresses = args.addresses.unwrap_or(def_addresses);
    let n_houses = args.houses.unwrap_or(def_houses);
    let years = args.years.unwrap_or(def_years);
    let n_bookings = args.bookings.unwrap_or(def_bookings);
    let start_date = args.start_date.unwrap_or_else(|| NaiveDate::from_ymd_opt(2025, 1, 1).expect("valid default start date"));

    // --- step selection ---
    let steps: HashSet<Step> = if !args.only.is_empty() {
        args.only.iter().copied().collect()
    } else {
        ALL_STEPS.iter().copied().filter(|s| !args.skip.contains(s)).collect()
    };

    println!("=== seeding === preset={} steps={:?}", if args.small { "small" } else { "normal" }, sorted_steps(&steps));
    let total = Instant::now();

    let country_ids = run_or_fetch(pool, Step::Countries, &steps, "countries", || seed::countries::seed(pool)).await?;
    let manager_ids = run_or_fetch(pool, Step::Managers, &steps, "managers", || seed::managers::seed(pool, n_managers)).await?;
    let person_ids = run_or_fetch(pool, Step::Persons, &steps, "persons", || seed::persons::seed(pool, n_persons)).await?;
    let address_ids = run_or_fetch(pool, Step::Addresses, &steps, "addresses", || seed::addresses::seed(pool, n_addresses, &country_ids)).await?;
    let house_ids = run_or_fetch(pool, Step::Houses, &steps, "houses", || seed::houses::seed(pool, n_houses, &address_ids, &manager_ids)).await?;

    let calendar_count = if steps.contains(&Step::Calendar) {
        let start = Instant::now();
        let count = seed::calendar::seed(pool, &house_ids, start_date, years).await?;
        let elapsed = start.elapsed();
        let rate = if elapsed.as_secs_f64() > 0.0 { count as f64 / elapsed.as_secs_f64() } else { 0.0 };
        println!(
            "{:>10}: {:>10} rows in {:>8.2?}  ({:>10.0} rows/s)  [{} years from {}]",
            "calendar", count, elapsed, rate, years, start_date
        );
        count
    } else {
        let (c,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM calendar").fetch_one(pool).await?;
        println!("{:>10}: {:>10} rows  (skipped — read from DB)", "calendar", c);
        c as u64
    };

    let bookings_count = if steps.contains(&Step::Bookings) {
        let start = Instant::now();
        let count = seed::bookings::seed(pool, &house_ids, &person_ids, start_date, years, n_bookings).await?;
        let elapsed = start.elapsed();
        let rate = if elapsed.as_secs_f64() > 0.0 { count as f64 / elapsed.as_secs_f64() } else { 0.0 };
        println!("{:>10}: {:>10} rows in {:>8.2?}  ({:>10.0} rows/s)  [target {}]", "bookings", count, elapsed, rate, n_bookings);
        count
    } else {
        let (c,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM bookings").fetch_one(pool).await?;
        println!("{:>10}: {:>10} rows  (skipped — read from DB)", "bookings", c);
        c as u64
    };

    println!("---");
    println!(
        "total: {} rows in {:.2?}",
        country_ids.len() as u64 + manager_ids.len() as u64 + person_ids.len() as u64 + address_ids.len() as u64 + house_ids.len() as u64 + calendar_count + bookings_count,
        total.elapsed()
    );
    Ok(())
}

async fn run_or_fetch<F, Fut>(pool: &PgPool, step: Step, included: &HashSet<Step>, table: &str, f: F) -> Result<Vec<i64>>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<Vec<i64>>>,
{
    if included.contains(&step) {
        let start = Instant::now();
        let ids = f().await?;
        let elapsed = start.elapsed();
        let rate = if elapsed.as_secs_f64() > 0.0 { ids.len() as f64 / elapsed.as_secs_f64() } else { 0.0 };
        println!("{:>10}: {:>10} rows in {:>8.2?}  ({:>10.0} rows/s)", table, ids.len(), elapsed, rate);
        Ok(ids)
    } else {
        let start = Instant::now();
        let ids = fetch_ids(pool, table).await?;
        println!("{:>10}: {:>10} rows  (skipped — read from DB in {:.2?})", table, ids.len(), start.elapsed());
        Ok(ids)
    }
}

async fn fetch_ids(pool: &PgPool, table: &str) -> Result<Vec<i64>> {
    let sql = format!("SELECT id FROM {table} ORDER BY id");
    sqlx::query_scalar(&sql).fetch_all(pool).await.with_context(|| format!("failed to fetch ids from {table}"))
}

fn sorted_steps(steps: &HashSet<Step>) -> Vec<Step> {
    ALL_STEPS.iter().copied().filter(|s| steps.contains(s)).collect()
}

async fn verify(pool: &PgPool) -> Result<()> {
    let tables = ["countries", "addresses", "managers", "persons", "houses", "calendar", "bookings"];
    println!("=== row counts ===");
    for t in tables {
        let sql = format!("SELECT COUNT(*) FROM {t}");
        let (count,): (i64,) = sqlx::query_as(&sql).fetch_one(pool).await?;
        println!("{:>10}: {:>12}", t, count);
    }
    Ok(())
}
