# syntax=docker/dockerfile:1.7

ARG RUST_VERSION=1.95
ARG APP_NAME=rental-api

# ---- chef base ----
FROM --platform=$BUILDPLATFORM rust:${RUST_VERSION}-slim-bookworm AS chef
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates python3 python3-pip \
    && rm -rf /var/lib/apt/lists/* \
    && pip3 install ziglang --break-system-packages \
    && cargo install cargo-chef cargo-zigbuild --locked

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
    fi

# Bring rust-toolchain.toml in before any cargo invocation so that rustup
# activates the pinned toolchain once; the subsequent target add and both
# cargo builds then all use the same toolchain installation.
COPY rust-toolchain.toml .
RUN rustup target add $(cat /rust_target)

COPY --from=planner /app/recipe.json recipe.json

RUN cargo chef cook --zigbuild --release --target $(cat /rust_target) --recipe-path recipe.json

COPY . .
ENV SQLX_OFFLINE=true

RUN cargo zigbuild --release --target $(cat /rust_target) --bin ${APP_NAME} \
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
