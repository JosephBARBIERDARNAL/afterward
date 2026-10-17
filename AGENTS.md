# Too Late — Agent Notes

Updated: 2026-10-17

## Project status

The user wants a roguelike with no currency or power that persists between runs. The current prototype is five authored rooms; the end goal is 100 rooms across ten visual chapters, with solo bosses in rooms 10, 20, …, 100. The safe room appears at startup and between runs. Its door explicitly starts a fresh run. Room-clear bonuses exist only for the active run and are erased on death, Back Home, or restart.

The Bevy/Rust port is in progress. `src/main.rs` is the current Rust entry point. The Phaser implementation is temporarily retained as a parity reference and must be removed once the Bevy version covers the current game. Do not add new features to Phaser.

The current Bevy pass uses 2D shape primitives as placeholder art. The player can move, attack, dash, clear rooms, choose temporary bonuses, and navigate keyboard menus. Finish parity for authored obstacles and layouts, full enemy patterns and elite behaviors, the bonus cache, damage animation/audio, mouse/controller support, and browser/Wasm packaging. Ask the user to play the next usable Bevy milestone before tuning balance.

The ranged enemy's normal projectile aim is too predictable. Keep that behavior until the planned combat-tuning pass.

## Current stack

- Bevy 0.19.1 and Rust 2024.
- The current executable is a native Bevy app. WebAssembly/browser output remains a migration requirement.
- Use Bevy's built-in 2D primitives for placeholders. Start from the [Bevy learning index](https://bevy.org/assets/#learning), [2D assets index](https://bevy.org/assets/#2d), and [official 2D Shapes example](https://github.com/bevyengine/bevy/blob/latest/examples/2d/2d_shapes.rs). The assets index is a community plugin directory; do not mistake it for a sprite pack.
- Keep rendering and audio replaceable without coupling them to combat rules.
- Phaser, TypeScript, Vite, and Bun files remain only as temporary migration references.

## Project layout

- `src/main.rs` — current Bevy app and in-progress game loop.
- `game-specs.md` — authoritative product and scope specification.
- `TODO.md` — migration and content milestone tracking.
- `src/game/` — legacy Phaser implementation; remove after Bevy parity.

## Run and check

- `cargo run` — start the Bevy build.
- `cargo fmt --check` — check Rust formatting.
- `cargo clippy --all-targets -- -D warnings` — check Rust lint.
- `cargo check` — typecheck the Rust project.
- `bun run dev` and `bun run check` remain available for the legacy reference only.

## Scope and implementation rules

- Keep the full 100-room, ten-chapter goal and all ten solo bosses explicit without claiming they are implemented.
- Every run starts with base stats and no carried resources. Do not implement money, a shop, permanent upgrades, persistent unlocks, or bonuses that survive between runs.
- Keep current run-only bonus choices and clear them when a new run starts.
- Core combat retains directional movement and aim, a primary melee combo, a vulnerable heavy attack, a defensive dash, health/damage, six enemy types, two elite behaviors, and death/restart.
- Preserve arrow and ZQSD movement. Give each action one clear path per control scheme; avoid redundant shortcuts and duplicate click/key actions.
- Prefer data-driven tuning and authored arena geometry. Keep visual/audio assets behind presentation code so replacements do not change gameplay rules.
- Always remove dead code as part of each change.
- Do not claim bonuses or balance are tuned based on implementation alone; the user's play feedback is the gate for tuning.
