# Too Late — Development TODO

## Milestone 1 — Combat MVP (complete for now)

- [x] Create one readable arena with placeholder visuals.
- [x] Add movement and explicit directional aim for keyboard/mouse and controller.
- [x] Add a fast three-hit primary combo, high-damage vulnerable heavy, and defensive dash.
- [x] Add health/damage and three enemy types: pursuing melee, ranged, and heavy.
- [x] Add death, restart, and arena-clear states.
- [x] Add generated sound effects and combat feedback.
- [x] Apply initial usability feedback: reliable run reset, AZERTY keys, dash recharge bar, cleaner arena corners, and wrapped footer copy.

## Milestone 2 — Rooms (complete for now)

- [x] Build a fixed sequence of five handcrafted encounter layouts with increasing enemy count and stronger combinations.
- [x] Open the east gate after each cleared room and transition while preserving health.
- [x] User reports that the room transitions work well.

## Milestone 3 — Runs (complete for now)

- [x] Offer three temporary bonus choices after each non-final room.
- [x] Add max health, base damage, healing, dash nova, faster dash recharge, and increased attack range bonuses.
- [x] Show selected bonuses in the HUD and reset them when a new run starts.
- [x] Show bold floating damage numbers for hits dealt and bold red numbers for damage received.
- [x] Strengthen the enemy-hit sound.
- [x] User approved the five-room run and bonus-choice direction.

## Milestone 4 — Builds (in progress)

- [x] Expand the bonus pool to ten choices with both stat bonuses and combat-changing upgrades.
- [x] Add a wider heavy swing, a burning final combo hit, a final-hit burst, and life-stealing attacks.
- [ ] Ask the user to playtest the new upgrades and how different builds feel; tune from feedback.

## Later milestones

- [ ] Milestone 5: add permanent resources, saving, and a between-run upgrade shop.
- [ ] Milestone 6: add more enemy types, behavior-changing elites, optional risky paths, and exploration rewards.
- [ ] Milestone 7: create the final boss and a completable run.
- [ ] Milestone 8: add the surreal Egyptian setting, story discoveries, and selectively replace placeholder assets.
- [ ] Milestone 9: balance, polish effects/audio/UI, optimize, and playtest with friends.

## Future combat tuning

- [ ] Make the ranged enemy's projectile aim less predictable; keep its current behavior until a dedicated tuning pass.

## Development workflow

- [x] Add Bun-based GitHub Actions CI for formatting, linting, typechecking, and production builds.
