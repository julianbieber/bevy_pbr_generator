# Justfile for bevy_pbr_generator
# Run with: just <command>

# Default recipe - build and check
default:
    just check

# Build the project
build:
    cargo build

# Run clippy linter
clippy:
    cargo clippy --all-targets --all-features -D warnings

# Run tests (if any)
test:
    cargo test

# Check the project (build + clippy)
check: build clippy

# Run the application with default settings
run:
    cargo run

# Run with custom resolution
run-res *args:
    cargo run -- {{args}}

# Format code
fmt:
    cargo fmt

# Clean build artifacts
clean:
    cargo clean

# Generate documentation
 docs:
    cargo doc --open

# Install the binary locally
install:
    cargo install --path .

# Update dependencies
update:
    cargo update

# Show help
help:
    @just --list
