# Justfile for bevy_pbr_generator
# Run with: just <command>

# Default recipe - build and check
default:
    @just --list

# Build the project
build:
    cargo build

# Run clippy linter
clippy:
    cargo clippy --all-targets --all-features -- -D warnings

# Run tests (if any)
test:
    cargo test

# Check the project (build + clippy)
check: build clippy

# Run the application with default settings
run:
    cargo run --release

# Run with custom resolution
run-res *args:
    cargo run --release -- {{args}}

# Format code
fmt:
    cargo fmt

# Clean build artifacts
clean:
    cargo clean

# Generate documentation
docs:
    cargo doc --open

# Update dependencies
update:
    cargo update

