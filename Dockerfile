# Stage 1: Build the Rust application
FROM rust:1.85.0-slim-bookworm AS builder

# Install system dependencies required for building the Rust application
# - pkg-config: helps find installed libraries
# - libssl-dev: OpenSSL development files for TLS/crypto support
RUN apt-get update && apt-get install -y \
  pkg-config \
  libssl-dev \
  && rm -rf /var/lib/apt/lists/* # Clean up to reduce image size

# Set the working directory for the build stage
WORKDIR /app

# Copy the entire project into the container
COPY . .

# Build the application in release mode
# --release: enables optimizations
# --verbose: shows detailed build output
RUN cargo build --release --verbose
# RUN ls -al /app/target
# RUN ls -al /app/target/release || echo "Release directory not found"

# Stage 2: Create the final lightweight runtime image
FROM debian:bookworm-slim

# Install only the runtime dependencies needed by the application
# - ca-certificates: for HTTPS connections
# - libpq-dev: PostgreSQL client libraries
# - libssl3: OpenSSL runtime libraries
RUN apt-get update && apt-get install -y \
  ca-certificates \
  libpq-dev \
  libssl3 \
  && rm -rf /var/lib/apt/lists/* # Clean up to reduce image size

WORKDIR /app
COPY --from=builder /app/target/release/tokencheck-subscriptions /app/tokencheck-subscriptions

# Document which port the application listens on
EXPOSE 8080

# Define the command to run when the container starts
CMD ["./tokencheck-subscriptions"]
