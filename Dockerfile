# Use official Rust image as builder
FROM rust:1.85-slim AS builder

# Install build dependencies (INCLUDING cmake, make, perl for aws-lc-sys)
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    build-essential \
    cmake \
    perl \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /app

# Copy entire workspace
COPY . .

# Build the server binary
RUN cargo build --release -p server

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Copy the server binary
COPY --from=builder /app/target/release/server /usr/local/bin/app

EXPOSE 8080
CMD ["app"]
