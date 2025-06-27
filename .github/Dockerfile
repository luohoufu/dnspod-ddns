# --- Stage 1: The Builder ---
# We use the official Rust image as a build environment.
# Using a specific version ensures reproducible builds.
# Using the slim-bullseye variant is a good balance of size and functionality.
FROM rust:1.78-slim-bullseye AS builder

# Set the working directory inside the container.
WORKDIR /usr/src/app

# Install dependencies needed for static linking with musl.
RUN apt-get update && apt-get install -y musl-tools

# Add the musl target to rustup.
RUN rustup target add x86_64-unknown-linux-musl

# Copy the Cargo.toml and Cargo.lock files to cache dependencies.
# This layer is only re-built when these files change.
COPY Cargo.toml Cargo.lock ./

# Create a dummy src/main.rs to build only the dependencies.
# This is another caching optimization.
RUN mkdir src && \
  echo "fn main() {}" > src/main.rs && \
  cargo build --release --target x86_64-unknown-linux-musl && \
  rm -rf src

# Copy the actual source code into the container.
COPY src ./src

# Build the application for release, statically linked against musl.
# The --release flag enables optimizations.
RUN cargo build --release --target x86_64-unknown-linux-musl


# --- Stage 2: The Final Image ---
FROM alpine:latest

# Alpine uses musl libc, so it's compatible with our static build.
# We add a non-root user for security.
RUN addgroup -S appgroup && adduser -S appuser -G appgroup

# Copy the compiled binary from the builder stage.
COPY --from=builder /usr/src/app/target/x86_64-unknown-linux-musl/release/ddns /usr/local/bin/ddns

# Switch to the non-root user.
USER appuser

# Set the entrypoint for the container.
ENTRYPOINT ["ddns"]