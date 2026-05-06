# Architecture

This document describes the architecture of the house rental backend — a Rust REST API backed by
Postgres. Its primary audience is Java/Spring Boot developers learning Rust by comparison.
Authentication and authorisation are intentionally out of scope.

## Layered Architecture

The application follows the same three-layer model as a Spring Boot application.

```
HTTP Request
     │
     ▼
┌─────────────┐     = @RestController
│  Handlers   │       actix-web handler functions
└──────┬──────┘
       │
       ▼
┌─────────────┐     = @Service
│  Services   │       plain Rust structs
└──────┬──────┘
       │
       ▼
┌─────────────┐     = @Repository / JpaRepository
│ Repositories│       SQLx queries against Postgres
└──────┬──────┘
       │
       ▼
   Postgres
```

There is no IoC container. Dependencies are wired explicitly in `main.rs` and passed as actix-web
`AppState` — the equivalent of Spring's `@Autowired`, but resolved at compile time.

## Module Layout

```
src/
  main.rs              — startup, wiring, server config  (= Spring Boot main class)
  config.rs            — typed config loaded from env vars  (= application.properties)
  db.rs                — SQLx connection pool setup
  errors.rs            — AppError enum implementing ResponseError  (= @ControllerAdvice)
  routes.rs            — route registration

  handlers/            — HTTP layer  (= @RestController)
    house.rs
    booking.rs
    manager.rs
    person.rs
    calendar.rs

  services/            — business logic  (= @Service)
    house.rs
    booking.rs
    manager.rs
    person.rs
    calendar.rs

  repositories/        — database access  (= @Repository)
    house.rs
    booking.rs
    manager.rs
    person.rs
    calendar.rs

  models/              — domain structs + request/response types  (= @Entity + record classes)
    house.rs
    booking.rs
    address.rs
    country.rs
    manager.rs
    person.rs
    calendar.rs
```

## Technology Stack

| Concern | Crate | Spring Boot equivalent |
|---|---|---|
| HTTP server | actix-web | Spring MVC |
| Async runtime | tokio | Virtual threads / Project Reactor |
| Database | sqlx | Spring Data JPA + JDBC |
| JSON | serde / serde_json | Jackson |
| Tracing | tracing + tracing-actix-web | Spring Sleuth / Micrometer Tracing |
| Logging | tracing-subscriber (JSON) | Logback / Log4j2 |
| Metrics | actix-web-prom | Spring Boot Actuator + Micrometer |
| Error handling | thiserror + anyhow | @ControllerAdvice |
| Config | config + dotenvy | application.properties / @Value |
| Validation | validator | Bean Validation / @Valid |

## Cross-cutting Concerns

**Tracing**
`tracing-actix-web` attaches a request-id span to every incoming request automatically — the
equivalent of Spring Sleuth's MDC propagation. Spans are emitted as structured JSON.

**Logging**
`tracing-subscriber` with `fmt::json()` produces structured JSON log lines compatible with
log aggregators (Loki, ELK). No separate logging framework is needed; tracing handles both.

**Metrics**
A `/metrics` endpoint exposes Prometheus-format counters and histograms. Grafana scrapes this
endpoint directly — equivalent to Spring Boot Actuator's Micrometer Prometheus endpoint.

**Health check**
`GET /health` returns HTTP 200 with a JSON body. Kubernetes liveness and readiness probes point
here.

**Error handling**
A single `AppError` enum implements actix-web's `ResponseError` trait and maps every domain error
to the correct HTTP status code and JSON error body. This replaces Spring's `@ControllerAdvice`.

## Configuration

Configuration follows the 12-factor app pattern: all values come from environment variables.
`dotenvy` loads a `.env` file in development; in production the variables are injected by
Kubernetes. At startup they are parsed once into a typed `AppConfig` struct — there is no runtime
property injection.

## Container and Deployment

**Docker (multi-stage build)**
- Stage 1 (builder): `rust:slim` with `cargo-chef` for reproducible, layer-cached dependency builds
- Stage 2 (runtime): `gcr.io/distroless/cc` — no shell or package manager, image under 20 MB

**Kubernetes**
A `Deployment` manifest references the container image. Liveness and readiness probes both call
`GET /health`. Resource limits are set to reflect Rust's low memory footprint.

**GitHub Actions**
Pipeline: compile → clippy → test → Docker build → push to GitHub Container Registry and DockerHub.

## Key Differences from Spring Boot

| Spring Boot | Rust |
|---|---|
| IoC container wires beans at runtime | Dependencies wired explicitly in `main.rs` at compile time |
| Annotations drive behaviour (`@Service`, `@Transactional`) | No annotations; types and traits drive behaviour |
| SQL validated at runtime (JPA/Hibernate) | SQL validated against live DB **at compile time** (sqlx macros) |
| `null` + `@Nullable` / `Optional<T>` | `Option<T>` — absence is always explicit in the type |
| Checked and unchecked exceptions | Errors are return values: `Result<T, E>` — no throws, no try/catch |
