# hotel-booking

Rust REST API for hotel room rental and booking management — built with actix-web, sqlx, and PostgreSQL.

## Quick start

```bash
# Start Postgres
just db-run

# Run migrations
just db-migrate

# Start the API (http://localhost:8080)
just run-dev

# Swagger UI
open http://localhost:8080/swagger-ui/
```

## Full observability stack (traces + logs + metrics + Grafana)

```bash
just compose-up   # app + Postgres + Jaeger + Loki + Prometheus + Grafana
just compose-down
```

## Documentation

| Document | Purpose |
|---|---|
| [`docs/prd.md`](docs/prd.md) | Product requirements |
| [`docs/system-design.md`](docs/system-design.md) | Architecture, patterns, guardrails |
| [`docs/dev_infos.md`](docs/dev_infos.md) | Developer quick-reference (ports, URLs, curl examples) |
| [`docs/operation_infos.md`](docs/operation_infos.md) | Ops guide — logs, metrics, traces, load tests |
| [`docs/adr/`](docs/adr/) | Architectural Decision Records |
| [`docs/rust-principles.md`](docs/rust-principles.md) | Rust coding principles |
| [`docs/testing-principles.md`](docs/testing-principles.md) | Testing strategy and conventions |

## Running tests

```bash
just test        # unit + integration (requires live Postgres)
just lint        # lint
just fmt-check   # format check
```

## Load tests

```bash
just seed-fresh && just loadtest-baseline   # read-only, 1 min
just seed-fresh && just loadtest-write      # read + write, 3 min
```

See [`docs/operation_infos.md`](docs/operation_infos.md#5-load-tests) for full details.
