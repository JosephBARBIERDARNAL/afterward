# List available recipes.
default:
    @just --list

# Run the game. Pass a room ID to start new runs there: `just run 7`.
run room="1":
    cargo run -- --room {{room}}

# Run the game starting directly at the Chapter 1 boss.
boss:
    cargo run -- --room 10

# Capture the visual walkthrough screenshots into target/visual-walkthrough.
walkthrough:
    cargo run -- --visual-walkthrough

# Format all Rust code.
fmt:
    cargo fmt --all

# Run every CI check: formatting, lint, tests, and the native build.
check:
    cargo fmt --all -- --check
    cargo clippy --locked --all-targets -- -D warnings
    cargo test --locked
    cargo build --locked --bin afterward

# Run the end-to-end scenarios: the real game, headless, played through key events.
e2e:
    cargo test --locked e2e::

# Run the tests, optionally filtered by name: `just test enemy_jumping`.
test filter="":
    cargo test --locked {{filter}}

# Format the Typst sources in art/ with typstyle (CI checks it).
art-fmt:
    typstyle -i art

# Render the Typst pixel art in art/ to PNGs at the same path under src/assets/
# (art/terrain/textures/wood/planks.typ -> src/assets/terrain/textures/wood/planks.png).
# Files starting with `_` are shared libraries, not images.
art:
    #!/usr/bin/env sh
    set -eu
    find art -name '*.typ' ! -name '_*' | while read -r src; do
        out="src/assets/${src#art/}"
        mkdir -p "$(dirname "$out")"
        typst compile --root art --format png --ppi 72 "$src" "${out%.typ}.png"
        echo "$src -> ${out%.typ}.png"
    done
