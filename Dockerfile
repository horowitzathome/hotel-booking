# syntax=docker/dockerfile:1.7

ARG RUST_VERSION=1.95
ARG APP_NAME=rental-api

# ---- chef base ----
FROM --platform=$BUILDPLATFORM rust:${RUST_VERSION}-slim-bookworm AS chef
RUN apt-get update && apt-get install -y --no-install-recommends \
    curl ca-certificates musl-tools clang \
    && rm -rf /var/lib/apt/lists/* \
    && cargo install cargo-chef --locked

WORKDIR /app

# ---- planner ----
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# ---- builder ----
FROM chef AS builder
ARG APP_NAME
# This ARG is automatically filled by Docker based on your computer
ARG TARGETARCH 

# Map Docker arch names to Rust target names
RUN if [ "$TARGETARCH" = "arm64" ]; then \
        echo "aarch64-unknown-linux-musl" > /rust_target; \
    else \
        echo "x86_64-unknown-linux-musl" > /rust_target; \
    fi && \
    rustup target add $(cat /rust_target)

COPY --from=planner /app/recipe.json recipe.json

# Build dependencies for the rental-api binary only — keeps loadtest/seeder
# deps out of the production image and avoids cross-compiling crates we don't ship.
RUN cargo chef cook --release --target $(cat /rust_target) --bin ${APP_NAME} --recipe-path recipe.json

COPY . .
ENV SQLX_OFFLINE=true

# Build the final binary
RUN cargo build --release --target $(cat /rust_target) --bin ${APP_NAME} \
    && cp target/$(cat /rust_target)/release/${APP_NAME} /app/server

# ---- runtime ----
FROM scratch AS runtime
WORKDIR /app
COPY --from=builder /app/server /app/server
COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/

ENV APP_ENV=production \
    SERVER_HOST=0.0.0.0 \
    SERVER_PORT=8080
    
EXPOSE 8080
ENTRYPOINT ["/app/server"]