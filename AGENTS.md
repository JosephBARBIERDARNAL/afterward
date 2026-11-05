## Game specifications

The entire game specification lives in `game-spec.md`. It is the reference for every behavior of the game: when the spec and the code disagree, ask which one is right; when a change alters behavior, update the spec in the same commit.

The game is in development, focused on Chapter 1 (rooms 1–10). Content and systems needed only for later chapters (rooms 11–100) are low priority unless explicitly requested.

## Run and check

Once you finish making a change (fixing a bug or adding a feature), make sure all code is well formatted, linted and tested. Everything should be green. This will run in CI too. Common commands:

- `cargo run` — start the Bevy build.
- `cargo fmt --all -- --check` — check Rust formatting.
- `cargo clippy --locked --all-targets -- -D warnings` — check Rust lint.
- `cargo test --locked` — run Rust tests.
- `cargo build --locked --bin afterward` — compile the native game.

## Rules

- When something is unclear, needs decision, is inconsistent or contradictory with something else, ask explicitly what should be done.
- When defining the number of enemies, room organization, etc., always do it in a consistent and centralized way in yaml files so that making changes in a specific room is easy. Those files should be the source of truth on the construction of the game and make the game a true data-driven game.
- All constant parameters (game dimensions, player stats, etc.) should be clearly defined in such way and easy to change.
- Always feel free to move, rename and change asset files for consistency and simplicity.
- Keep source files focused and reasonably sized. Split code into separate modules by responsibility instead of letting individual files grow too long.
- Follow established software engineering practices: keep responsibilities clear, reuse shared logic where appropriate, and keep configuration data in its designated source-of-truth files.
- Keep Clippy rules intact. Fix lint findings in code instead of lowering lint levels or adding lint suppressions to make checks pass.
- No free functions for validation or conversion. Define a newtype and implement `Deserialize`, `FromStr`, or `TryFrom` on it.
- A helper called once is inlined. This applies to test fixtures as much as to production code; a fixture used by two tests is still inlined into both.
- Combine `use` statements from one crate into a single `crate::{...}`.
- No accessor that only returns a field. Widen the field to `pub(crate)` instead of writing a getter around it.
- Prefer a tailored API over `std::mem::take`. `Option::take`, `Vec::split_off` and `HashMap::remove` name what moves and leave the container usable, where `mem::take` silently swaps in a `Default`.
- Treat every git change you did not explicitly make as belonging to someone else working on the project simultaneously; preserve it exactly and do not revert, restore, overwrite, format, or otherwise “clean up” that file.
- Don't add content to README.md; relevant development knowledge lives in AGENTS.md.
- When you finished implementing a feature or fixing a bug, make sure to commit it with a clear and concise commit message. Don't push to GitHub. Always git add all files, even the ones you didn't touch yourself, unless it looks like an error (for example a very large file or an unpacked asset archive).

## Bevy

The game is built using Bevy 0.20, in Rust. Here are relevant sources to look into:

- https://bevy.org/
- https://docs.rs/bevy/latest/bevy/
- https://bevy.org/learn/quick-start/getting-started/setup/
- https://bevy.org/learn/quick-start/getting-started/ecs/
- https://bevy.org/learn/quick-start/getting-started/plugins/
- https://bevy.org/learn/quick-start/getting-started/resources/

The game targets are MacOS, Windows and Linux (CI runs tests for all of them).

## Development

Common tasks are in the `justfile` (run `just` to list them):

- `just run` — play from room 1; `just run 7` starts new runs (and pause-menu restarts) in room 7.
- `just boss` — start at the Chapter 1 boss (room 10, with its intro).
- `just check` — formatting, Clippy, tests, and the native build, as in CI.

Without `just`, pass the flag directly: `cargo run -- --room 7`.

### Code layout

- `src/data.rs` loads and validates every YAML file under `src/assets/` into the `GameData` resource. `src/data/` holds one module per YAML source: `gameplay.rs` (`config/gameplay.yaml`), `enemies.rs`, `backgrounds.rs`, `bonuses.rs`, `rooms.rs`, `player_animations.rs`.
- `src/game/mod.rs` wires the `GamePlugin` (systems and states) and holds only shared items: `GameScreen`, `StartRoom`, `H_WORLD_UNITS`, small helpers. Components and resources live in the module that owns them (`session.rs` for `RunSession`, `sprites.rs` for sprite-sheet animation, `player/body.rs` for `PlayerBody`, `combat/enemy_body.rs` and `combat/components.rs` for combat entities, and so on) and are re-imported in `mod.rs` when other modules need them.
- Game modules reach shared items through `use super::*;`; add a `use` in `mod.rs` for a type that several modules need, or a direct `use crate::game::...` for a type only one other module needs.
- Tests live in `src/game/tests/` (gameplay, driven through a headless `App`) and in `#[cfg(test)]` modules next to the code they cover.
