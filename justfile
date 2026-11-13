# List available recipes.
default:
    @just --list

# Run the game. Pass a room ID to start new runs there: `just run 7`.
run room="1":
    cargo run -- --room {{room}}

# Run the game starting directly at the Chapter 1 boss.
boss:
    cargo run -- --room 10

# Capture screenshots of part of a run into target/visual-walkthrough:
# `just walkthrough menus`, `just walkthrough 3`, `just walkthrough 3-5`.
walkthrough what:
    cargo run -- --visual-walkthrough {{what}}

# Take a picture of whole rooms into target/room-shots, with no window:
# `just shot 3`, `just shot --grid 3 4` (ruled in H), `just shot` for every room.
shot *args:
    @scripts/room_shot.sh {{args}}

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

# Play all of Chapter 1 with the scripted test pilot. Ignored by `cargo test`
# while rooms are being designed, since it depends on their layouts.
playthrough:
    cargo test --locked chapter_one_run -- --ignored

# Run the tests, optionally filtered by name: `just test enemy_jumping`.
test filter="":
    cargo test --locked {{filter}}

# Format the Typst sources in art/ with typstyle (CI checks it).
art-fmt:
    typstyle -i art

# Recolour the robots' sheets into the dark robots' (enemies/type1_dark/).
# Needs ImageMagick; the PNGs are committed, so only after a source sheet changes.
dark-robots:
    python3 scripts/darken_robots.py

# Recolour the horror pack's monsters into their other palettes
# (enemies/horror/crimson/, cobalt/ and emerald/). Needs ImageMagick; the PNGs
# are committed, so only after a source sheet or a palette changes.
monsters:
    python3 scripts/recolour_monsters.py

# Cut the Black Knight's atlas into one strip per clip (enemies/black_knight/).
# Needs ImageMagick; the PNGs are committed, so only after the atlas or a clip changes.
black-knight:
    python3 scripts/slice_black_knight.py

# Render the Typst pixel art in art/ to PNGs at the same path under src/assets/
# (art/surfaces/textures/wood/planks.typ -> src/assets/surfaces/textures/wood/planks.png).
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
