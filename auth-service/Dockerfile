# Start with image that has the Rust toolchain installed
FROM rust:1.90-alpine AS chef
USER root
# Add cargo-chef to cache dependencies
RUN apk add --no-cache musl-dev cmake perl make gcc g++
RUN rustup update stable
RUN cargo install cargo-chef --locked
WORKDIR /app

FROM chef AS planner
COPY ./auth-service/ ./auth-service/
COPY ./test_helpers/ ./test_helpers/
WORKDIR /app/auth-service
# Capture info needed to build dependencies
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
WORKDIR /app
# Copy test_helpers so cargo can resolve the path dependency
COPY ./test_helpers/ ./test_helpers/
WORKDIR /app/auth-service
COPY --from=planner /app/auth-service/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json
# Build application
COPY ./auth-service/ /app/auth-service/
ENV SQLX_OFFLINE=true
RUN cargo build --release --bin auth-service

# We do not need the Rust toolchain to run the binary!
# Start with a minimal image and copy over the binary and assets folder.
FROM alpine:3.18 AS runtime
RUN apk add --no-cache libgcc
WORKDIR /app
COPY --from=builder /app/auth-service/target/release/auth-service /usr/local/bin
COPY --from=builder /app/auth-service/assets /app/assets
ENV REDIS_HOST_NAME=redis
ENTRYPOINT ["/usr/local/bin/auth-service"]