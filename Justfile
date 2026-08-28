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

# Opens the editor. Debug builds generate too slowly to drag a slider against.
run *args:
    cargo run --release -- {{args}}

# Scaffolds assets/materials/<name>.wgsl from the template, ready to edit while
# the editor is running.
new-material name:
    #!/usr/bin/env bash
    set -euo pipefail
    target="assets/materials/{{name}}.wgsl"
    if [ -e "$target" ]; then
        echo "$target already exists" >&2
        exit 1
    fi
    sed 's/@material Template/@material {{name}}/' assets/materials/_template.wgsl > "$target"
    echo "wrote $target"

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
