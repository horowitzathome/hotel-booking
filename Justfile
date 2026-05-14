set dotenv-load

db_container := env_var_or_default("DB_CONTAINER", "rental-postgres")
db_name      := env_var_or_default("DB_NAME",      "rental")
db_user      := env_var_or_default("DB_USER",      "rental")
db_password  := env_var_or_default("DB_PASSWORD",  "rental")
db_port      := env_var_or_default("DB_PORT",      "5432")
db_data_dir  := justfile_directory() / "database"

# Run or start the Postgres 18.3 container (idempotent)
db-run:
    @mkdir -p {{ db_data_dir }}
    docker run -d \
        --name {{ db_container }} \
        -e POSTGRES_DB={{ db_name }} \
        -e POSTGRES_USER={{ db_user }} \
        -e POSTGRES_PASSWORD={{ db_password }} \
        -p {{ db_port }}:5432 \
        -v {{ db_data_dir }}:/var/lib/postgresql \
        postgres:18.3 \
    || docker start {{ db_container }}

## Start Postgres container
db-start:
    docker start {{ db_container }}

# Stop the Postgres container (data is preserved)
db-stop:
    docker stop {{ db_container }}

# Destroy the container (data volume is kept in ./database)
db-rm: db-stop
    docker rm {{ db_container }}

# Open a psql shell inside the running container
db-shell:
    docker exec -it {{ db_container }} psql -U {{ db_user }} -d {{ db_name }}

# Run all pending migrations
db-migrate:
    sqlx migrate run

# Roll back the last migration
db-migrate-revert:
    sqlx migrate revert

# Show migration status
db-migrate-info:
    sqlx migrate info

# Create a new reversible migration  (usage: just db-migrate-add name=add_foo_column)
db-migrate-add name:
    sqlx migrate add --reversible {{ name }}

# Run server in dev mode
run-dev:
    cargo run

# Run server in prod mode
run-prod:
    cargo run --release

# Run all tests across the workspace
test:
    cargo test --workspace

# Run tests with output visible
test-verbose:
    cargo test --workspace -- --nocapture

# Run a single test by name (usage: just test-one name=my_test)
test-one name:
    cargo test --workspace {{ name }}

# Run tests for a specific module (usage: just test-mod mod=services::booking)
test-mod mod:
    cargo test --workspace {{ mod }}

# --- Build / lint / format (workspace-wide) ---

# Build all workspace members in dev profile
build:
    cargo build --workspace

# Build all workspace members in release profile
build-release:
    cargo build --workspace --release

# Build a single package, e.g. `just build-pkg pkg=seeder`
build-pkg pkg:
    cargo build -p {{ pkg }}

# Build a single package in release, e.g. `just build-pkg-release pkg=rental-api`
build-pkg-release pkg:
    cargo build -p {{ pkg }} --release

# Clippy across the whole workspace, including tests/benches/examples — fails on any warning
lint:
    cargo clippy --workspace --all-targets -- -D warnings

# Format every crate in the workspace
fmt:
    cargo fmt --all

# Verify formatting without writing — for CI / pre-commit
fmt-check:
    cargo fmt --all -- --check

# Pre-commit gate: format check + clippy + tests (tests require a running Postgres)
check: fmt-check lint test

# Same as `check` but also rebuilds in release — closer to what CI does
ci: fmt-check lint build-release test

# --- Observability: Jaeger (OTLP traces) ---

# Start Jaeger all-in-one (UI on :16686, OTLP/gRPC on :4317, OTLP/HTTP on :4318)
obs-up:
    docker run -d --name rental-jaeger \
        -p 16686:16686 \
        -p 4317:4317 \
        -p 4318:4318 \
        jaegertracing/all-in-one:1.62 \
    || docker start rental-jaeger

# Stop and remove Jaeger
obs-down:
    docker stop rental-jaeger
    docker rm rental-jaeger

# Tail Jaeger container logs
obs-logs:
    docker logs -f rental-jaeger

# --- Compose: full local stack (app + Postgres + Jaeger + Loki + Promtail + Prometheus + Grafana) ---
#
# Conflicts with the standalone `just db-run` / `just obs-up` / `just docker-run` targets
# because the container names overlap. Stop those first if needed.

# Bring up the full stack, building the app image if it has changed
compose-up:
    docker compose up -d --build

# Tail app logs
compose-logs:
    docker compose logs -f app

# Tail logs for any service: `just compose-logs-svc svc=loki`
compose-logs-svc svc:
    docker compose logs -f {{ svc }}

# Show running services
compose-ps:
    docker compose ps

# Stop and remove containers (named volumes preserved — Postgres bind mount also preserved)
compose-down:
    docker compose down

# Stop and remove containers AND wipe named volumes (DESTRUCTIVE — Loki/Prometheus/Grafana data lost)
compose-nuke:
    docker compose down -v

# --- SQLx offline mode (required for Docker builds) ---

# Regenerate .sqlx/ query metadata from the live DB.
# Commit the resulting .sqlx/ directory so `cargo build` can run with SQLX_OFFLINE=true
# (i.e. inside the Docker builder stage, where no DB is reachable).
sqlx-prepare:
    cargo sqlx prepare --workspace -- --all-targets  

# --- Docker image (multi-stage cargo-chef + distroless) ---

image_name := env_var_or_default("IMAGE_NAME", "rental-api")
image_tag  := env_var_or_default("IMAGE_TAG",  "dev")

# Build the production image
docker-build:
    docker build -t {{ image_name }}:{{ image_tag }} .

# Run the image, talking to host Postgres + host Jaeger (macOS / Docker Desktop)
docker-run:
    docker run --rm --name rental-api \
        -p 8080:8080 \
        -e DATABASE_URL=postgres://rental:rental@host.docker.internal:5432/rental \
        -e OTEL_EXPORTER_OTLP_ENDPOINT=http://host.docker.internal:4317 \
        -e APP_NAME=rental-api \
        {{ image_name }}:{{ image_tag }}

# Show final image size
docker-size:
    docker images {{ image_name }}:{{ image_tag }}

# --- Seeder: bulk-load dummy data into the rental DB ---

# Truncate all data tables (countries, addresses, managers, persons, houses, calendar, bookings)
seed-reset:
    cargo run --release -p seeder -- reset

# Load full default volumes (the demo target: ~1k managers, 100k persons, 10k addresses, 10k houses)
seed-load:
    cargo run --release -p seeder -- load

# Load a tiny dataset for fast iteration / smoke tests
seed-load-small:
    cargo run --release -p seeder -- load --small

# Reset then load full volumes (the typical "fresh start")
seed-fresh: seed-reset seed-load

# Load only specific steps, e.g. `just seed-only houses,managers`
seed-only steps:
    cargo run --release -p seeder -- load --only {{ steps }}

# Skip specific steps, e.g. `just seed-skip persons,managers`
seed-skip steps:
    cargo run --release -p seeder -- load --skip {{ steps }}

# Print row counts per table
seed-verify:
    cargo run --release -p seeder -- verify

# --- Load test (goose) — requires the API to be running on localhost:8080 ---

# Show goose's CLI options (host, users, hatch-rate, run-time, report-file, ...)
loadtest-help:
    cargo run --release -p loadtest -- --help

# Quick smoke test: 5 users for 10 s — assumes seeded DB
loadtest-smoke:
    cargo run --release -p loadtest -- --users 5 --hatch-rate 5 --run-time 10s --no-reset-metrics

# Baseline (read-only mix): 50 users for 1 min — assumes seeded DB. Re-runnable.
loadtest-baseline:
    cargo run --release -p loadtest -- --users 50 --hatch-rate 10 --run-time 1m --report-file loadtest-report.html

# Headline run: reseed from scratch then run baseline (slow: ~12 min seed + 1 min test)
loadtest-fresh: seed-fresh loadtest-baseline

# Read + write mix (record_payment, create_booking). Each run consumes some unpaid
# bookings + free windows, so reseed every few runs for a clean baseline.
loadtest-write:
    cargo run --release -p loadtest -- --users 50 --hatch-rate 10 --run-time 3m --report-file loadtest-report-write.html

# Reseed + read+write mix in one shot.
loadtest-write-fresh: seed-fresh loadtest-write

# Free-form passthrough: `just loadtest --users 200 --run-time 5m --report-file out.html`
loadtest *args:
    cargo run --release -p loadtest -- {{ args }}

# CURL Test Commands

# API Health
api-health:
    curl -v -i localhost:8080/health

# Generate some traffic
api-traffic:
    curl -v -i http://localhost:8080/api/v1/bookings
    curl -v -i http://localhost:8080/api/v1/countries
