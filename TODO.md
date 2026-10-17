# Too Late — Development TODO

## Milestone 1 — Migrate the current prototype to Bevy (in progress)

- [x] Add a Bevy 0.19 Rust application and Cargo lockfile.
- [x] Get Rust formatting, lint, and type checks clean in CI.
- [ ] Port the safe room, five authored rooms, room transitions, and fresh-run reset.
- [ ] Match player controls and combat: directional aim, combo, vulnerable heavy attack, dash, health, hit feedback, and sound.
- [ ] Match all six enemy patterns, both elite behaviors, the optional Crossing cache, and all ten run-only bonuses.
- [ ] Match the bonus catalog and pause menu, including Continue, Restart Run, Bonus, and Back Home.
- [ ] Preserve clear one-path-per-action controls and test mouse/gamepad input where supported.
- [ ] Add the Bevy browser/Wasm build path.
- [ ] Remove Phaser, TypeScript, Vite, Bun, and their legacy-only code after Bevy parity is complete.
- [ ] Invite the user to play the migrated game before tuning combat balance.

## Roguelike progression rule

- [x] Remove money, permanent upgrades, and any other progression that carries between runs from the specification.
- [ ] Confirm the Bevy build resets health, stats, and all temporary bonuses after death, Back Home, and Restart Run.
- [ ] Keep no shop, currency display, persistent unlocks, or cross-run bonuses.

## Full game goal

- [ ] Build ten distinct solo bosses for rooms 10, 20, …, 100; make room 100 the final boss.
- [ ] Expand the five-room prototype toward a 100-room run across ten distinct chapter styles.
- [ ] Add authored room variety, story discoveries, and optional challenges without breaking the fixed boss-room positions.
- [ ] Replace placeholder assets and generated audio when production assets are ready.
- [ ] Balance, polish, optimize, and playtest with friends.

## Future combat tuning

- [ ] Make the ranged enemy's projectile aim less predictable; keep its current behavior until a dedicated tuning pass.

## Development workflow

- [x] Move GitHub Actions CI to Rust formatting, Clippy, and Cargo checks; remove legacy Bun checks.
