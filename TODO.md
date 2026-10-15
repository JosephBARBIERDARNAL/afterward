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
- [x] Require the player to open the east door after clearing a room; opening it presents the bonus choice.
- [x] Add max health, base damage, healing, dash nova, faster dash recharge, and increased attack range bonuses.
- [x] Show selected bonuses in the HUD and reset them when a new run starts.
- [x] Show bold floating damage numbers for hits dealt and bold red numbers for damage received.
- [x] Strengthen the enemy-hit sound.
- [x] User approved the five-room prototype and the direction of different run bonuses.

## Milestone 4 — Builds (implemented; playtest pending)

- [x] Expand the bonus pool to ten choices with both stat bonuses and combat-changing upgrades.
- [x] Add a wider heavy swing, a burning final combo hit, a final-hit burst, and life-stealing attacks.
- [x] Give bonuses unique names, list picked bonuses on separate HUD lines, and add a full effect catalog.
- [x] Add an Escape pause menu with Continue, Restart Run, Bonus, and Back Home actions.
- [ ] Ask the user to playtest the new upgrades and how different builds feel; tune from feedback.

## Milestone 5 — Permanent Progression (implemented; playtest pending)

- [x] Award $1 for each room cleared.
- [x] Save money and permanent upgrade levels in browser storage.
- [x] Add a safe room at game start and between runs, with a clickable shop fixture and a door that starts the next run.
- [ ] Ask the user to playtest resource pacing, upgrade effects, and save persistence.

## Milestone 6 — Full Enemy Roster (in progress)

- [x] Expand the roster from three to six enemy types: melee, ranged, heavy, charger, skirmisher, and bomber.
- [x] Add two marked elite behaviors: the faster, tougher Vanguard charger and the three-shot Volley ranged enemy.
- [ ] Playtest the new enemy patterns and tune their health, damage, and timings from feedback.
- [x] Add an optional bonus cache to The Crossing. It offers a separate three-choice run bonus and no extra money.
- [ ] Add optional dangerous paths with exploration rewards; keep room-clear money flat at $1 and bonus options based on different effects.

## Later milestones

- [ ] Milestone 7: build ten distinct solo bosses for rooms 10, 20, …, 100; make room 100 the final boss.
- [ ] Milestone 8: expand from the five-room prototype toward the 100-room run across ten distinct chapter styles, add story discoveries, and selectively replace placeholder assets.
- [ ] Milestone 9: balance, polish effects/audio/UI, optimize, and playtest with friends.

## Future combat tuning

- [ ] Make the ranged enemy's projectile aim less predictable; keep its current behavior until a dedicated tuning pass.

## Development workflow

- [x] Add Bun-based GitHub Actions CI for formatting, linting, typechecking, and production builds.
