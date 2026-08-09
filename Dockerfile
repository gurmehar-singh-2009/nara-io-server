# Use official Rust image as builder
FROM rust:1.75-slim as builder

# Install build dependencies
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /app

# Copy Cargo.toml and Cargo.lock to cache dependencies
COPY Cargo.toml Cargo.lock ./

# Create dummy src/main.rs to cache dependencies
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf target/release/deps/your_project_name*

# Copy actual source code
COPY . .

# Build the actual binary
RUN cargo build --release

# Use slim Debian for runtime
FROM debian:bookworm-slim

# Install runtime dependencies (if any)
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl1.1 \
    && rm -rf /var/lib/apt/lists/*

# Copy binary from builder
COPY --from=builder /app/target/release/your_binary_name /usr/local/bin/app

# Expose WebSocket port
EXPOSE 8080

# Run the binary
CMD ["app"]
