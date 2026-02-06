# Build stage
FROM rust:1.84-bookworm AS builder

WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src
COPY migrations ./migrations
COPY sqlx.toml ./sqlx.toml

# Build for release
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the binary from builder
COPY --from=builder /app/target/release/budget /app/budget

# Copy migrations
COPY --from=builder /app/migrations /app/migrations

# Copy configuration examples (will be overridden by env vars)
COPY Budget.toml.example ./Budget.toml
COPY Rocket.toml ./Rocket.toml

# Create a non-root user
RUN useradd -m -u 1000 appuser && chown -R appuser:appuser /app
USER appuser

# Expose the default port
EXPOSE 8000

# Run the binary
CMD ["/app/budget"]
