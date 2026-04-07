FROM rust
WORKDIR /app
COPY . .
RUN cargo build --release
RUN cargo run --release