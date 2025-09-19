# Multi-stage Dockerfile for Aprio Swarm
FROM rust:1.75-slim as builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY crates/ ./crates/
COPY examples/ ./examples/

# Build the workspace
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -r -s /bin/false appuser

# Set working directory
WORKDIR /app

# Copy built binaries
COPY --from=builder /app/target/release/nats-publisher /app/
COPY --from=builder /app/target/release/nats-subscriber /app/
COPY --from=builder /app/target/release/simple-document-demo /app/

# Create test data directory
RUN mkdir -p /app/test-data && chown -R appuser:appuser /app

# Switch to non-root user
USER appuser

# Default command
CMD ["./nats-subscriber"]
