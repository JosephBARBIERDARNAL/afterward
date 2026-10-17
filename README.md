# Too Late

**Too Late** is a single-player action roguelike in active migration from Phaser/TypeScript to Bevy/Rust. The game is a five-room prototype today; the end goal remains a 100-room run across ten visual chapters, with a solo boss every tenth room.

## Bevy development

Install a current stable Rust toolchain, then run:

```sh
cargo run
```

The Rust port uses Bevy 0.19 and built-in 2D shape primitives as replaceable placeholders. The [Bevy learning resources](https://bevy.org/assets/#learning), [2D assets index](https://bevy.org/assets/#2d), and [2D Shapes example](https://github.com/bevyengine/bevy/blob/latest/examples/2d/2d_shapes.rs) are documented in [`game-specs.md`](./game-specs.md). The 2D assets index lists community plugins rather than art packs.

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

## Current migration status

The Bevy implementation is a work in progress. It has a basic safe room, a five-room run, six enemy placeholders, core movement and attacks, room-clear doors, temporary bonuses, and keyboard pause/selection menus. Enemy behavior, authored room details, elite patterns, the Crossing cache, sound, click-friendly menu controls, and browser/Wasm packaging still need parity work. The legacy Phaser app remains available as a reference until that work is done.

The ranged enemy's normal projectile aim is intentionally still too predictable; revisit it during a dedicated combat-tuning pass. Ask the user to play the migrated build at the next playable milestone before tuning balance.

See [`game-specs.md`](./game-specs.md), [`TODO.md`](./TODO.md), and [`AGENTS.md`](./AGENTS.md) for scope and implementation notes.
