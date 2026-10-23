# Developing this repo: every cargo invocation it is built, checked and run with.
# Path arguments resolve against the directory just is called from, as they do
# for the installed binaries. Other projects use those binaries, not this file.

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

manifest := justfile_directory() / "Cargo.toml"

# Opens the editor on a project, by default example/; only this way do src/shaders edits hot-reload.
[no-cd]
run project=(justfile_directory() / "example"):
    cargo run --release --manifest-path {{ quote(manifest) }} -- {{ quote(project) }}

# Makes a directory a material project, or refreshes its copy of the package.
[no-cd]
init project=".":
    cargo run --release --manifest-path {{ quote(manifest) }} -- init {{ quote(project) }}

# Writes <name>.wesl from the material template into a project, by default example/.
[no-cd]
new-material name project=(justfile_directory() / "example"):
    cargo run --release --manifest-path {{ quote(manifest) }} -- new {{ quote(name) }} {{ quote(project) }}

# Combines material export folders into one image per map plus atlas.json.
[no-cd]
combine *args:
    cargo run --release --manifest-path {{ quote(manifest) }} --bin bevy_pbr_combine -- {{ args }}

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
