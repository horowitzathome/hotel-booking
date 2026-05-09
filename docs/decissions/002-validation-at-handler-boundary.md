# ADR 002 — Input validation runs at the handler boundary, not in services

## Status
Accepted

## Decision
Field-shape validation (the `validator` crate's `#[validate]` attributes on request structs) is
executed **once, in the actix-web handler**, immediately after the JSON body has been deserialised
and before the service is called. Services do not call `.validate()` on their inputs. Business
rules that depend on database state or cross-field logic remain in the service layer.

## Rationale
- The handler is the only entry point that accepts external bytes. Anything past it is an
  internal Rust value the program constructed itself; re-checking shape there is redundant work
  with no behavioural change.
- Two places to express the same rule means two places to keep in sync — drift risk for zero
  benefit.
- The 400 (shape) vs 422 (business rule) distinction is an HTTP concern. Services shouldn't know
  about HTTP status codes; a service-layer validation failure that *should* have been caught
  earlier is a programming error, not a user error.
- This mirrors Spring Boot's `@Valid` on `@RequestBody`: validation happens once, at the
  controller, never inside `@Service` beans.

## Consequences
- Each handler that accepts a `web::Json<...Request>` body calls `body.validate()?;` exactly once
  before delegating to the service. The `?` lifts `validator::ValidationErrors` into
  `AppError::ValidationError(Vec<String>)` via the `From` impl in `src/errors.rs`, which the
  existing `ResponseError` impl maps to **HTTP 400** with the standard
  `{"error":"validation failed","details":[...]}` body.
- Service functions accept the raw `Create*Request` / `Update*Request` types without re-validating.
- Service-layer logic is reserved for **business rules**: `from <= to` on date ranges,
  `status != Rented` on calendar creates, "booking is not already cancelled", "all days in range
  are Rentable", etc. These return `AppError::UnprocessableEntity(...)` → **HTTP 422**.
- Internal callers (tests, future CLI subcommands, future queue workers) can construct request
  structs directly and skip the redundant work.

## When to revisit
If a second entry point appears (CLI, Kafka consumer, gRPC service) that bypasses the actix-web
handler, switch to the **parse-don't-validate** pattern: introduce a `Validated<T>` newtype whose
constructor runs `.validate()`, and change service signatures to accept `&Validated<Create*Request>`
instead of `&Create*Request`. This pushes the guarantee into the type system at compile time, with
zero runtime cost, and forces every entry point through the same gate. Until that second entry
point exists, the newtype is ceremony without payoff.

## Alternatives considered
- **Validate in both handler and service** — rejected: redundant work, drift risk, no behaviour
  change.
- **Validate only in services** — rejected: the handler still has to construct the request struct
  from JSON; failing later means doing more work before rejecting an obviously bad request, and
  the service ends up entangled with HTTP-shaped error reporting.
- **`actix-web-validator` crate** (provides a `Json<T: Validate>` extractor that auto-validates) —
  rejected: saves one `body.validate()?;` line per handler at the cost of an extra dependency and
  a less explicit error path. Six call sites is below the threshold where the dependency pays for
  itself.
