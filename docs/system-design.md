# System Design Document: hotel-booking

This document describes HOW the hotel-booking REST API is implemented: module structure, types, patterns, data flows, and operational guardrails. It is the architectural counterpart to [`prd.md`](prd.md), which defines WHAT the system does.

The project is a Rust REST API for villa rental management. It serves as a learning artefact for Java/Spring Boot developers, so the architecture deliberately follows the same layered shape (`@RestController` / `@Service` / `@Repository`) familiar from Spring, expressed in idiomatic Rust. Section 2 (Tech Stack) lists each Rust crate alongside its Spring Boot equivalent.

## 1. Architecture Overview

The application is a single actix-web HTTP service backed by PostgreSQL. It follows a flat, three-layer architecture: **handlers → services → repositories**. There is no IoC container; dependencies are wired explicitly in `main.rs` and propagated as actix-web `AppState`.

```text
HTTP Request
     │
     ▼
┌─────────────┐     handlers/            actix-web functions; JSON in/out;
│  Handlers   │                          field-shape validation; HTTP status mapping
└──────┬──────┘
       │
       ▼
┌─────────────┐     services/            pure async functions; business rules;
│  Services   │                          422 errors; no SQL, no HTTP
└──────┬──────┘
       │
       ▼
┌─────────────┐     repositories/        sqlx::query! macros against Postgres;
│ Repositories│                          transactions; row locking for booking writes
└──────┬──────┘
       │
       ▼
   PostgreSQL
```

The crate is published as a workspace member (`crates/rental-api`) inside a Cargo workspace that also hosts `seeder` (data fixtures) and `loadtest` (k6-style write/read profiles). Only `rental-api` is described here.

### 1.1 Design Principles

| Principle | Rule | Rationale |
|-----------|------|-----------|
| **Ownership by default** | Service and repository functions take borrowed `&PgPool` and `&Request`; return owned response types. No `Arc<Mutex<T>>`. | Aligns with Rust's ownership model; no shared-mutable-state bugs. |
| **Stateless mappers at boundaries** | Repositories map `sqlx` rows to model structs inline (one place, no shared mapper trait). Mappers are pure: row in, struct out. | Anti-corruption layer between DB rows and response types; trivially testable. |
| **No domain framework annotations beyond the request shape** | Response types carry only `Serialize` + `ToSchema`. Request types carry `Deserialize` + `ToSchema` + `Validate`. Database-row types use `sqlx::Type` only on enums whose names must match PG enum types. | Keeps response/request structs honest about their boundary role; no JPA-style entity-everywhere coupling. |
| **Idiomatic Rust** | `?` for error propagation, iterator chains over `for`, `if let`/`while let` for one-arm matching, `From`/`Into` for cross-layer error conversion. | Reduces boilerplate; explicit intent. |
| **Type-driven development** | Enums (`CalendarStatus`, `BookingStatus`) mirror Postgres enum types via `sqlx::Type`. `Option<T>` everywhere absence is allowed. `Decimal` (not `f64`) for prices. `NaiveDate` (not `String`) for dates. | Pushes correctness into the type system. No stringly-typed status values reach the service layer. |
| **Write-then-read** | Create/update operations perform the mutation and then call `find_by_id` to load the full embedded response shape. | The JOIN SQL that builds an embedded response lives in exactly one place. |
| **No IoC container** | Dependencies are wired in `main.rs` and propagated through `web::Data<AppState>`. | Resolved at compile time; no runtime reflection. |

These principles apply equally to production code and tests.

## 2. Tech Stack

| Component | Crate | Version | Spring Boot equivalent |
|-----------|-------|---------|------------------------|
| Language | Rust | stable (1.87+) | Java 21 |
| HTTP server | `actix-web` | 4 | Spring MVC |
| Async runtime | `tokio` | 1 (full features) | Virtual threads / Reactor |
| Database driver | `sqlx` | 0.8 (postgres, chrono, rust_decimal, migrate, runtime-tokio-rustls) | Spring Data JPA + JDBC |
| JSON | `serde` + `serde_json` | 1 | Jackson |
| Validation | `validator` | 0.19 (derive) | Bean Validation `@Valid` |
| Error handling | `thiserror` + `anyhow` | 2 / 1 | `@ControllerAdvice` |
| Tracing | `tracing` + `tracing-actix-web` | 0.1 / 0.7 | Spring Sleuth / Micrometer Tracing |
| Distributed tracing | `opentelemetry` + `opentelemetry-otlp` + `tracing-opentelemetry` | 0.31 / 0.31 / 0.32 | OpenTelemetry SDK |
| Logging | `tracing-subscriber` | 0.3 (env-filter, json) | Logback / Log4j2 |
| Metrics | `actix-web-prom` | 0.9 | Spring Boot Actuator + Micrometer |
| Decimal / dates | `rust_decimal`, `chrono` | 1 / 0.4 | `BigDecimal`, `LocalDate` |
| OpenAPI / Swagger | `utoipa` + `utoipa-swagger-ui` | 5 / 9 | springdoc-openapi |
| Config | `dotenvy` | 0.15 | `application.properties` / `@Value` |
| Security audit | `cargo-audit` | latest | OWASP dependency-check |

Release profile is size-optimised: `opt-level = "z"`, `lto = true`, `codegen-units = 1`, `panic = "abort"`, `strip = true`.

## 3. Module Structure

The project is a Cargo workspace. The HTTP service lives in `crates/rental-api/`. All paths below are relative to that crate.

```text
crates/rental-api/
├── Cargo.toml
└── src/
    ├── main.rs              entry point: load config, init tracing, build pool,
    │                        register routes, mount Swagger UI, start HttpServer
    ├── lib.rs               pub mod declarations; defines `AppState { pool: PgPool }`
    ├── config.rs            AppConfig::from_env(); reads DATABASE_URL, SERVER_*, APP_*
    ├── db.rs                create_pool(): PgPoolOptions + run sqlx::migrate! at startup
    ├── errors.rs            AppError enum + ResponseError impl + From conversions
    ├── routes.rs            actix-web route table (one place, all routes)
    ├── openapi.rs           utoipa ApiDoc derive listing every handler path and schema
    ├── handlers/            HTTP layer (= @RestController)
    │   ├── mod.rs
    │   ├── health.rs        GET /health
    │   ├── country.rs
    │   ├── address.rs
    │   ├── manager.rs
    │   ├── person.rs
    │   ├── house.rs
    │   ├── calendar.rs
    │   └── booking.rs
    ├── services/            business logic (= @Service)
    │   ├── mod.rs
    │   ├── country.rs
    │   ├── address.rs
    │   ├── manager.rs
    │   ├── person.rs
    │   ├── house.rs
    │   ├── calendar.rs      business-rule validation: from <= to, status != Rented
    │   └── booking.rs       business-rule validation: from <= to
    ├── repositories/        database access (= @Repository, sqlx queries)
    │   ├── mod.rs
    │   ├── country.rs
    │   ├── address.rs
    │   ├── manager.rs
    │   ├── person.rs
    │   ├── house.rs
    │   ├── calendar.rs
    │   └── booking.rs       transactions + FOR UPDATE row lock for create/cancel
    └── models/              request, response, and DB-row types (= @Entity + records)
        ├── mod.rs
        ├── country.rs
        ├── address.rs
        ├── manager.rs
        ├── person.rs
        ├── house.rs
        ├── calendar.rs      CalendarStatus enum (sqlx::Type, type_name "calendar_status")
        └── booking.rs       BookingStatus enum (sqlx::Type, type_name "booking_status")
```

Adjacent directories at the crate root:

- `migrations/` (at repo root) — `sqlx::migrate!` source. Contains `0001_initial_schema.{up,down}.sql`.
- `.sqlx/` (at repo root) — generated offline metadata for `sqlx::query!` macros. Required for builds without a live database (CI, Docker). Regenerated via `cargo sqlx prepare --workspace`.
- `crates/rental-api/tests/` — integration tests using `#[sqlx::test(migrations = "../../migrations")]`, which provisions a fresh database per test.

### 3.1 Dependency Direction

`handlers` → `services` → `repositories` → `sqlx::PgPool`. Models are leaf types referenced from all three layers. There are no upward dependencies. The HTTP framework (`actix-web`) is referenced only in `handlers/`, `errors.rs`, and `main.rs`.

### 3.2 Naming Conventions

| Concept | Convention | Example |
|---------|-----------|---------|
| Response type | Bare noun, `Serialize + ToSchema` | `House`, `Booking`, `CalendarEntry` |
| Create request | `Create{Entity}Request`, `Deserialize + ToSchema + Validate` | `CreateBookingRequest` |
| Update request | `Update{Entity}Request` | `UpdateHouseRequest` |
| Action request | `{Action}{Entity}Request` | `RecordPaymentRequest` |
| Query-string struct | `{Resource}{Action}Query` (local to handler file) | `BookingListQuery`, `CalendarDeleteQuery` |
| Pagination params | `models::pagination::PaginationQuery` — shared across all list handlers | `limit: Option<i64>`, `offset: Option<i64>` |
| Status enum | `{Entity}Status`, `sqlx::Type` with `type_name = "snake_case"` matching the PG enum | `BookingStatus`, `CalendarStatus` |
| Function names | Snake-case verbs: `list`, `get`, `create`, `update`, `delete`, `cancel`, `record_payment` | |
| Module aliases | `use crate::services::{x} as svc;` and `use crate::repositories::{x} as repo;` inside handlers and services respectively | |

**Prohibited:** `Manager`, `Helper`, `Utility`, `Handler` (as a type suffix), `Processor`, `Base`, `Info`, `Data` (as a type suffix). The `Handler` directory name is allowed for HTTP entry points; types inside are functions, not structs.

## 4. Domain Model

### 4.1 Response and Request Types

All public types are plain `Debug + Serialize`-or-`Deserialize` structs. They derive `utoipa::ToSchema` so they appear in the OpenAPI document at `/api-docs/openapi.json`.

| Entity | Response struct | Embedded fields |
|--------|-----------------|------------------|
| `Country` | `id`, `name`, `iso_code` (CHAR(2), see [ADR 2026-05-06](adr/2026-05-06-country-codes-iso-3166-1-alpha-2.md)) | — |
| `Address` | `id`, `street`, `number`, `postcode`, `city`, `province: Option<String>`, `country: Country` | full `Country` |
| `Manager` | `id`, `first_name`, `last_name`, `email`, `phone` | — |
| `Person` | `id`, `first_name`, `last_name`, `email`, `phone` | — |
| `House` | `id`, `name`, `description`, `address: Address`, `manager: Manager` | full `Address` (with `Country`) and full `Manager` |
| `CalendarEntry` | `id`, `date: NaiveDate`, `status: CalendarStatus`, `price: Decimal` | — |
| `Booking` | `id`, `house: BookingHouse`, `person: BookingPerson`, `from`, `to`, `status`, `expected_total_price: Option<Decimal>`, `paid_at: Option<NaiveDate>`, `total_paid: Option<Decimal>` | summary `BookingHouse { id, name }` and `BookingPerson { id, first_name, last_name }` |

`Booking.expected_total_price` is serialized with `skip_serializing_if = "Option::is_none"` — it appears only in the 201 response of `POST /bookings` (computed by the repository, attached by the service), never persisted, never returned by `GET /bookings/{id}` (REQ-BK-002, edge case 8).

### 4.2 Enums

```rust
#[derive(..., sqlx::Type)]
#[sqlx(type_name = "calendar_status")]
pub enum CalendarStatus { NotRentable, Rentable, Rented }

#[derive(..., sqlx::Type)]
#[sqlx(type_name = "booking_status")]
pub enum BookingStatus { Active, Cancelled }
```

These mirror Postgres enum types declared in `migrations/0001_initial_schema.up.sql`. The `sqlx::Type` derive makes them usable as bind parameters and `query!` return types without manual conversion.

### 4.3 Identifiers

Identifiers are `i64` everywhere (`BIGSERIAL` in Postgres). No newtype wrapper is currently introduced — the trade-off favours simplicity for an instructional project. Introducing `HouseId(i64)` style newtypes would be the natural next step if cross-entity ID confusion becomes a real concern.

## 5. Routing and Endpoints

All domain routes are registered in `src/routes.rs` under a single `/api/v1` scope. Observability routes (`/health`, `/metrics`) sit at the root. Swagger UI is mounted at `/swagger-ui/` and serves the OpenAPI document from `/api-docs/openapi.json`.

| Resource | Routes | Requirement |
|----------|--------|-------------|
| Countries | `GET /countries`, `GET/PUT/DELETE /countries/{id}`, `POST /countries` | [REQ-CO-001](prd.md#req-co-001), REQ-CO-002 |
| Addresses | `POST /addresses`, `GET/PUT/DELETE /addresses/{id}` (no `LIST`) | REQ-AD-001, REQ-AD-002 |
| Managers | `GET/POST /managers`, `GET/PUT/DELETE /managers/{id}` | REQ-MG-001, REQ-MG-002 |
| Persons | `GET/POST /persons`, `GET/PUT/DELETE /persons/{id}` | REQ-PE-001, REQ-PE-002 |
| Houses | `GET/POST /houses`, `GET/PUT/DELETE /houses/{id}` | REQ-HO-001, REQ-HO-002 |
| Calendar | `GET/POST/PATCH/DELETE /houses/{house_id}/calendar`, `GET /houses/{house_id}/calendar/{id}` | REQ-CA-001 – REQ-CA-006 |
| Bookings | `GET/POST /bookings`, `GET /bookings/{id}`, `POST /bookings/{id}/cancel`, `POST /bookings/{id}/payment` | REQ-BK-001 – REQ-BK-005 |
| Observability | `GET /health`, `GET /metrics`, `GET /swagger-ui/*`, `GET /api-docs/openapi.json` | REQ-OB-001, REQ-OB-002 |

Conventions enforced uniformly:

- `POST` returns **201** with a `Location` header.
- `DELETE` returns **204** with no body.
- State transitions use `POST /{resource}/{id}/{action}` (`/cancel`, `/payment`).
- Range filters and bulk operations on calendar use `?from=YYYY-MM-DD&to=YYYY-MM-DD`. Both bounds are inclusive.
- JSON fields use `snake_case`. Dates are ISO 8601 calendar days (no time, no zone).

## 6. Request Flow

### 6.1 Standard CRUD Flow

```
HTTP request
   │
   ▼
handler (handlers/{entity}.rs)
   ├── extract web::Json<Request>, web::Path<id>, web::Query<Q>
   ├── body.validate()?     ──── field-shape validation
   │                              (ADR 2026-05-09: handler boundary)
   │                              ValidationErrors → AppError::ValidationError → 400
   ▼
service (services/{entity}.rs)
   ├── apply business rules (from <= to, status != Rented, not already cancelled…)
   │                              violation → AppError::UnprocessableEntity → 422
   ▼
repository (repositories/{entity}.rs)
   ├── sqlx::query! / sqlx::query_scalar!  (compile-time-checked SQL)
   │                              RowNotFound → AppError::NotFound → 404
   │                              23505 unique_violation → AppError::Conflict → 409
   │                              23503 foreign_key_violation → AppError::Conflict → 409
   │                              other → AppError::Internal → 500
   ▼
return owned response struct  (handlers wrap in HttpResponse::Ok/Created/NoContent)
```

### 6.2 Write-then-Read Pattern

`create` and `update` repository functions perform the write, then call the local `find_by_id` to load the full embedded response. Reasons:

- The JOIN SQL that produces the embedded response (e.g. `House` with `Address` + `Country` + `Manager`) lives in exactly one place.
- The response always reflects committed state, including DB defaults and triggers.
- Cost: one extra `SELECT` per write. Acceptable for the load profile in scope.

### 6.3 Booking Create — Concurrency-Safe Path

`repositories::booking::create` is the only path that mutates two tables (`bookings` and `calendar`) transactionally. It uses `SELECT … FOR UPDATE` to prevent the classic double-booking race:

```text
BEGIN
  SELECT id, date, status, price
  FROM calendar
  WHERE house_id = $1 AND date BETWEEN $from AND $to
  ORDER BY date
  FOR UPDATE                           -- row-level lock for the duration of the tx

  if entries.len() != (to - from + 1)  -> 422 (missing days, edge case 7)
  if any status != Rentable            -> 422 (not all days Rentable, REQ-BK-003)

  INSERT INTO bookings (…) RETURNING id
  UPDATE calendar SET status='Rented' WHERE house_id = $1 AND date BETWEEN $from AND $to
COMMIT

SELECT (via find_by_id) to build the response
attach expected_total_price = SUM(entries.price) to the response (service layer)
```

`expected_total_price` is **computed** in the repository (from locked rows), **attached** by the service, **returned only in the 201 body**, and **never persisted** (REQ-BK-002, edge case 8).

### 6.4 Booking Cancel

```text
BEGIN
  SELECT status, house_id, from_date, to_date
  FROM bookings WHERE id = $1 FOR UPDATE

  if status == Cancelled  -> 422 (already cancelled, REQ-BK-004, edge case 9)

  UPDATE bookings SET status='Cancelled', paid_at=NULL, total_paid=NULL WHERE id = $1
  UPDATE calendar SET status='Rentable' WHERE house_id=… AND date BETWEEN …
COMMIT
```

Payment fields are nulled on cancellation (REQ-BK-004). The `ck_bookings_payment` check constraint enforces that `paid_at` and `total_paid` are either both set or both null.

### 6.5 Calendar Create — Idempotent Insert

`repositories::calendar::create` iterates `[from, to]` day-by-day inside a single transaction, issuing `INSERT … ON CONFLICT (house_id, date) DO NOTHING RETURNING …`. Days that already have an entry are skipped silently; only newly created rows are returned (REQ-CA-003, edge case 3). The transaction commits even if no rows were inserted.

### 6.6 Calendar Delete — All-or-Nothing

`repositories::calendar::delete` first runs a `SELECT COUNT(*)` joining `calendar → bookings → calendar (from/to)` to detect any non-cancelled booking that covers any day in the requested range. If the count is > 0, the operation aborts with `AppError::Conflict` (409) and no rows are deleted (REQ-CA-005, edge case 6). Otherwise the `DELETE` runs unconditionally.

## 7. Validation Strategy

Validation is split deliberately along two axes:

| Layer | Library / mechanism | Examples | HTTP status |
|-------|---------------------|----------|-------------|
| Handler — field shape | `validator` crate's `#[validate(...)]` attributes on request structs; `body.validate()?` is called once per handler before delegating to the service | `length(min=1, max=255)`, `range(min=1)`, `email`, `length(equal=2)` for ISO codes, custom `validate_non_negative_decimal` for prices | **400** (`AppError::ValidationError`) |
| Service — business rules | Inline Rust checks (`if … return Err(AppError::UnprocessableEntity(…))`) | `from <= to`, `status != Rented` on create, "booking is not already cancelled", "cannot record payment for cancelled booking", "all days must be Rentable" | **422** (`AppError::UnprocessableEntity`) |
| Database — referential | FK constraints and unique indexes; sqlx error code mapping | `country_id` references a missing country, duplicate `iso_code` | **409** (`AppError::Conflict` from 23503 / 23505) |

The placement decision is documented in [ADR 2026-05-09](adr/2026-05-09-validation-at-handler-boundary.md). Services accept request structs directly without re-validating — internal callers (tests, future CLI workers) construct them without ceremony.

`ValidationErrors` from `validator` is flattened into a `Vec<String>` of `"field: code"` entries by `From<validator::ValidationErrors> for AppError` in `errors.rs`, which walks `Field`, `Struct`, and `List` variants recursively. The 400 response body is `{ "error": "validation failed", "details": [...] }`.

## 8. Error Handling

A single `AppError` enum implements actix-web's `ResponseError`. This is the project's equivalent of Spring's `@ControllerAdvice`.

```rust
pub enum AppError {
    NotFound(String),            // 404
    Conflict(String),            // 409
    UnprocessableEntity(String), // 422
    ValidationError(Vec<String>),// 400 (field shape)
    Internal(#[source] anyhow::Error), // 500
}
```

### 8.1 Response Bodies

| Variant | Status | Body |
|---------|--------|------|
| `NotFound`, `Conflict`, `UnprocessableEntity` | 404 / 409 / 422 | `{ "error": "<message>" }` |
| `ValidationError(details)` | 400 | `{ "error": "validation failed", "details": ["field: code", …] }` |
| `Internal(e)` | 500 | `{ "error": "internal server error" }`. The full `e` is logged at `error` level via `tracing::error!(error = ?e, "internal server error");` — never serialized to the client. |

### 8.2 Conversions (`From` impls)

| From | To | Behaviour |
|------|----|-----------|
| `sqlx::Error::RowNotFound` | `AppError::NotFound("resource not found")` | Default. Repository functions may override with a more specific message (e.g. `format!("booking {id} not found")`). |
| `sqlx::Error::Database` SQLSTATE `23505` (unique_violation) | `AppError::Conflict(db.message())` | |
| `sqlx::Error::Database` SQLSTATE `23503` (foreign_key_violation) | `AppError::Conflict(db.message())` | |
| Any other `sqlx::Error` | `AppError::Internal(anyhow!(…))` | Logged as `internal server error`. |
| `anyhow::Error` | `AppError::Internal(e)` | |
| `validator::ValidationErrors` | `AppError::ValidationError(Vec<String>)` | Recursive flattening of Field/Struct/List variants. |

### 8.3 Mapping Table to PRD Status Codes

| PRD scenario | Origin | `AppError` variant | HTTP |
|--------------|--------|---------------------|------|
| Missing or malformed JSON, invalid field shape (edge case 13) | handler `body.validate()?` | `ValidationError` | 400 |
| Unknown numeric ID in path (edge case 12) | repository `find_by_id` → `RowNotFound` | `NotFound` | 404 |
| Referential delete blocked (edge case 1) | repository FK check or PG `23503` | `Conflict` | 409 |
| Calendar range held by booking (REQ-CA-005, edge case 6) | repository pre-check `SELECT COUNT(*)` | `Conflict` | 409 |
| `from > to`, `status = Rented` on create, days not all `Rentable`, already cancelled, payment on cancelled (REQ-CA-003, REQ-BK-003, REQ-BK-004, REQ-BK-005, edge cases 2, 4, 7, 9, 10) | service or repository business rule | `UnprocessableEntity` | 422 |
| Anything else | `From<sqlx::Error>` fallback | `Internal` | 500 |

## 9. Configuration

All configuration is environment-driven (12-factor). `AppConfig::from_env()` is called once at startup; there is no runtime reconfiguration. `dotenvy::dotenv().ok()` loads `.env` in development.

| Env var | Type | Default | Purpose |
|---------|------|---------|---------|
| `DATABASE_URL` | string | **required** | Postgres connection string. |
| `DATABASE_MAX_CONNECTIONS` | u32 | `5` | sqlx pool ceiling. |
| `DATABASE_MIN_CONNECTIONS` | u32 | `1` | sqlx pool floor. |
| `DATABASE_CONNECT_TIMEOUT_SECS` | u64 | `5` | Acquire timeout. |
| `SERVER_HOST` | string | `0.0.0.0` | Bind host. |
| `SERVER_PORT` | u16 | `8080` | Bind port. |
| `APP_NAME` | string | `rental-api` | Service name for tracing/OTLP. |
| `APP_ENV` | string | `development` | When `development`, logs use text formatter; otherwise JSON. |
| `OTEL_EXPORTER_OTLP_ENDPOINT` | string | unset | If set, OTLP/gRPC span exporter is enabled and spans flow to the configured collector (e.g. Jaeger, Tempo). |
| `RUST_LOG` (via `EnvFilter`) | string | `info,sqlx=warn,actix_web=info,h2=off,hyper=off,tonic=off,tower=off,reqwest=off` | Log/trace filter. |

Constants embedded in code rather than configuration:

- API prefix: `/api/v1`
- Metrics path: `/metrics`
- Health path: `/health`
- Swagger UI: `/swagger-ui/`
- OpenAPI document: `/api-docs/openapi.json`

## 10. Database

### 10.1 Schema

Single migration file `migrations/0001_initial_schema.up.sql` defines:

- Two Postgres enum types: `calendar_status` (`NotRentable`, `Rentable`, `Rented`), `booking_status` (`Active`, `Cancelled`).
- Seven tables: `countries`, `addresses`, `managers`, `persons`, `houses`, `calendar`, `bookings`.
- Identifier columns are `BIGSERIAL`.
- `countries.iso_code` is `CHAR(2)` with `CHECK (iso_code ~ '^[A-Z]{2}$')` and `UNIQUE` (see [ADR 2026-05-06](adr/2026-05-06-country-codes-iso-3166-1-alpha-2.md)).
- `managers.email` and `persons.email` are `UNIQUE`.
- `calendar` has `UNIQUE (house_id, date)` — at most one entry per house per day (REQ-CA-003 invariant).
- `bookings` keeps both `from_calendar_id`/`to_calendar_id` (FK with `ON DELETE SET NULL`) **and** `from_date`/`to_date` (denormalised). The dates remain valid for historical reporting even if calendar rows are eventually deleted; the FKs allow `calendar` deletion checks to detect overlap (`repositories::calendar::delete`).
- `bookings` has `CHECK ((paid_at IS NULL) = (total_paid IS NULL))` — payment is recorded atomically.
- Indexes cover all FK columns and `calendar (house_id, date)`.

Migrations run at startup via `sqlx::migrate!("../../migrations").run(&pool).await`. The service does not start serving domain endpoints if migrations fail (REQ-OP-003).

### 10.2 sqlx Usage

- All SQL uses the **`sqlx::query!` / `sqlx::query_scalar!`** compile-time-checked macros — queries are verified against the live database schema (or against `.sqlx/` offline metadata) at `cargo check` time.
- `.sqlx/` is committed to git. CI and Docker builds run without a live database via `SQLX_OFFLINE=true`.
- Custom Postgres enums are queried with the explicit type annotation `status AS "status: BookingStatus"` so sqlx maps the Postgres value into the Rust enum.
- Bind parameters use Postgres positional `$1, $2, …`.
- Optional filters (`WHERE ($1::bigint IS NULL OR …)`) keep list endpoints to one SQL statement regardless of whether filters are present (`repositories::booking::find_all`, `repositories::calendar::find_all`).
- All list endpoints support optional `limit` and `offset` query parameters (`models::pagination::PaginationQuery`). Omitting both returns all rows (backward-compatible). The SQL uses `LIMIT $n::bigint OFFSET COALESCE($m::bigint, 0)`; PostgreSQL treats `LIMIT NULL` as no limit.

### 10.3 Connection Pool

A single `sqlx::PgPool` is created in `db::create_pool` from `PgPoolOptions` with min/max connections and acquire timeout from `AppConfig`. The pool is wrapped in `web::Data<AppState>` and passed to handlers; every handler clones the `Data` reference (cheap `Arc` bump) and forwards `&state.pool` down the call chain.

### 10.4 Concurrency

The only path that requires explicit locking is booking writes (Section 6.3 / 6.4): both use `SELECT … FOR UPDATE` inside a transaction. All other repository operations are single-statement and rely on Postgres's MVCC defaults.

## 11. Observability

| Concern | Mechanism | Endpoint / Output |
|---------|-----------|-------------------|
| **Health** (REQ-OB-001) | `handlers::health::health` returns `{ "status": "ok" }` with 200 unconditionally. Mounted as `GET /health` outside the `/api/v1` scope so it is unaffected by API versioning. | `GET /health` |
| **Metrics** (REQ-OB-002) | `actix-web-prom`'s `PrometheusMetricsBuilder`. Excludes `/swagger-ui/...` and `/api-docs/openapi.json` so they do not pollute counters. | `GET /metrics` (Prometheus text format) |
| **Logging** (REQ-OB-003) | `tracing-subscriber` with an `EnvFilter` from `RUST_LOG` (or the default). `fmt::layer()` (text) in development; `fmt::layer().json()` in non-development. | stdout |
| **Tracing — request scoping** | `TracingLogger::default()` middleware attaches a request span to every request. A `wrap_fn` reads the `RequestId` from extensions and writes it as the `x-request-id` response header. | every HTTP response |
| **Tracing — distributed** | Optional. If `OTEL_EXPORTER_OTLP_ENDPOINT` is set, `opentelemetry_otlp::SpanExporter` (gRPC/tonic) is built, an `SdkTracerProvider` with a batch exporter is registered, and `tracing-opentelemetry` bridges tracing spans into OTLP. Spans flow to Jaeger / Tempo / any OTLP collector. On shutdown the provider is flushed. | OTLP endpoint |
| **Span fields** | Repository and service functions use `#[tracing::instrument(skip(pool, …), fields(layer = "repository" | "service", …))]` to attach a `layer` field and selected request fields (`house_id`, `person_id`) to every span. | included in spans and logs |

## 12. Deployment

### 12.1 Container Image

Two-stage Docker build:

Four-stage multi-arch build (`linux/amd64` + `linux/arm64`):

1. **chef:** `rust:slim` with `cargo-chef`, `musl-tools`, and `clang` installed. Runs `cargo chef prepare` to produce a deterministic dependency manifest.
2. **builder:** `cargo chef cook` warms the dependency layer; `cargo build --release --target {x86_64,aarch64}-unknown-linux-musl` with `SQLX_OFFLINE=true` produces a statically-linked binary.
3. **runtime:** `FROM scratch` — empty base, no libc, no shell, no package manager. Final image is ~18 MB (binary + CA certificates bundle only).

The release binary uses `opt-level = "z"`, LTO, single codegen unit, `panic = abort`, and stripping (set in workspace `Cargo.toml`).

### 12.2 Kubernetes

A `Deployment` manifest references the published image. Both liveness and readiness probes call `GET /health`. Resource requests/limits are set conservatively to match the low memory footprint observed under load tests.

### 12.3 CI / CD

GitHub Actions pipeline: `cargo fmt --check` → `cargo clippy -- -D warnings` → `cargo test` → Docker build → push to GitHub Container Registry and DockerHub (REQ-OP-002).

## 13. Testing Strategy

### 13.1 Test Pyramid

```text
       ┌────────────────┐
       │  Integration   │  Real Postgres (sqlx::test); per-test fresh DB
       │   ~15%         │  Files: crates/rental-api/tests/*.rs
      ┌┴────────────────┴┐
      │   Unit Tests      │  #[cfg(test)] mod tests inside source files
      │   ~80%            │  Pure functions: validate_date_range, etc.
      └───────────────────┘
       Load tests: separate crate (crates/loadtest) — not part of the
       Cargo test target. Run on demand against a deployed instance.
```

### 13.2 Unit Tests

Co-located inside source modules as `#[cfg(test)] mod tests { … }`. They test pure functions only — date-range validation, status-rule validation, the `ValidationErrors → AppError` flattening logic. They never touch the database, the filesystem, or the HTTP layer.

### 13.3 Integration Tests

Located in `crates/rental-api/tests/`. Each test is annotated `#[sqlx::test(migrations = "../../migrations")]`, which:

- Creates a fresh Postgres database (template-based).
- Runs all migrations.
- Injects a `PgPool` into the test function.
- Drops the database when the test finishes.

This means each test runs in full isolation against a real schema — no mocks, no in-memory substitutes. A shared `tests/common/mod.rs` provides fixture builders (`create_test_house`, `create_test_person`) that compose the country → address → manager → house hierarchy.

Integration tests exercise the **service layer** (e.g. `services::booking::create`) directly, not via HTTP. The HTTP layer is thin enough that handler-level coverage is verified through Swagger UI and the load tests rather than per-endpoint integration tests.

### 13.4 Assertion Style

Tests use the standard library's `assert!`, `assert_eq!`, `assert_matches!`. No external assertion library. Error-shape assertions use `assert!(matches!(result, Err(AppError::UnprocessableEntity(_))))`.

### 13.5 No Mocks in Domain Code

There are no mock libraries (`mockall`, etc.) in use. The domain has no traits requiring mocks: services depend on `&PgPool` directly, and integration tests provide a real pool. If a future requirement introduces an external service trait, `mockall` is acceptable at that boundary only.

### 13.6 Load Tests

`crates/loadtest/` contains realistic write/read load profiles (Phase A, smart-write, etc.) that drive the running service. They validate that the architecture (FOR UPDATE booking lock, write-then-read pattern, pool sizing) holds under contention. They are not run as part of `cargo test`.

## 14. Cross-Cutting Implementation Notes

- **No global state.** The only shared state is the `PgPool` inside `AppState`. There are no `static mut`, `OnceCell`, or `lazy_static` runtime singletons in domain code.
- **No `unwrap()` in production code paths.** Repository and service code uses `?`. The startup path in `main.rs` uses `expect("…")` only for invariants that must hold for the service to function (e.g. building the Prometheus middleware).
- **No `unsafe` code.**
- **Workspace-level `release` profile.** Defined once at the workspace root and inherited by every crate.
- **OpenAPI as source of truth for the wire format.** Every handler is annotated with `#[utoipa::path(...)]`, and every request/response struct derives `ToSchema`. The `ApiDoc` derive in `openapi.rs` enumerates the full surface, which is then served by `utoipa-swagger-ui`. Drift between code and OpenAPI is therefore impossible — changing a handler signature or schema requires updating both in the same edit.

## 15. References

- [`prd.md`](prd.md) — what the system does and acceptance criteria
- [`rust-principles.md`](rust-principles.md) — ownership, types, error handling
- [`ddd-principles.md`](ddd-principles.md) — domain design vocabulary used in naming conventions
- [`tdd-principles.md`](tdd-principles.md), [`testing-principles.md`](testing-principles.md) — testing approach
- [`operation_infos.md`](operation_infos.md) — observability runbook: log queries, Prometheus PromQL, Jaeger trace queries, local stack URLs
- [`dev_infos.md`](dev_infos.md) — developer quick-reference: sqlx-cli setup, migration commands, local service URLs
- [`adr/2026-05-06-country-codes-iso-3166-1-alpha-2.md`](adr/2026-05-06-country-codes-iso-3166-1-alpha-2.md) — ISO 3166-1 alpha-2 for country codes
- [`adr/2026-05-09-validation-at-handler-boundary.md`](adr/2026-05-09-validation-at-handler-boundary.md) — validation runs in handlers, business rules in services
