FROM rust:latest AS builder
WORKDIR /app

COPY . .
RUN cargo build --release

FROM debian:trixie
WORKDIR /app

COPY --from=builder /app/target/release/minecraft-stats /app/minecraft-stats

CMD [ "/app/minecraft-stats" ]