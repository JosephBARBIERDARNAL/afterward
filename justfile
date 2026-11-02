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

# Run the tests, optionally filtered by name: `just test enemy_jumping`.
test filter="":
    cargo test --locked {{filter}}
