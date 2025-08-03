# Fast build Dockerfile for 1inch Service
FROM rust:1.88.0-slim as builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy only the service and common library
COPY kembridge-common ./kembridge-common
COPY kembridge-1inch-service .

# Build with optimization for faster compilation
ENV CARGO_INCREMENTAL=1
RUN cargo build --release

# Runtime image
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    nodejs \
    npm \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/kembridge-1inch-service /usr/local/bin/

EXPOSE 4001

HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD curl -f http://localhost:4001/health || exit 1

CMD ["kembridge-1inch-service"]