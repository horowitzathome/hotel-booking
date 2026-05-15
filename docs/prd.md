# Product Requirements Document: hotel-booking

## 1. Problem Statement

Java and Spring Boot developers lack a concrete reference for evaluating Rust as a backend platform. They need a working REST API that exercises the same concerns as a typical Spring Boot service — CRUD endpoints, relational persistence, validation, observability, containerisation — so they can compare ergonomics, runtime behaviour, and operational characteristics side by side. The system also serves as a demonstration of PostgreSQL as the primary data store for usability, performance, and throughput.

The domain is villa rental management: countries, addresses, managers, persons (renters), houses, per-house calendar entries with daily pricing and availability, and bookings with payment recording.

## 2. Goals

- Deliver a REST API that manages countries, addresses, managers, persons, houses, calendars, and bookings end to end.
- Provide a learning artefact aimed at Java/Spring Boot developers comparing Rust to the JVM ecosystem.
- Demonstrate PostgreSQL as the system of record under realistic write and read load.
- Expose Kubernetes-compatible health endpoints and Prometheus-format metrics.
- Ship as a minimal container image that runs on macOS and Linux.
- Build and publish via GitHub Actions.

## 3. Non-Goals

- **No authentication or authorisation** — out of scope; the API is open.
- **No user-facing UI** — backend only.
- **No multi-tenant isolation** — single logical tenant.
- **No currency handling** — prices are scalar decimal values without currency codes.
- **No time-of-day or time-zone semantics** — all dates are calendar days (`YYYY-MM-DD`).
- **No persisted expected booking total** — only the actual paid amount is stored (REQ-BK-002).
- **No soft deletes** — deletes are physical and blocked by referential constraints.
- **No partial-success calendar deletion** — referential conflicts abort the whole operation (REQ-CA-005).

## 4. Requirements

All domain endpoints are prefixed `/api/v1/`. Observability endpoints (`/health`, `/metrics`) sit at the root. JSON uses `snake_case`. Dates are ISO 8601 calendar days. Error bodies follow `{ "error": string, "details"?: string[] }`. Create returns 201 with a `Location` header; delete returns 204; state transitions use `POST .../{action}`.

Status codes: 400 validation, 404 not found, 409 referential conflict, 422 business-rule violation.

### 4.1 Countries

<a id="req-co-001"></a>
| ID | Requirement | Acceptance Criteria |
|----|-------------|---------------------|
| REQ-CO-001 | List, get, create, update countries by `name` and `iso_code`. | `GET /countries` returns 200 with array; `GET /countries/{id}` returns 200 or 404; `POST` returns 201 with `Location`; `PUT` returns 200 or 404. |
| REQ-CO-002 | Delete a country only when no address references it. | Returns 204 on success; 409 if any address references the country; 404 if absent. |

### 4.2 Addresses

| ID | Requirement | Acceptance Criteria |
|----|-------------|---------------------|
| REQ-AD-001 | Get, create, update an address with `street`, `number`, `postcode`, `city`, optional `province`, and `country_id`. | Responses embed the full country object. Missing/invalid fields return 400. Unknown `country_id` returns 400 or 404. |
| REQ-AD-002 | Delete an address only when no house references it. | Returns 204 on success; 409 if any house references it; 404 if absent. |

### 4.3 Managers

| ID | Requirement | Acceptance Criteria |
|----|-------------|---------------------|
| REQ-MG-001 | List, get, create, update managers by `first_name`, `last_name`, `email`, `phone`. | CRUD endpoints return the documented shapes and status codes. |
| REQ-MG-002 | Delete a manager only when they manage no house. | Returns 204 on success; 409 if any house references the manager; 404 if absent. |

### 4.4 Persons (Renters)

| ID | Requirement | Acceptance Criteria |
|----|-------------|---------------------|
| REQ-PE-001 | List, get, create, update persons by `first_name`, `last_name`, `email`, `phone`. | CRUD endpoints return the documented shapes and status codes. |
| REQ-PE-002 | Delete a person only when they have no booking (active or cancelled). | Returns 204 on success; 409 if any booking references the person; 404 if absent. |

### 4.5 Houses

| ID | Requirement | Acceptance Criteria |
|----|-------------|---------------------|
| REQ-HO-001 | List, get, create, update houses with `name`, `description`, `address_id`, `manager_id`. | Responses embed full address (with country) and manager objects. Unknown referenced IDs return 400 or 404. |
| REQ-HO-002 | Delete a house only when it has no booking. | Returns 204 on success; 409 if any booking references it; 404 if absent. |

### 4.6 Calendar (per house)

A calendar entry has `date`, `status` (`NotRentable`, `Rentable`, `Rented`), and `price`. Only one entry exists per `(house, date)`.

| ID | Requirement | Acceptance Criteria |
|----|-------------|---------------------|
| REQ-CA-001 | List calendar entries for a house, optionally filtered by `from` and `to` query parameters. | Returns 200 with array. Range is inclusive on both ends. Missing range returns all entries for the house. |
| REQ-CA-002 | Get a single calendar entry by ID. | Returns 200 or 404. |
| REQ-CA-003 | Create entries for every day in `[from, to]` with caller-supplied `status` (`NotRentable` or `Rentable`) and `price`. | Days already having an entry are skipped silently. Response contains only newly created entries. `status = Rented` returns 422. |
| REQ-CA-004 | Update the price of every existing entry in `[from, to]`. | Returns 200 with the updated entries. Updates do not change any booking's `total_paid`. |
| REQ-CA-005 | Delete every entry in `[from, to]` only when no non-cancelled booking references any entry in the range. | Returns 204 on success; 409 if any entry is referenced by a non-cancelled booking, with no entries deleted. |
| REQ-CA-006 | The `Rented` status is set and cleared exclusively by booking operations. | No calendar endpoint accepts or produces a transition to `Rented`. |

### 4.7 Bookings

A booking has `status` (`Active`, `Cancelled`), `from`, `to`, `house`, `person`, optional `paid_at`, optional `total_paid`.

| ID | Requirement | Acceptance Criteria |
|----|-------------|---------------------|
| REQ-BK-001 | List bookings, optionally filtered by `house_id` or `person_id` query parameters. | Returns 200 with array. Filters combine with AND when both are supplied. |
| REQ-BK-002 | Get a booking by ID, including embedded house, person, date range, status, and payment fields. | Returns 200 or 404. `expected_total_price` is never present in GET responses. |
| REQ-BK-003 | Create a booking for `house_id`, `person_id`, `from`, `to` when every day in the range is `Rentable`. | On success: returns 201 with the booking plus a computed `expected_total_price`; flips every day in the range to `Rented`. If any day is not `Rentable` (including `NotRentable`, `Rented`, or missing entry): returns 422 with no state change. |
| REQ-BK-004 | Cancel an active booking. | Returns 200 with `status = Cancelled`, `paid_at = null`, `total_paid = null`; resets every covered calendar entry to `Rentable`. Cancelling an already-cancelled booking returns 422. |
| REQ-BK-005 | Record payment on a booking with `paid_at` and `total_paid`. | Returns 200 with the supplied values stored. Recording payment on a cancelled booking returns 422. |

### 4.8 Observability and Operations

| ID | Requirement | Acceptance Criteria |
|----|-------------|---------------------|
| REQ-OB-001 | Expose `GET /health` returning `{ "status": "ok" }` with 200 when the process is up. | Suitable for Kubernetes liveness and readiness probes. |
| REQ-OB-002 | Expose `GET /metrics` in Prometheus text format. | Scrapeable by Grafana / Prometheus without authentication. |
| REQ-OB-003 | Emit structured logs and end-to-end request traces. | Each HTTP request produces a log record and a trace span. |
| REQ-OP-001 | Ship as a minimised container image runnable on macOS and Linux. | A documented container build produces an image that starts and serves `/health` on both platforms. |
| REQ-OP-002 | Build, test, and publish the image via GitHub Actions. | A push to the default branch triggers a workflow that runs the test suite and publishes the image. |
| REQ-OP-003 | Persist all domain state in PostgreSQL. | The service requires a reachable PostgreSQL instance to start serving domain endpoints. |

## 5. Configuration

| Property | Type | Default | Description |
|----------|------|---------|-------------|
| Database URL | string | — | PostgreSQL connection string. Required. |
| HTTP bind address | string | `0.0.0.0:8080` | Address the API binds to. |
| Log level | string | `info` | Filter threshold for structured logs. |
| Metrics endpoint | path | `/metrics` | Prometheus scrape path. |
| Health endpoint | path | `/health` | Liveness/readiness path. |

Concrete values and overrides live in [system-design.md#constants](system-design.md#constants).

## 6. Edge Cases

| # | Applies To | Edge Case | Expected Behavior |
|---|-----------|-----------|-------------------|
| 1 | REQ-CO-002, REQ-AD-002, REQ-MG-002, REQ-PE-002, REQ-HO-002 | Delete a resource referenced by another resource. | Return 409; no rows deleted. |
| 2 | REQ-CA-003, REQ-BK-003 | `from` later than `to` in the request body. | Return 422. No state change. |
| 3 | REQ-CA-003 | Range overlaps existing entries. | Skip overlapping days silently; create the remainder; return the created subset. |
| 4 | REQ-CA-003 | Request specifies `status = "Rented"`. | Return 422. No entries created. |
| 5 | REQ-CA-004 | Range contains no existing entries. | Return 200 with empty array. No new entries created. |
| 6 | REQ-CA-005 | Any single day in the delete range is held by a non-cancelled booking. | Return 409. No entries deleted. |
| 7 | REQ-BK-003 | One or more days in the range are `NotRentable`, `Rented`, or missing a calendar entry. | Return 422. Booking not created; calendar unchanged. |
| 8 | REQ-BK-003 | Booking response contains `expected_total_price` computed from per-day prices. | `expected_total_price` returned in the 201 body only; never persisted; never present on later GETs. |
| 9 | REQ-BK-004 | Cancel a booking that is already cancelled. | Return 422. No state change. |
| 10 | REQ-BK-005 | Record payment on a cancelled booking. | Return 422. Payment fields remain null. |
| 11 | REQ-CA-004, REQ-BK-005 | Calendar price changes after a booking was paid. | Booking's `total_paid` is unaffected. Price updates apply only to the calendar. |
| 12 | All endpoints | Unknown numeric ID in the path. | Return 404. |
| 13 | All write endpoints | Missing or malformed JSON body. | Return 400 with `details` listing per-field reasons. |

## 7. Non-Functional Requirements

| Category | Requirement |
|----------|-------------|
| **Performance** | The system sustains the load profiles exercised by the project's load-test suite against a single PostgreSQL instance. Targets and measured numbers live in [system-design.md](system-design.md). |
| **Robustness** | Failed write operations leave no partial state. Calendar bulk operations are all-or-nothing where referential conflicts apply (REQ-CA-005). |
| **Portability** | The container image runs on macOS (developer machines) and Linux (CI, production) without changes. |
| **Testability** | Every requirement above is exercised by automated tests in the repository's test suite. |
| **Encoding** | All file I/O and HTTP responses use UTF-8. |
| **Observability** | Health, metrics, logs, and traces are available without configuration beyond the connection string and bind address. |
