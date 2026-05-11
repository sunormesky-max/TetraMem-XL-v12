FROM rust:slim-bookworm AS builder

RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
RUN mkdir -p src benches tests && \
    echo "fn main() {}" > src/main.rs && \
    echo "" > benches/benchmarks.rs && \
    echo "" > tests/full_suite.rs && \
    echo "" > tests/scale_bench.rs && \
    echo "" > tests/stress_test.rs && \
    cargo build --locked --release && rm -rf src benches tests

COPY src/ src/
RUN mkdir -p benches tests && \
    echo "" > benches/benchmarks.rs && \
    echo "" > tests/full_suite.rs && \
    echo "" > tests/scale_bench.rs && \
    echo "" > tests/stress_test.rs && \
    touch src/main.rs && cargo build --locked --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

RUN groupadd -r tetramem && useradd -r -g tetramem -d /app tetramem

COPY --from=builder /app/target/release/tetramem-v12 /usr/local/bin/tetramem-v12

RUN mkdir -p /app/backups /app/data && chown -R tetramem:tetramem /app

USER tetramem
WORKDIR /app

EXPOSE 3456
HEALTHCHECK --interval=30s --timeout=5s --retries=3 \
    CMD /usr/local/bin/tetramem-v12 health --addr http://localhost:3456 || exit 1

ENTRYPOINT ["tetramem-v12"]
CMD ["serve"]
