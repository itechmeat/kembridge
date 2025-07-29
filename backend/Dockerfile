FROM rust:1.88.0 AS chef
RUN cargo install cargo-chef cargo-watch
WORKDIR /app

# Plan dependencies
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Build dependencies
FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# Build application
COPY . .
RUN cargo build --release --bin kembridge-backend

# Development stage
FROM rust:1.88.0 AS development
WORKDIR /app
RUN cargo install cargo-watch
COPY . .
EXPOSE 3000
# Use a more stable approach with restart wrapper
CMD ["sh", "-c", "while true; do cargo run --bin kembridge-backend || echo 'Backend crashed, restarting in 5 seconds...'; sleep 5; done"]

# Production stage
FROM debian:bookworm-slim AS production
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/kembridge-backend /app/kembridge-backend
EXPOSE 3000
CMD ["/app/kembridge-backend"]