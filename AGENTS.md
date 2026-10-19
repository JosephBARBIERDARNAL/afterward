## Game specifications

The entire game specifications live in `game-spec.md`. This is the reference for every specifications of the game.

We are currently in the phase of defining the entire game specification, not development.

## Run and check

Once you finish making a change (fixing a bug or adding a feature), make sure all code is well formatted, linted and tested. Everything should be green. This will run in CI too. Common commands:

- `cargo run` — start the Bevy build.
- `cargo fmt --all -- --check` — check Rust formatting.
- `cargo clippy --locked --all-targets -- -D warnings` — check Rust lint.
- `cargo test --locked` — run Rust tests.
- `cargo build --locked --bin afterward` — compile the native game.

## Rules

- When something is unclear, needs decision, is inconsistent or contradictory with something else, ask what should be done explicitely.
- When defining number of ennemies, room organization, etc, always make sure to do that is a consistent and centralized way in yaml files so that making changes in a specific room is easy. Those files should be the source of truth on the construction of the game and make the game a true data-driven game.
- All constant parameters (game dimension, player stats, etc) should be clearly defined in such way and easy to change.
- Always feel free to move, rename and change asset files for consistency and simplicity.w
- Keep source files focused and reasonably sized. Split code into separate modules by responsibility instead of letting individual files grow too long.
- Follow established software engineering practices: keep responsibilities clear, reuse shared logic where appropriate, and keep configuration data in its designated source-of-truth files.
- Keep Clippy rules intact. Fix lint findings in code instead of lowering lint levels or adding lint suppressions to make checks pass.
- No free functions for validation or conversion. Define a newtype and implement `Deserialize`, `FromStr`, or `TryFrom` on it
- A helper called once is inlined. This applies to test fixtures as much as to production code; a fixture used by two tests is still inlined into both.
- Combine `use` statements from one crate into a single `crate::{...}`.
- No accessor that only returns a field. Widen the field to `pub(crate)` instead of writing a getter around it.
- Prefer a tailored API over `std::mem::take`. `Option::take`, `Vec::split_off` and `HashMap::remove` name what moves and leave the container usable, where `mem::take` silently swaps in a `Default`.

## Bevy

The game is built using Bevy 0.20, in Rust. Here are relevant sources to look into:

- https://bevy.org/
- https://docs.rs/bevy/latest/bevy/
- https://bevy.org/learn/quick-start/getting-started/setup/
- https://bevy.org/learn/quick-start/getting-started/ecs/
- https://bevy.org/learn/quick-start/getting-started/plugins/
- https://bevy.org/learn/quick-start/getting-started/resources/

The game targets are MacOS, Windows and Linux (CI runs tests for all of them).
