default:
    @just --list

run:
    cargo run

build:
    cargo build

clippy:
    cargo clippy --all-targets --all-features -- -D warnings

test:
    cargo test

check: build clippy test

fmt:
    cargo fmt

clean:
    cargo clean

update:
    cargo update

new-material name:
    #!/usr/bin/env bash
    set -euo pipefail
    target="assets/materials/{{name}}.wgsl"
    if [ -e "$target" ]; then echo "$target already exists" >&2; exit 1; fi
    sed "s/@material Template/@material {{name}}/" assets/materials/_template.wgsl > "$target"
    echo "$target"

check-placeholders:
    @rg -n 'TODO\(jb-(comment|doc)\)' --glob '!Justfile' || echo "no placeholders"
