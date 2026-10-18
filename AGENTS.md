## Game specifications

The entire game specifications live in gdd/main.md.

## Run and check

Once you finish making a change (fixing a bug or adding a feature), make sure all code is well formatted, linted and tested. Everything should be green. This will run in CI too. Common commands:

- `cargo run` — start the Bevy build.
- `cargo fmt --all -- --check` — check Rust formatting.
- `cargo clippy --locked --all-targets -- -D warnings` — check Rust lint.
- `cargo test --locked` — run Rust tests.
- `cargo build --locked --bin too-late` — compile the native game.
