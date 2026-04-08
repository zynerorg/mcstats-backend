FROM rust:latest AS builder
WORKDIR /app

RUN apt-get update && apt-get install -y libpq-dev && rm -rf /var/lib/apt/lists/*

COPY . .
RUN cargo build --release --bin syncer --bin api

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y libpq5 && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/syncer /app/syncer
COPY --from=builder /app/target/release/api /app/api

ENV RUST_LOG=info
ENV WORLD_PATH=/app/world

EXPOSE 3000

LABEL maintainer="oliver@zyner.org"