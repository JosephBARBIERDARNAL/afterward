# Too Late

**Too Late** is a single-player action roguelike in active migration from Phaser/TypeScript to Bevy/Rust. The game is a five-room prototype today; the end goal remains a 100-room run across ten visual chapters, with a solo boss every tenth room.

## Bevy development

Install a current stable Rust toolchain, then run:

```sh
cargo run
```

The Rust port uses Bevy 0.19 and built-in 2D shape primitives as replaceable placeholders. The [Bevy learning resources](https://bevy.org/assets/#learning), [2D assets index](https://bevy.org/assets/#2d), and [2D Shapes example](https://github.com/bevyengine/bevy/blob/latest/examples/2d/2d_shapes.rs) are documented in [`game-specs.md`](./game-specs.md). The 2D assets index lists community plugins rather than art packs.

The native window opens at 1280×720. Its camera keeps the existing arena framing while rendering the game larger.

The Rust entry point in `src/main.rs` only starts the app. `src/app.rs` configures the window, `src/bevy_game/` owns game state and gameplay systems, and `src/presentation/` contains replaceable visuals and generated audio. Arena layout, combat, progression, and UI are separated so gameplay rules do not depend on rendering details.

Run Rust checks with:

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo check
```

The browser/Wasm target remains part of the migration scope and is not configured yet. The previous Phaser build is temporarily retained for parity reference while the port is incomplete; its `bun run dev` and `bun run check` commands still work during that period.

## Controls

| Action | Keyboard / mouse |
| --- | --- |
| Move | Arrow keys or ZQSD (AZERTY) |
| Aim | Mouse |
| Primary combo | J |
| Heavy attack | K |
| Dash | Space |
| Interact with the door | E |
| Choose a run bonus | Left/right arrows + Enter |
| Pause menu | Esc |

Movement and aiming are directional. The safe-room door starts a run. After clearing a non-final room, approach its east door and press E to choose a temporary bonus. Every bonus and stat change resets when a new run starts. There is no money, shop, permanent upgrade, or progression saved between runs.

On AZERTY keyboards, movement follows the labeled ZQSD keys; arrow keys also work.

## Current migration status

The Bevy implementation is a work in progress. It has the safe room, five authored rooms with visible obstacles and collision, the Crossing bonus cache, six enemy patterns, two elite behaviors, core movement and attacks, visible player and enemy health bars, a directional weapon with action cues, clearer UI text, animated damage numbers, generated gameplay sound cues, room-clear doors, temporary bonuses, and keyboard pause/selection menus. Controller input and browser/Wasm packaging still need parity work. The legacy Phaser app remains available as a reference until that work is done. Playtest the migrated rooms and cache before tuning their balance.

The ranged enemy's normal projectile aim is intentionally still too predictable; revisit it during a dedicated combat-tuning pass. Ask the user to play the migrated build at the next playable milestone before tuning balance.

See [`game-specs.md`](./game-specs.md), [`TODO.md`](./TODO.md), and [`AGENTS.md`](./AGENTS.md) for scope and implementation notes.
