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

# Debug builds generate too slowly to drag a slider against.
# Opens the editor on a material project (a directory with a wesl.toml), else on example/.
run project="":
    cargo run --release -- {{ if project == "" { "" } else { quote(join(invocation_directory(), project)) } }}

# Combines material export folders into one image per map plus atlas.json.
combine *args:
    cargo run --release --bin combine -- {{args}}

# An open editor lists the new file after a restart.
# Scaffolds <name>.wesl from the template at the package root of a material project.
new-material project name:
    #!/usr/bin/env bash
    set -euo pipefail
    project={{ quote(join(invocation_directory(), project)) }}
    if [ ! -f "$project/wesl.toml" ]; then
        echo "$project has no wesl.toml" >&2
        exit 1
    fi
    if [[ ! "{{name}}" =~ ^[A-Za-z_][A-Za-z0-9_]*$ ]]; then
        echo "{{name}} is not a WESL identifier, so it cannot name a module" >&2
        exit 1
    fi
    root=$(sed -n 's/^root *= *"\(.*\)" *$/\1/p' "$project/wesl.toml")
    target="$project/${root:-shaders}/{{name}}.wesl"
    if [ -e "$target" ]; then
        echo "$target already exists" >&2
        exit 1
    fi
    sed 's/@material Template/@material {{name}}/' {{ quote(justfile_directory() / "example/shaders/_template.wesl") }} > "$target"
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
