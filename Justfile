# Every cargo invocation this project is developed with, so none of them have to
# be remembered as flags.

default:
    @just --list

build:
    cargo build

# Fails on any warning, so this is the gate a change has to pass.
clippy:
    cargo clippy --all-targets --all-features -- -D warnings

test:
    cargo test

check: build clippy

# Release build; args are the generator's own flags, e.g. `-r 2048 -t rocky`.
run *args:
    cargo run --release -- {{args}}

fmt:
    cargo fmt

clean:
    cargo clean

# Opens the rendered docs in a browser.
docs:
    cargo doc --open

# Rewrites Cargo.lock.
update:
    cargo update
