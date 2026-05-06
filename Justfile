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
