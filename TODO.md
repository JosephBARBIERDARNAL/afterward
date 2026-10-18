# Afterward — Development TODO

## Milestone 1 — Migrate the current prototype to Bevy (implementation complete; playtesting pending)

- [x] Add a Bevy 0.19 Rust application and Cargo lockfile.
- [x] Split the Bevy entry point into app setup, game state, arena, combat, progression, UI, and presentation modules.
- [x] Separate formatting/lint, tests, and native executable compilation in CI; build Linux, macOS Apple Silicon/Intel, and Windows targets.
- [x] Port the safe room, five authored room layouts and obstacles, room transitions, and fresh-run reset.
- [x] Preserve keyboard movement, directional aim, combo, heavy attack, dash, health, and hit feedback.
- [x] Port generated sound cues for primary/heavy attacks, dash, bonus bursts, hit/death, enemy defeat, and room clear.
- [ ] Playtest generated sound levels and effects with the user.
- [x] Add animated damage numbers for hits dealt and hits received, including projectile damage.
- [x] Show the player's current/max health and each living enemy's remaining health during combat.
- [x] Keep the player's directional weapon visible, show action cues, strengthen attack telegraphs, and sharpen UI text.
- [x] Port all six enemy patterns and both elite behaviors, including visible attack warnings.
- [x] Port the optional Crossing cache as an additional run-only bonus choice.
- [ ] Playtest all ten run-only bonuses and confirm they work as described.
- [x] Match the bonus catalog and pause menu, including Continue, Restart Run, Bonus, and Back Home.
- [x] Preserve one clear keyboard path per action and mouse aiming; browser and controller support are outside migration scope.
- [ ] Playtest the AZERTY mapping, larger window, and faster movement with the user.
- [x] Remove Phaser, TypeScript, Vite, Bun, and their legacy-only code.

## Roguelike progression rule

- [x] Remove money, permanent upgrades, and any other progression that carries between runs from the specification.
- [x] Clear health upgrades and all temporary bonuses after death, Back Home, and Restart Run.
- [x] Keep no shop, currency display, persistent unlocks, or cross-run bonuses in Bevy.

## Full game goal

- [ ] Build ten distinct solo bosses for rooms 10, 20, …, 100; make room 100 the final boss.
- [ ] Expand the five-room prototype toward a 100-room run across ten distinct chapter styles.
- [ ] Add authored room variety, story discoveries, and optional challenges without breaking the fixed boss-room positions.
- [ ] Replace placeholder assets and generated audio when production assets are ready.
- [ ] Balance, polish, optimize, and playtest with friends.

## Future combat tuning

- [ ] Make the ranged enemy's projectile aim less predictable; keep its current behavior until a dedicated tuning pass.
