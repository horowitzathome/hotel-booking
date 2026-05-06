# This file contains the steps to code the program

## Phase 1 — Project Skeleton

Step 1: Cargo.toml — add all crates

Add actix-web, actix-web-prom, tracing, tracing-subscriber (with JSON feature), tracing-actix-web, dotenvy, serde/serde_json, thiserror, anyhow, validator, chrono, rust_decimal. sqlx and tokio are already there.

Step 2: config.rs — typed config from env

A single AppConfig struct covering database URL, pool sizes, server host/port, app name/env. Loaded once at startup via dotenvy + environment reads.

Step 3: main.rs — actix server + wiring

Async main, load config, init tracing, build DB pool, wire AppState, start actix-web HttpServer.

Step 4: db.rs — SQLx pool

Pool setup with connect_timeout, min/max connections from config. Run sqlx::migrate!() on startup.

Step 5: errors.rs — AppError enum

Variants for NotFound, Conflict, UnprocessableEntity, ValidationError, Internal. Implements actix ResponseError → maps to 404/409/422/400/500 with the standard JSON body.

Step 6: /health + /metrics endpoints

Health handler inline in main or a small handlers/health.rs. Prometheus metrics via actix-web-prom middleware.

Step 7: routes.rs — route registration stub

Empty scope for /api/v1/ wired in; health and metrics attached.

## Phase 2 — Domain vertical slices (one at a time, in dependency order)

Each slice = models/ → repositories/ → services/ → handlers/ → wire into routes.rs.

Step 8: Domain Countries: Simplest — no FK deps

Step 9:	Domain Managers: 

Standalone

Step 10: Domain Persons: 

Standalone

Step 11: Domain Addresses: 

Needs countries embedded in response

Step 12	Domain Houses: 

Needs address + manager embedded

Step 13: Calendar: 

Sub-resource of house; skip-existing logic

Step 14: Bookings:

Most complex: status transitions, calendar flip, payment

## Phase 3 — Cross-cutting polish

Step Step 15: Tracing span enrichment (request-id via tracing-actix-web)

Step 16: Input validation (validator crate, #[validate] on request structs)

Step 17: Dockerfile (multi-stage with cargo-chef + distroless)

Step 18: GitHub Actions CI pipeline

## Suggested starting point

Steps 1–7 give you a compiling, running actix server with health, metrics, structured JSON logs, and a working DB connection — a solid foundation before any domain code. 

## Documentation of Implementation Sessions

Here a list of completed steps. For each step this is listed: 

- a summary what has been implemented
- comments or notes which are important
- reminders or open issues or points not to forget or for a later rework 

---

### Step 1 — Cargo.toml: add all crates (2026-05-06)

**Implemented:** Added all required dependencies to `Cargo.toml`:
- `actix-web = "4"` — HTTP server
- `actix-web-prom = "0.9"` — Prometheus metrics middleware
- `tracing = "0.1"`, `tracing-subscriber = "0.3"` (features: env-filter, json), `tracing-actix-web = "0.7"` — structured logging + request tracing
- `serde = "1"` (derive), `serde_json = "1"` — JSON serialization
- `thiserror = "2"`, `anyhow = "1"` — error handling
- `validator = "0.19"` (derive) — input validation
- `dotenvy = "0.15"` — .env loading
- `chrono = "0.4"` (serde), `rust_decimal = "1"` (serde-with-str) — domain types
- `sqlx = "0.8"` and `tokio = "1"` were already present

**Notes:** `cargo build` succeeds cleanly. The IDE reported a spell-check warning on "chrono" — not a real error, safely ignored.

**Open issues / reminders:** None.

---

### Step 2 — config.rs: typed config from env (2026-05-06)

**Implemented:** Created `src/config.rs` with `AppConfig` struct and `from_env()` constructor:
- Calls `dotenvy::dotenv().ok()` to load `.env` if present (silently skips if absent, for production)
- Three private helpers: `required()` (hard error if missing), `optional()` (falls back to default), `parse::<T>()` (parses typed values with fallback default)
- Fields: `database_url`, `database_max_connections` (default 5), `database_min_connections` (default 1), `database_connect_timeout_secs` (default 5), `server_host` (default `0.0.0.0`), `server_port` (default 8080), `app_name`, `app_env`
- Helper methods: `server_addr()` → `"host:port"` string, `is_development()` → bool

**Notes:** `cargo build` succeeds with only expected dead-code warnings (nothing uses config yet). `dotenvy::dotenv().ok()` is intentionally fallible — in production env vars are injected by Kubernetes without a `.env` file.

**Open issues / reminders:** None.

---

### Step 3 — main.rs: actix server + wiring (2026-05-06)

**Implemented:**
- `src/main.rs`: `#[actix_web::main]` async entry point; loads `AppConfig`, initialises tracing, creates the DB pool, wires `AppState` into actix-web `Data`, starts `HttpServer` with `TracingLogger` middleware.
- `src/db.rs` (stub): `create_pool()` builds a `PgPool` via `PgPoolOptions` using the config values; migration call is left for Step 4.
- `init_tracing()`: in development mode uses human-readable `fmt` output; in production uses `fmt().json()`. Filter defaults to `info,sqlx=warn,actix_web=info` if `RUST_LOG` is not set.
- `AppState { pool: sqlx::PgPool }` defined in `main.rs`; passed as `web::Data` (actix wraps it in `Arc` internally — no manual `Arc` needed).

**Notes:** Verified with a live run: server connects to Postgres, starts 8 workers on `0.0.0.0:8080`, and shuts down gracefully on SIGTERM. Structured log fields (`app`, `env`, `address`, `max_connections`) are visible in output.

**Open issues / reminders:**
- Routes scope (`/api/v1/`) and health/metrics endpoints are Steps 5–7.

---

### Step 4 — db.rs: SQLx pool + migrations (2026-05-06)

**Implemented:** Extended `src/db.rs` with `sqlx::migrate!("./migrations").run(&pool)` after pool creation. The macro embeds all SQL files from `migrations/` at compile time; `.run()` applies any not-yet-applied migrations at startup using sqlx's built-in `_sqlx_migrations` tracking table. Added `anyhow::Context` for descriptive error messages on both connection failure and migration failure.

**Notes:** Verified with live run — log shows `database migrations applied` immediately after pool creation, confirming the schema was already applied and sqlx correctly detected nothing new to run (idempotent). Migrations run before the server starts accepting requests, so the app never starts with a stale schema.

**Open issues / reminders:** None.

---

### Step 5 — errors.rs: AppError enum (2026-05-06)

**Implemented:** Created `src/errors.rs` and registered it as `mod errors` in `main.rs`:
- `AppError` enum with five variants: `NotFound(String)`, `Conflict(String)`, `UnprocessableEntity(String)`, `ValidationError(Vec<String>)`, `Internal(#[source] anyhow::Error)`
- `ResponseError` impl maps variants to HTTP status codes: 404 / 409 / 422 / 400 / 500
- `error_response()` produces the standard JSON body `{"error": "...", "details": [...]}` per the API spec; `details` is only included for `ValidationError`
- `Internal` errors log the full error with `tracing::error!` but return a generic `"internal server error"` message to clients — no internal details leak to the user
- `From<sqlx::Error>`: maps `RowNotFound` → `NotFound`, Postgres code `23505` (unique violation) and `23503` (FK violation) → `Conflict`, all others → `Internal`
- `From<anyhow::Error>`: wraps into `Internal`

**Notes:** `cargo build` succeeds with a single expected dead-code warning (`AppError` unused until handlers are added). The approach matches the actix-web `ResponseError` trait contract — implementing only `status_code` and `error_response` is sufficient.

**Open issues / reminders:** None.

---

### Step 6 — /health + /metrics endpoints (2026-05-06)

**Implemented:**
- Created `src/handlers/mod.rs` and `src/handlers/health.rs` — `health()` async handler returns `{ "status": "ok" }` with HTTP 200.
- Added `PrometheusMetricsBuilder::new("api").endpoint("/metrics")` in `main.rs`; the resulting middleware is cloned into each actix worker via `App::wrap(prometheus.clone())`.
- `TracingLogger` is placed *after* the Prometheus middleware so request traces include the metric collection overhead.
- `GET /health` is wired directly in `main.rs` for now; domain routes will move to `routes.rs` in Step 7.

**Notes:** Verified with a live run:
- `curl /health` → `{"status":"ok"}`
- `curl /metrics` → full Prometheus histogram/counter output (endpoint, method, status labels).
- `cargo build` succeeds with only the expected dead-code warning on `AppError` (no handlers yet).

**Open issues / reminders:**
- Step 7 will introduce `routes.rs`; the `/health` route registration can stay in `main.rs` (it is an observability endpoint, not a domain route) or move to `routes.rs` — decide then.

