FROM rust:1.80-slim-bullseye AS builder

WORKDIR /app
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml ./
COPY migrations ./migrations
COPY src ./src
COPY tests ./tests

RUN cargo build --release

FROM debian:bullseye-slim

WORKDIR /app
RUN apt-get update && apt-get install -y ca-certificates curl libssl1.1 && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/llmops-backend /app/llmops-backend
COPY --from=builder /app/migrations /app/migrations

EXPOSE 8210
ENV SERVER_HOST=0.0.0.0
ENV SERVER_PORT=8210
ENV DATABASE_URL=sqlite://data/llmops.db?mode=rwc

CMD ["/app/llmops-backend"]
