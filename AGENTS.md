# Too Late — Agent Notes

Updated: 2026-10-15

## Project status

The user has played the three-room sequence and reports that rooms work well. Milestone 3 is in progress: the prototype now uses five progressively harder rooms and offers three temporary bonuses after each non-final room. Bonuses are shown in the HUD and reset with a new run. Damage feedback and a stronger enemy-hit sound are in place.

The ranged enemy's projectile aim feels too predictable; record it as future combat tuning and leave its current behavior in place for now. Ask the user to playtest the five-room run and bonus choices at the next playable check-in. Browser interaction could not be checked by the agent because browser-control tools were not available in this session.

## Current stack

- Phaser 4.2.1, TypeScript, and Vite.
- Phaser is the browser-first 2D framework. Phaser Editor provides visual scene editing, animation tools, and physics inspectors for Phaser projects.
- Desktop keyboard/mouse and standard browser Gamepad API inputs are supported. Keyboard movement uses arrows or ZQSD for AZERTY. The design remains controller-first.
- The MVP uses generated placeholder shapes and Web Audio tones. Keep rendering and sound code replaceable without coupling it to combat rules.
- Local-only project. No hosted Site or deployment has been requested.

## Project layout

- `src/game/CombatScene.ts` — arena simulation, combat state, HUD, and input wiring.
- `src/game/data/arena.ts` — authored room layouts, geometry, and spawn points.
- `src/game/data/balance.ts` — MVP combat tuning values.
- `src/game/presentation/` — placeholder rendering and generated sound effects.
- `game-specs.md` — authoritative product and scope specification.
- `TODO.md` — milestone tracking and open work.

## Run and build

- `bun install` — install dependencies.
- `bun run dev` — start the local browser game.
- `bun run build` — create a static browser build in `dist/`.
- `bun run check` — run formatting, lint, typecheck, and production-build checks.

## Scope and implementation rules

- Follow the milestone order in `game-specs.md`; keep the current build focused on the five-room run, temporary bonuses, and clear combat feedback.
- Core combat must keep player movement, explicit directional aiming, a primary melee combo, a vulnerable heavy attack, a defensive dash, health/damage, three enemy types, placeholder art/audio, and death/restart.
- No block or parry, target auto-selection, mobile-specific controls, progression, upgrades, or generated rooms in Milestone 1.
- Prefer data-driven tuning and authored arena geometry. Keep visual/audio assets behind presentation modules so replacements do not change gameplay rules.
- Ask the user when a design decision materially changes the specified game. At each useful playable milestone, invite the user to play and report what feels wrong.

## Verification notes

Do not claim the bonuses or combat balance are tuned based on implementation alone; user play feedback is the gate to the next milestone.
