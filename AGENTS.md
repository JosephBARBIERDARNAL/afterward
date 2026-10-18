# Afterward — Agent Notes

Updated: 2026-10-17

## Project status

The user wants a roguelike with no currency or power that persists between runs. The current prototype is five authored rooms; the end goal is 100 rooms across ten visual chapters, with solo bosses in rooms 10, 20, …, 100. The safe room appears at startup and between runs. Its door explicitly starts a fresh run. Room-clear bonuses exist only for the active run and are erased on death, Back Home, or restart.

The Bevy/Rust native desktop implementation is the only game client. Browser/Wasm builds and controller input are outside the current scope. `src/main.rs` is a thin entry point; application setup, game systems, and presentation live in focused modules.

The current Bevy build uses 2D shape primitives as placeholder art. It has the safe room, all five authored room layouts and obstacles, obstacle collision and line-of-sight, the Crossing bonus cache, run-only bonuses, and keyboard pause/bonus menus. The primary melee combo uses a broad horizontal sword sweep. Player and enemy health bars, animated dealt/received damage numbers, a visible directional weapon and action cues, sharper UI text, six enemy patterns, both elite behaviors, and generated sound cues are implemented. The former Phaser and TypeScript implementation has been removed. Playtest the migrated slice before tuning balance.

The ranged enemy's normal projectile aim is too predictable. Keep that behavior until the planned combat-tuning pass.

## Current stack

- Bevy 0.19.1 and Rust 2024.
- The current executable is a native Bevy app. Browser/Wasm and controller support are outside the migration scope.
- The native window opens at 1280×720; the camera zoom preserves the prototype's 960×540 arena framing.
- Use Bevy's built-in 2D primitives for placeholders. Start from the [Bevy learning index](https://bevy.org/assets/#learning), [2D assets index](https://bevy.org/assets/#2d), and [official 2D Shapes example](https://github.com/bevyengine/bevy/blob/latest/examples/2d/2d_shapes.rs). The assets index is a community plugin directory; do not mistake it for a sprite pack.
- Keep rendering and audio replaceable without coupling them to combat rules.
- The five-room build is a prototype slice; the 100-room, ten-chapter run remains the long-term content goal.

## Project layout

- `src/main.rs` — minimal executable entry point.
- `src/app.rs` — Bevy window configuration and app startup.
- `src/bevy_game/mod.rs` — game plugin and ordered system schedule.
- `src/bevy_game/model.rs` — game state and gameplay components.
- `src/bevy_game/arena.rs` — authored room layouts, spawning, and obstacle geometry.
- `src/bevy_game/combat.rs` and `src/bevy_game/combat/enemies.rs` — player combat, enemy behavior, and projectile resolution.
- `src/bevy_game/progression.rs` — run resets and temporary bonus rules.
- `src/bevy_game/ui.rs` — menus, HUD, health displays, and text.
- `src/presentation/` — replaceable visual and audio effects; keep combat rules out of this layer.
- `game-specs.md` — authoritative product and scope specification.
- `TODO.md` — migration and content milestone tracking.

## Run and check

- `cargo run` — start the Bevy build.
- `cargo fmt --all -- --check` — check Rust formatting.
- `cargo clippy --locked --all-targets -- -D warnings` — check Rust lint.
- `cargo test --locked` — run Rust tests.
- `cargo build --locked --bin too-late` — compile the native game.

## Scope and implementation rules

- Keep the full 100-room, ten-chapter goal and all ten solo bosses explicit without claiming they are implemented.
- Every run starts with base stats and no carried resources. Do not implement money, a shop, permanent upgrades, persistent unlocks, or bonuses that survive between runs.
- Keep current run-only bonus choices and clear them when a new run starts.
- Core combat retains directional movement and mouse aim, a primary melee combo, a vulnerable heavy attack, a defensive dash, health/damage, six enemy types, two elite behaviors, and death/restart.
- Preserve arrow and ZQSD movement. Bevy `KeyCode` is physical, so use `KeyW`/`KeyA`/`KeyS`/`KeyD` to match AZERTY's labeled Z/Q/S/D keys. Give each action one clear keyboard path; avoid redundant shortcuts and duplicate click/key actions.
- Prefer data-driven tuning and authored arena geometry. Keep visual/audio assets behind presentation code so replacements do not change gameplay rules.
- Always remove dead code as part of each change.
- Do not claim bonuses or balance are tuned based on implementation alone; the user's play feedback is the gate for tuning.
