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

# Run all tests
test:
    cargo test

# Run tests with output visible
test-verbose:
    cargo test -- --nocapture

# Run a single test by name (usage: just test-one name=my_test)
test-one name:
    cargo test {{ name }}

# Run tests for a specific module (usage: just test-mod mod=services::booking)
test-mod mod:
    cargo test {{ mod }}

# Build dev
build: 
    cargo build

# Build release
build-release: 
    cargo build --release

# Run clippy then all tests
check: lint test

# Run clippy
lint:
    cargo clippy -- -D warnings

# Format code
fmt:
    cargo fmt

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
    cargo sqlx prepare -- --all-targets

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

# CURL Test Commands

# API Health
api-health:
    curl -v -i localhost:8080/health

# Generate some traffic
api-traffic:
    curl -v -i http://localhost:8080/api/v1/bookings
    curl -v -i http://localhost:8080/api/v1/countries
