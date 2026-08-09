# Use official Rust image as builder
FROM rust:1.75-slim as builder

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

# Use distroless or minimal image (no OpenSSL needed)
FROM gcr.io/distroless/cc-debian12

# Copy binary from builder
COPY --from=builder /app/target/release/your_binary_name /app

# Expose WebSocket port
EXPOSE 8080

# Run the binary
CMD ["/app"]
