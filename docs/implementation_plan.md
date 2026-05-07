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

Step 15: OpenAPI / Swagger UI (utoipa + utoipa-swagger-ui)

Add `utoipa = "4"` (features: actix_extras) and `utoipa-swagger-ui = "8"` (feature: actix-web) to Cargo.toml.
Derive `#[derive(ToSchema)]` on all model structs (Country, Address, Manager, Person, House, CalendarEntry, Booking and their request types).
Annotate every handler with `#[utoipa::path(...)]`. Build an `ApiDoc` with `#[derive(OpenApi)]` in a new `src/openapi.rs`, register it in `main.rs`, and mount `SwaggerUi` at `/swagger-ui`.

Step 16: Tracing span enrichment (request-id via tracing-actix-web)

Step 17: Input validation (validator crate, #[validate] on request structs)

Step 18: Dockerfile (multi-stage with cargo-chef + distroless)

Step 19: GitHub Actions CI pipeline

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

---

### Step 7 — routes.rs: route registration stub (2026-05-06)

**Implemented:**
- Created `src/routes.rs` with a `configure(cfg: &mut web::ServiceConfig)` function that registers an empty `/api/v1/` scope — the single mount point for all future domain routes.
- Registered `mod routes` in `main.rs` and wired `.configure(routes::configure)` into the `App` builder alongside the existing health and metrics endpoints.
- `/health` stays in `main.rs` (observability, not a domain route); `/metrics` is managed by the Prometheus middleware — neither moves to `routes.rs`.

**Notes:** `cargo build` succeeds with only the expected dead-code warning on `AppError`. The `/api/v1/` scope returns 404 for all paths until domain handlers are added in Phase 2.

**Open issues / reminders:** None. Phase 1 is now complete — the server compiles, connects to Postgres, runs migrations, serves `/health` and `/metrics`, and has a wired `/api/v1/` scope ready for domain slices.

---

### Step 8 — Domain Countries (2026-05-06)

**Implemented:** Full Countries domain slice following the models → repositories → services → handlers → routes pattern:

- `src/models/mod.rs` + `src/models/country.rs` — `Country` (with `sqlx::FromRow` + `Serialize`), `CreateCountryRequest`, `UpdateCountryRequest` (both `Deserialize`)
- `src/repositories/mod.rs` + `src/repositories/country.rs` — five SQLx functions: `find_all`, `find_by_id`, `create`, `update`, `delete`. All use `query_as!` / `query!` macros for compile-time SQL validation. `find_by_id` and `update` map `RowNotFound` to a descriptive `AppError::NotFound`; `delete` checks `rows_affected() == 0` for the same.
- `src/services/mod.rs` + `src/services/country.rs` — thin delegation layer over the repository (consistent with the layered architecture; business logic will grow here in later domains).
- `src/handlers/country.rs` — five actix-web async handler functions wired to `web::Data<AppState>`. `create` returns 201 + `Location` header per the API spec.
- `src/handlers/mod.rs` — added `pub mod country`.
- `src/routes.rs` — replaced the empty scope stub with five routes under `/api/v1/countries`.
- `src/main.rs` — added `mod models`, `mod repositories`, `mod services`.

**Endpoints implemented:**
- `GET /api/v1/countries` → 200 `[Country]`
- `GET /api/v1/countries/{id}` → 200 `Country` (404 if missing)
- `POST /api/v1/countries` → 201 + `Location` header + `Country` body (409 on duplicate `iso_code`)
- `PUT /api/v1/countries/{id}` → 200 `Country` (404 if missing, 409 on unique conflict)
- `DELETE /api/v1/countries/{id}` → 204 (404 if missing, 409 if referenced by an address)

**Notes:** `cargo build` and `cargo clippy` succeed with only the pre-existing dead-code warnings on `AppError` variants not yet used by any handler. The `CHAR(2)` Postgres column for `iso_code` maps cleanly to `String` via sqlx. The FK violation (Postgres code `23503`) on delete is already handled by the existing `AppError::from(sqlx::Error)` impl, which maps it to 409 Conflict.

**Open issues / reminders:** None.

---

### Step 9 — Domain Managers (2026-05-06)

**Implemented:** Full Managers domain slice following the identical models → repositories → services → handlers → routes pattern as Countries:

- `src/models/manager.rs` — `Manager` (with `sqlx::FromRow` + `Serialize`), `CreateManagerRequest`, `UpdateManagerRequest` (both `Deserialize`). Fields: `id`, `first_name`, `last_name`, `email`, `phone`.
- `src/repositories/manager.rs` — five SQLx functions: `find_all` (ordered by `last_name, first_name`), `find_by_id`, `create`, `update`, `delete`. Same `RowNotFound` → `AppError::NotFound` and `rows_affected() == 0` patterns as Countries.
- `src/services/manager.rs` — thin delegation layer over the repository.
- `src/handlers/manager.rs` — five actix-web async handler functions. `create` returns 201 + `Location: /api/v1/managers/{id}` header.
- Updated `src/models/mod.rs`, `src/repositories/mod.rs`, `src/services/mod.rs`, `src/handlers/mod.rs` — added `pub mod manager`.
- `src/routes.rs` — added five routes under `/api/v1/managers` to the existing `/api/v1` scope.

**Endpoints implemented:**
- `GET /api/v1/managers` → 200 `[Manager]`
- `GET /api/v1/managers/{id}` → 200 `Manager` (404 if missing)
- `POST /api/v1/managers` → 201 + `Location` header + `Manager` body (409 on duplicate `email`)
- `PUT /api/v1/managers/{id}` → 200 `Manager` (404 if missing, 409 on unique conflict)
- `DELETE /api/v1/managers/{id}` → 204 (404 if missing, 409 if referenced by a house via FK violation)

**Notes:** `cargo build` and `cargo clippy` succeed with only the pre-existing dead-code warning on unused `AppError` variants. The unique constraint on `email` (Postgres code `23505`) and the FK constraint from `houses.manager_id` (Postgres code `23503`) are both handled by the existing `AppError::from(sqlx::Error)` impl — no extra code needed.

**Open issues / reminders:** None.

---

### Step 10 — Domain Persons (2026-05-06)

**Implemented:** Full Persons domain slice following the identical models → repositories → services → handlers → routes pattern as Managers:

- `src/models/person.rs` — `Person` (with `sqlx::FromRow` + `Serialize`), `CreatePersonRequest`, `UpdatePersonRequest` (both `Deserialize`). Fields: `id`, `first_name`, `last_name`, `email`, `phone`.
- `src/repositories/person.rs` — five SQLx functions: `find_all` (ordered by `last_name, first_name`), `find_by_id`, `create`, `update`, `delete`. Same `RowNotFound` → `AppError::NotFound` and `rows_affected() == 0` patterns as Managers.
- `src/services/person.rs` — thin delegation layer over the repository.
- `src/handlers/person.rs` — five actix-web async handler functions. `create` returns 201 + `Location: /api/v1/persons/{id}` header.
- Updated `src/models/mod.rs`, `src/repositories/mod.rs`, `src/services/mod.rs`, `src/handlers/mod.rs` — added `pub mod person`.
- `src/routes.rs` — added five routes under `/api/v1/persons` to the existing `/api/v1` scope.

**Endpoints implemented:**
- `GET /api/v1/persons` → 200 `[Person]`
- `GET /api/v1/persons/{id}` → 200 `Person` (404 if missing)
- `POST /api/v1/persons` → 201 + `Location` header + `Person` body (409 on duplicate `email`)
- `PUT /api/v1/persons/{id}` → 200 `Person` (404 if missing, 409 on unique conflict)
- `DELETE /api/v1/persons/{id}` → 204 (404 if missing, 409 if referenced by a booking via FK violation)

**Notes:** `cargo build` and `cargo clippy` succeed with only the pre-existing dead-code warning on unused `AppError` variants. The unique constraint on `email` (Postgres code `23505`) and the FK constraint from `bookings.person_id` (Postgres code `23503`) are both handled by the existing `AppError::from(sqlx::Error)` impl — no extra code needed.

**Open issues / reminders:** None.


---

### Step 11 — Domain Addresses (2026-05-06)

**Implemented:** Full Addresses domain slice following the models → repositories → services → handlers → routes pattern:

- `src/models/address.rs` — `Address` (with `Serialize`; embeds `Country` as a nested struct), `CreateAddressRequest`, `UpdateAddressRequest` (both `Deserialize`). Fields: `id`, `street`, `number`, `postcode`, `city`, `province` (optional), `country` (embedded `Country`). The request types carry `country_id` (FK) rather than the full object.
- `src/repositories/address.rs` — four functions: `find_by_id`, `create`, `update`, `delete`. `find_by_id` uses a single JOIN query (`query!` macro) that fetches address and country columns with aliases (`country_id`, `country_name`, `country_iso_code`) and constructs the nested `Address` struct in Rust. `create` inserts with `query_scalar!` to obtain the new `id`, then delegates to `find_by_id` for the full response. `update` uses `execute` with rows_affected check (→ 404), then delegates to `find_by_id`. `delete` follows the same rows_affected pattern as earlier domains.
- `src/services/address.rs` — thin delegation layer (no `list` — the API spec has no list-all endpoint for addresses).
- `src/handlers/address.rs` — four actix-web async handler functions. `create` returns 201 + `Location: /api/v1/addresses/{id}` header.
- Updated `src/models/mod.rs`, `src/repositories/mod.rs`, `src/services/mod.rs`, `src/handlers/mod.rs` — added `pub mod address`.
- `src/routes.rs` — added four routes under `/api/v1/addresses` (no list route, per API spec).

**Endpoints implemented:**
- `GET /api/v1/addresses/{id}` → 200 `Address` with embedded `Country` (404 if missing)
- `POST /api/v1/addresses` → 201 + `Location` header + `Address` body (409 on FK violation for unknown `country_id`)
- `PUT /api/v1/addresses/{id}` → 200 `Address` (404 if missing, 409 on FK violation for unknown `country_id`)
- `DELETE /api/v1/addresses/{id}` → 204 (404 if missing, 409 if referenced by a house via FK violation)

**Notes:** `cargo build` and `cargo clippy` succeed with only the pre-existing dead-code warning. The key difference from prior domains is the embedded `Country` in the response. Rather than using `sqlx::FromRow` with `#[sqlx(flatten)]` (which conflicts on the shared `id` column name), the repository uses a `query!` macro with explicit column aliases and manual struct construction in Rust. The `create`/`update` path uses a two-step write-then-read pattern (INSERT/UPDATE → `find_by_id`) to avoid duplicating the JOIN SQL.

**Open issues / reminders:** None.

---

### Step 12 — Domain Houses (2026-05-06)

**Implemented:** Full Houses domain slice following the models → repositories → services → handlers → routes pattern:

- `src/models/house.rs` — `House` (with `Serialize`; embeds `Address` and `Manager` as nested structs), `CreateHouseRequest`, `UpdateHouseRequest` (both `Deserialize`). Fields: `id`, `name`, `description`, `address` (embedded `Address` with embedded `Country`), `manager` (embedded `Manager`). Request types carry `address_id` and `manager_id` (FKs) rather than full objects.
- `src/repositories/house.rs` — five functions: `find_all`, `find_by_id`, `create`, `update`, `delete`. Both `find_all` and `find_by_id` use a four-table JOIN (`houses → addresses → countries`, `houses → managers`) with explicit column aliases and manual struct construction in Rust (same pattern as Addresses). `find_all` orders by `h.name`. `create` uses `query_scalar!` to get the new `id` then delegates to `find_by_id`. `update` uses `execute` with `rows_affected() == 0` check (→ 404), then delegates to `find_by_id`. `delete` follows the same `rows_affected` pattern.
- `src/services/house.rs` — thin delegation layer over the repository (five functions: `list`, `get`, `create`, `update`, `delete`).
- `src/handlers/house.rs` — five actix-web async handler functions. `create` returns 201 + `Location: /api/v1/houses/{id}` header.
- Updated `src/models/mod.rs`, `src/repositories/mod.rs`, `src/services/mod.rs`, `src/handlers/mod.rs` — added `pub mod house`.
- `src/routes.rs` — added five routes under `/api/v1/houses` to the existing `/api/v1` scope.

**Endpoints implemented:**
- `GET /api/v1/houses` → 200 `[House]` with embedded `Address` (+ `Country`) and `Manager`
- `GET /api/v1/houses/{id}` → 200 `House` (404 if missing)
- `POST /api/v1/houses` → 201 + `Location` header + `House` body (409 on FK violation for unknown `address_id` or `manager_id`)
- `PUT /api/v1/houses/{id}` → 200 `House` (404 if missing, 409 on FK violation)
- `DELETE /api/v1/houses/{id}` → 204 (404 if missing, 409 if referenced by a booking via FK violation)

**Notes:** The key pattern here is the four-table JOIN query shared by `find_all` and `find_by_id`, using column aliases (e.g. `addr_id`, `mgr_first_name`, `country_iso_code`) to avoid name collisions across joined tables. Manual struct construction in Rust is used instead of `sqlx::FromRow` + `#[sqlx(flatten)]`, which conflicts on shared `id` column names. The write-then-read pattern (`INSERT`/`UPDATE` → `find_by_id`) avoids duplicating the JOIN SQL. FK violations for unknown `address_id` or `manager_id` (Postgres code `23503`) and booking references on delete are handled by the existing `AppError::from(sqlx::Error)` impl.

**Open issues / reminders:** None.

---

### Step 13 — Domain Calendar (2026-05-06)

**Implemented:** Full Calendar domain slice as a sub-resource of House:

- `src/models/calendar.rs` — `CalendarEntry` (`id`, `date: NaiveDate`, `status: String`, `price: Decimal`), `CreateCalendarRequest` (`from`, `to`, `status`, `price`), `UpdateCalendarPriceRequest` (`from`, `to`, `price`).
- `src/repositories/calendar.rs` — five functions: `find_all` (with optional `from`/`to` query filters via `$2::date IS NULL OR date >= $2` pattern), `find_by_id` (checks both `id` and `house_id`), `create` (per-day loop with `INSERT ... ON CONFLICT (house_id, date) DO NOTHING RETURNING ...` inside a transaction; `fetch_optional` returns `None` for skipped days), `update_price` (`UPDATE ... RETURNING` for the date range, sorted by date), `delete` (JOIN query against `bookings` to detect active booking overlap before deletion).
- `src/services/calendar.rs` — thin delegation with business-rule validation: `create` rejects `status = 'Rented'` (422) and invalid status values (422); `create`, `update_price`, and `delete` all reject `from > to` (422).
- `src/handlers/calendar.rs` — five actix-web async handler functions; `CalendarListQuery` has optional `from`/`to` for the list endpoint; `CalendarDeleteQuery` has required `from`/`to` as query params for delete; `create` returns 201 + `Location` header + array body.
- Updated `src/models/mod.rs`, `src/repositories/mod.rs`, `src/services/mod.rs`, `src/handlers/mod.rs` — added `pub mod calendar`.
- `src/routes.rs` — added five routes under `/api/v1/houses/{house_id}/calendar`.
- `Cargo.toml` — changed `rust_decimal` feature from `serde-with-str` to `serde-with-float` so that `Decimal` serializes as a JSON number (e.g. `120.0`) as required by the API spec, not as a string.

**Endpoints implemented:**
- `GET /api/v1/houses/{house_id}/calendar?from=&to=` → 200 `[CalendarEntry]` (both query params optional)
- `GET /api/v1/houses/{house_id}/calendar/{id}` → 200 `CalendarEntry` (404 if missing or wrong house)
- `POST /api/v1/houses/{house_id}/calendar` → 201 + `Location` header + `[CalendarEntry]` (only newly created entries; existing days silently skipped; 422 for `Rented` status or invalid status or `from > to`)
- `PATCH /api/v1/houses/{house_id}/calendar` → 200 `[CalendarEntry]` updated entries
- `DELETE /api/v1/houses/{house_id}/calendar?from=&to=` → 204 (409 if any entry in range belongs to a non-cancelled booking)

**Notes:** `cargo build` and `cargo clippy` succeed with only the pre-existing dead-code warning on `ValidationError`. Key patterns: (1) the `ON CONFLICT DO NOTHING RETURNING` + `fetch_optional` trick to identify only newly inserted rows; (2) the `$2::date IS NULL OR date >= $2` pattern for optional date range filtering in a single `query!` macro call; (3) the booking overlap check uses a four-way JOIN (`calendar → bookings → calendar c_from → calendar c_to`) to find calendar entries that fall within the date range of any non-cancelled booking. The `serde-with-float` change in Cargo.toml does not affect any existing domain (Countries, Addresses, Managers, Persons, Houses) since none of them use `Decimal`.

**Open issues / reminders:** None.

---

### Step 14 — Domain Bookings (2026-05-06)

**Implemented:** Full Bookings domain slice — the most complex domain, involving status transitions, calendar entry flipping, and payment recording:

- `src/models/booking.rs` — `BookingHouse` (`id`, `name`), `BookingPerson` (`id`, `first_name`, `last_name`), `Booking` (embeds both; has `expected_total_price: Option<Decimal>` with `#[serde(skip_serializing_if = "Option::is_none")]` so it only appears in the create 201 response), `CreateBookingRequest` (`house_id`, `person_id`, `from`, `to`), `RecordPaymentRequest` (`paid_at`, `total_paid`).
- `src/repositories/booking.rs` — five functions: `find_all` (four-table JOIN with optional `$1::bigint IS NULL OR b.house_id = $1` pattern for house/person filters), `find_by_id` (same JOIN by booking id), `create` (transaction: `FOR UPDATE` lock on calendar range → validate count and all-Rentable → insert booking → flip calendar to Rented; returns `(Booking, expected_total_price)`), `cancel` (transaction: `FOR UPDATE OF b` lock → check not already Cancelled → update booking to Cancelled + clear payment → flip calendar back to Rentable), `record_payment` (check not Cancelled → update paid_at + total_paid).
- `src/services/booking.rs` — thin delegation; `create` validates `from <= to` (422) then merges `expected_total_price` into the returned `Booking`.
- `src/handlers/booking.rs` — five actix-web async handler functions. `BookingListQuery` has optional `house_id` and `person_id` query params. `create` returns 201 + `Location: /api/v1/bookings/{id}`.
- Updated `src/models/mod.rs`, `src/repositories/mod.rs`, `src/services/mod.rs`, `src/handlers/mod.rs` — added `pub mod booking`.
- `src/routes.rs` — added five routes under `/api/v1/bookings`.

**Endpoints implemented:**
- `GET /api/v1/bookings?house_id=&person_id=` → 200 `[Booking]` (both query params optional)
- `GET /api/v1/bookings/{id}` → 200 `Booking` (404 if missing)
- `POST /api/v1/bookings` → 201 + `Location` header + `Booking` body including `expected_total_price` (422 if from > to, 422 if any day missing or not Rentable, 409 on FK violation for unknown house/person)
- `POST /api/v1/bookings/{id}/cancel` → 200 `Booking` (404 if missing, 422 if already Cancelled)
- `POST /api/v1/bookings/{id}/payment` → 200 `Booking` (404 if missing, 422 if Cancelled)

**Notes:** `cargo build` and `cargo clippy` succeed with only the pre-existing dead-code warning on `ValidationError`. Key patterns: (1) the `(Booking, Decimal)` tuple return from `repo::create` lets the service layer attach `expected_total_price` without the repository needing to know about the field; (2) `FOR UPDATE` on the calendar range in `create` prevents race conditions between concurrent booking attempts for overlapping date ranges; (3) `FOR UPDATE OF b` in `cancel` locks only the bookings row, not the joined calendar rows; (4) the `$1::bigint IS NULL OR b.house_id = $1` pattern mirrors the `$2::date IS NULL OR date >= $2` pattern from Calendar for optional filter parameters in a single `query!` call.

**Open issues / reminders:** None. Phase 2 is now complete — all domain vertical slices (Countries, Managers, Persons, Addresses, Houses, Calendar, Bookings) are implemented.

---

### Step 15 — OpenAPI / Swagger UI (2026-05-07)

**Implemented:** Compile-time-generated OpenAPI 3 spec via `utoipa`, served alongside an interactive Swagger UI:

- `Cargo.toml` — added `utoipa = "5"` (features: `actix_extras`, `chrono`, `decimal_float`) and `utoipa-swagger-ui = "9"` (feature: `actix-web`). Bumped from the plan's `utoipa = "4"` / `utoipa-swagger-ui = "8"` to the current stable releases (5.5 / 9.0). The `decimal_float` feature is required so `rust_decimal::Decimal` is rendered as a JSON number, matching our `serde-with-float` choice in Step 13.
- All model structs and enums got `#[derive(ToSchema)]`: `Country`, `CreateCountryRequest`, `UpdateCountryRequest`, `Manager`, `CreateManagerRequest`, `UpdateManagerRequest`, `Person`, `CreatePersonRequest`, `UpdatePersonRequest`, `Address`, `CreateAddressRequest`, `UpdateAddressRequest`, `House`, `CreateHouseRequest`, `UpdateHouseRequest`, `CalendarStatus`, `CalendarEntry`, `CreateCalendarRequest`, `UpdateCalendarPriceRequest`, `BookingStatus`, `BookingHouse`, `BookingPerson`, `Booking`, `CreateBookingRequest`, `RecordPaymentRequest`.
- Every domain handler (and the `/health` handler) was annotated with `#[utoipa::path(...)]` documenting method, path, tag, path/query parameters, request body, and the success + key error response codes (404, 409, 422 as applicable). Tags: `health`, `countries`, `managers`, `persons`, `addresses`, `houses`, `calendar`, `bookings`.
- Created `src/openapi.rs` exposing `pub struct ApiDoc;` with `#[derive(OpenApi)]` listing every annotated path and every schema. Title/description/version are set in the `info(...)` block.
- `src/lib.rs` — registered `pub mod openapi`.
- `src/main.rs` — built `ApiDoc::openapi()` once outside the `HttpServer::new` closure (so it is cloned per worker, not regenerated) and mounted it via `SwaggerUi::new("/swagger-ui/{_:.*}").url("/api-docs/openapi.json", openapi.clone())`.

**Endpoints added:**
- `GET /api-docs/openapi.json` → full OpenAPI 3 JSON document
- `GET /swagger-ui/` (and `/swagger-ui/index.html`) → interactive Swagger UI

**Verified live:** `curl /api-docs/openapi.json` returns 17 paths and 25 component schemas. `curl /swagger-ui/index.html` returns the Swagger UI HTML. `cargo build`, `cargo clippy --all-targets`, and `cargo test --lib --bins` all pass.

**Notes:**
- I used the explicit fully-qualified type form `body = crate::models::xxx::Type` in `#[utoipa::path(...)]` rather than relying on a `use` import in each handler — this keeps the existing `use crate::models::xxx::{...}` lines minimal (only the request types we already imported) and avoids extra imports just for the doc macro.
- `utoipa-swagger-ui` only registers `GET` for the static assets; `HEAD /swagger-ui/` returns 404 by design — not an issue.
- The `actix_extras` feature lets utoipa parse actix-web extractor types in handler signatures, so `web::Data<...>` etc. are automatically excluded from documented parameters.

**Open issues / reminders:** None.