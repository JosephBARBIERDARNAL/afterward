## Game specifications

The entire game specification lives in `game-spec.md`. It is the reference for every behavior of the game: when the spec and the code disagree, ask which one is right; when a change alters behavior, update the spec in the same commit.

The game is in development, focused on Chapter 1 (rooms 1–10). Content and systems needed only for later chapters (rooms 11–100) are low priority unless explicitly requested.

The project is at an early, experimental stage. Many choices (assets, room layouts and materials, tuning values, editor features, even parts of the spec) are trials that may be reworked or dropped soon. Expect them to change: prefer simple, easy-to-replace solutions over heavy abstractions or polish, don't treat the current state as a settled design, and when something looks like a placeholder or a half-made decision, ask rather than build on it as final.

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

### End-to-end tests

`src/game/tests/e2e.rs` runs the real `GamePlugin` inside Bevy's default plugins, headless (no window, no GPU, no audio device), at a fixed 60 FPS with a fixed random seed. Scenarios in `src/game/tests/e2e/` play it through keyboard events (`tap`, `hold`, `typed_key` for other layouts) and check the screen and run state. Run them with `just e2e`; CI runs them on Linux, macOS, and Windows as their own step.

- Add a scenario for every player-facing flow or input bug: menus, pause, death, the bonus screen, the editor's shortcuts.
- Scenarios must survive room edits: `defeat_enemies` and `step_toward_exit` are the only shortcuts, for steps whose real input depends on a room's layout. Don't assert on specific spawn positions.
- `PreferencesDir(None)` keeps tests away from the player's saved settings and bindings. `HeadlessGame::with_copied_data` points the YAML data (and editor saves) at a temporary copy, so tests never write to `src/assets`.

### Debug room editor

Debug builds (`cargo run`, `just run`) add `D  Debug editor` to the main menu (opens the start room) and the pause menu (opens the room being played). The editor lives in `src/game/editor/` and is compiled out of release builds (`#[cfg(debug_assertions)]`).

- Shortcuts take Cmd on macOS and Ctrl elsewhere (both work everywhere; the help shows the platform's). They read `KeyboardInput` events rather than `ButtonInput`, because macOS sends no key-up for keys released while Cmd is held. Letter shortcuts match the character the key types on the active layout (`KeyboardInput::logical_key`), not its physical position, so Cmd+Z is Z on AZERTY too.
- It edits the rooms in `GameData` in place: Cmd+R playtests the edited room immediately (and makes it the start room, so pause-menu restarts replay it), and the edits stay in memory until the game quits.
- Cmd+S validates all game data with `GameData::validate` (the same checks as at startup) and writes each changed room back to `src/assets/rooms/room_NNN.yaml` through the `Display` impl in `src/data/room_yaml.rs`. Saving is refused while the data is invalid; the inspector shows the problem as soon as an edit causes it.
- Saved files use one canonical layout. Keep notes about blocks and enemies in `label:` fields and the room's idea in `description:`; per-entry YAML comments would be lost on the next save. When a room field is added, update `room_yaml.rs` too (its round-trip test catches fields it drops).
- The on-screen help lists every control: click to select, Shift+click to add or remove, drag empty space to box-select, drag to move the whole selection (Alt for the fine grid), drag a selected block's edge to resize, two-finger trackpad scroll or right/middle drag to pan, pinch or mouse wheel to zoom, T/P/D/E to add terrain/platform/decoration/enemy at the cursor, Cmd+C/X/V to copy, cut, and paste (into any room, at the cursor), Cmd+A to select all, Cmd+D to duplicate, arrows for the inspector (room size, background, material, decoration layer, enemy type, stat overrides, labels; Material and X/Y apply to the whole selection).
- The material palette (top right) shows every material of `terrain/materials.yaml`: clicking a swatch paints the selected blocks and decorations and becomes the material new ones use. Clicks over a panel never reach the room.
- The key that opens the editor from a menu (D) is ignored on the frame it opens, so it doesn't also add a decoration.

### Typst pixel art

Some assets are drawn in code with [Typst](https://typst.app) instead of a paint program: geometric, grid-aligned art such as the `wood`, `metal`, and `carved` terrain textures and the HUD pieces in `src/assets/ui/`. Sources live in `art/`, mirroring their output path under `src/assets/` (`art/terrain/textures/wood/planks.typ` renders `src/assets/terrain/textures/wood/planks.png`).

- `just art` renders every source (needs `typst` 0.15.1). The PNGs are committed, so building the game never needs Typst; rerun `just art` and commit both after editing a source. CI (`.github/workflows/art.yml`) re-renders with the same pinned Typst version and fails if any PNG differs from its source; bump `TYPST_VERSION` there when upgrading Typst locally.
- `art/_pixel.typ` makes a page exactly the image's size and draws 1 pt squares, rendered at 72 ppi: one square per pixel, no anti-aliasing. `pixels` draws text art (one character per pixel, looked up in a palette); `pixels-by` computes each pixel from a function.
- Files starting with `_` are shared libraries, not images: `art/_palettes.typ` for palettes used across families, `_<family>.typ` for a family's own palettes and patterns. A palette swap is a second short source reusing the same pattern (`dark_planks.typ`, `rusty_plates.typ`).
- Import libraries with relative paths.
- UI images load through `ui_style::load_pixel_art` (nearest sampling, so they stay crisp scaled up). Gray images can be tinted with `ImageNode::color`, like the ammo round.
- Keep to axis-aligned squares on whole points: curves, diagonal strokes, and text come out anti-aliased.

### Code layout

- `src/data.rs` loads and validates every YAML file under `src/assets/` into the `GameData` resource. `src/data/` holds one module per YAML source: `gameplay.rs` (`config/gameplay.yaml`), `enemies.rs`, `backgrounds.rs`, `bonuses.rs`, `rooms.rs`, `terrain.rs` (`terrain/materials.yaml`), `player_animations.rs`.
- `src/game/terrain.rs` holds room collision; `src/game/terrain_art.rs` draws blocks, platforms, and decorations (world-aligned tiled textures via a repeat sampler and `Sprite::rect`, top caps, outlines of uncovered edges).
- `src/game/mod.rs` wires the `GamePlugin` (systems and states) and holds only shared items: `GameScreen`, `StartRoom`, `H_WORLD_UNITS`, small helpers. Components and resources live in the module that owns them (`session.rs` for `RunSession`, `sprites.rs` for sprite-sheet animation, `player/body.rs` for `PlayerBody`, `combat/enemy_body.rs` and `combat/components.rs` for combat entities, and so on) and are re-imported in `mod.rs` when other modules need them.
- Game modules reach shared items through `use super::*;`; add a `use` in `mod.rs` for a type that several modules need, or a direct `use crate::game::...` for a type only one other module needs.
- Tests live in `src/game/tests/` (gameplay, driven through a headless `App`) and in `#[cfg(test)]` modules next to the code they cover.
