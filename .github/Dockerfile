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
# We use the `scratch` image, which is the smallest possible image.
# It's completely empty, perfect for a single, static binary.
FROM scratch

# Copy the compiled, static binary from the builder stage.
COPY --from=builder /usr/src/app/target/x86_64-unknown-linux-musl/release/ddns /ddns

# Set the user to a non-root user for better security.
# Since `scratch` doesn't have users, we specify a user ID.
# This requires the container runtime to support `-u <uid>`.
# A value > 1000 is common for non-system users.
USER 1001

# Set the entrypoint for the container. This is the command that will run
# when the container starts.
ENTRYPOINT ["/ddns"]