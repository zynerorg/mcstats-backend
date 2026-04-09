FROM rust:latest AS builder
WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY migration ./migration
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src
COPY . .
RUN cargo build --release

FROM debian:trixie
WORKDIR /app

COPY --from=builder /app/target/release/minecraft-stats /app/minecraft-stats

CMD [ "/app/minecraft-stats" ]