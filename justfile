# --- Justfile for dnspod-ddns Project ---

# --- Variables ---
# The name of the binary executable.
BINARY_NAME := "ddns"
# Default build target for creating a portable, statically linked Linux executable.
# To use this, you must have the musl target installed:
# `rustup target add x86_64-unknown-linux-gnu`
STATIC_TARGET := "x86_64-unknown-linux-gnu"

# --- Code Quality & Testing ---

# Format the code using `rustfmt`.
fmt:
    @echo ">>> Formatting code..."
    @cargo fmt --all

# Lint the code with `clippy` to catch common mistakes and style issues.
lint:
    @echo ">>> Linting with clippy..."
    @cargo clippy --all-targets -- -D warnings

# Run all tests in the workspace.
test:
    @echo ">>> Running tests..."
    @cargo test --all-targets

# Perform a quick compilation check without building artifacts.
check:
    @echo ">>> Checking compilation..."
    @cargo check

# A convenient command for Continuous Integration (CI) pipelines.
# Runs formatter, linter, and tests.
ci: fmt lint test

# --- Building the Application ---

# Build the application in debug mode.
build:
    @echo ">>> Building in debug mode..."
    @cargo build

# Build the application in release mode for production use.
# The output will be optimized and located in `target/release/`.
build-release:
    @echo ">>> Building in release mode..."
    @cargo build --release

# Build a statically linked release binary using musl.
# This creates a highly portable executable for Linux environments like Docker or bare metal.
build-static:
    @echo ">>> Building statically linked release binary for target '{{STATIC_TARGET}}'..."
    @cargo build --release --target {{STATIC_TARGET}}

# --- Running the Application ---

# Run the application in debug mode.
# Pass arguments after `--`. Example: `just run -- --domain example.com ...`
run *ARGS:
    @echo ">>> Running in debug mode with args: {{ARGS}}"
    @cargo run -- {{ARGS}}

# Run the optimized release version of the application.
# Pass arguments after `--`. Example: `just run-release -- --domain example.com ...`
run-release *ARGS:
    @echo ">>> Running release build with args: {{ARGS}}"
    @cargo run --release -- {{ARGS}}

# --- Packaging and Deployment ---

# Create a distributable archive (tar.gz) containing the static binary.
# This is useful for distributing the application.
package: build-static
    @echo ">>> Packaging static binary into a tarball..."
    @mkdir -p dist
    @tar -czvf dist/{{BINARY_NAME}}-{{STATIC_TARGET}}.tar.gz -C target/{{STATIC_TARGET}}/release {{BINARY_NAME}}
    @echo "✅ Packaged successfully: dist/{{BINARY_NAME}}-{{STATIC_TARGET}}.tar.gz"

# --- Housekeeping ---

# Clean all build artifacts from the `target` directory.
clean:
    @echo ">>> Cleaning build artifacts..."
    @cargo clean

# Default command to run when `just` is invoked without arguments.
# It lists all available commands.
default:
    @just --list