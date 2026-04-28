FROM rust:1.95.0-slim-bookworm AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && cargo build --release && rm -rf src

COPY src/ src/
COPY tests/ tests/
RUN touch src/main.rs && cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/tetramem-v12 /usr/local/bin/tetramem-v12

RUN mkdir -p /app/backups /app/data
WORKDIR /app

EXPOSE 3456

ENTRYPOINT ["tetramem-v12"]
CMD ["serve"]
