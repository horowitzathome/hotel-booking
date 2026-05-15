# Input validation runs at the handler boundary, not in services

**Status:** Accepted

## Context

Field-shape validation (the `validator` crate's `#[validate]` attributes on request structs) must run somewhere. The two natural candidates are the actix-web handler (immediately after JSON deserialisation) and the service layer. Placing it in the wrong layer either duplicates work or entangles services with HTTP-shaped concerns.

## Options Considered

1. **Validate in handler only** — one call per endpoint, before the service is reached; services receive structurally valid inputs
2. **Validate in service only** — service is the single authoritative gatekeeper; handler passes raw input through
3. **Validate in both handler and service** — belt-and-suspenders; highest redundancy
4. **Use `actix-web-validator` crate** — provides a `Json<T: Validate>` extractor that auto-validates, removing the explicit call in each handler

## Decision

We validate in the handler only (Option 1). The handler is the only entry point that accepts external bytes. Services receive values the program itself constructed; re-checking shape there is redundant work with no behavioural change. `actix-web-validator` was rejected because it saves one `body.validate()?;` line per handler at the cost of an extra dependency and a less obvious error path — six call sites is below the threshold where the dependency pays for itself.

## Consequences

- Each handler that accepts a `web::Json<...Request>` body calls `body.validate()?;` exactly once before delegating to the service.
- The `?` lifts `validator::ValidationErrors` into `AppError::ValidationError` via the `From` impl in `src/errors.rs`, which maps to **HTTP 400**.
- Service functions accept the raw `Create*Request` / `Update*Request` types without re-validating.
- Business rules (`from <= to`, `status != Rented`, "booking is not already cancelled", etc.) remain in the service layer and return `AppError::UnprocessableEntity` → **HTTP 422**.
- Internal callers (tests, future CLI subcommands, future queue workers) can construct request structs directly without a redundant validation call.
- **When to revisit:** if a second entry point appears (CLI, Kafka consumer, gRPC service) that bypasses the actix-web handler, switch to the parse-don't-validate pattern: introduce a `Validated<T>` newtype whose constructor runs `.validate()`, and change service signatures to accept `&Validated<Create*Request>`. This pushes the guarantee into the type system at compile time and forces every entry point through the same gate.

## Implementation

Implemented in Step 17 (input validation). No formal requirement ID.

## References

- `src/errors.rs` — `From<validator::ValidationErrors> for AppError` conversion
- `src/handlers/*.rs` — `body.validate()?;` at the top of each create/update handler
