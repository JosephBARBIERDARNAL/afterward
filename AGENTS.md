# Too Late — Agent Notes

Updated: 2026-10-15

## Project status

The user reports that rooms and the run bonus direction feel good. The current five-room build is a prototype slice; the specified end goal is a 100-room run split into ten visual chapters, with solo bosses in rooms 10, 20, …, 100. The safe room appears at game start and between runs. Clearing a room awards saved money displayed with `$`; click the shop fixture to reveal permanent upgrade options. After clearing a non-final room, approach and interact with the east door to open it and see the bonus choices. The prototype offers three choices from a ten-bonus pool, lists picks by name on separate HUD lines, and has a pause menu with a full bonus catalog, Restart Run, and Back Home. Temporary bonuses reset with a new run. Damage feedback and a stronger enemy-hit sound are in place. The prototype now has six enemy types and two elite behaviors; playtest these before tuning.

The ranged enemy's normal projectile aim feels too predictable; keep its current behavior for now. Ask the user to playtest the combat-changing upgrades and new enemy patterns at the next playable check-in. Browser interaction could not be checked by the agent because browser-control tools were not available in this session.

## Current stack

- Phaser 4.2.1, TypeScript, and Vite.
- Phaser is the browser-first 2D framework. Phaser Editor provides visual scene editing, animation tools, and physics inspectors for Phaser projects.
- Desktop keyboard/mouse and standard browser Gamepad API inputs are supported. Keyboard movement uses arrows or ZQSD for AZERTY. The design remains controller-first.
- The MVP uses generated placeholder shapes and Web Audio tones. Keep rendering and sound code replaceable without coupling it to combat rules.
- Local-only project. No hosted Site or deployment has been requested.

## Project layout

- `src/game/CombatScene.ts` — arena simulation, combat state, HUD, and input wiring.
- `src/game/HubScene.ts` — no-enemy safe room, permanent upgrade shop, and start-run door.
- `src/game/data/arena.ts` — authored room layouts, geometry, and spawn points.
- `src/game/data/balance.ts` — MVP combat tuning values.
- `src/game/progression.ts` — permanent upgrade definitions and local save data.
- `src/game/presentation/` — placeholder rendering and generated sound effects.
- `game-specs.md` — authoritative product and scope specification.
- `TODO.md` — milestone tracking and open work.

## Run and build

- `bun install` — install dependencies.
- `bun run dev` — start the local browser game.
- `bun run build` — create a static browser build in `dist/`.
- `bun run check` — run formatting, lint, typecheck, and production-build checks.

## Scope and implementation rules

- Follow the milestone order in `game-specs.md`; the current prototype is a five-room slice with temporary bonuses, a catalog/pause UI, persistent money, and a safe-room upgrade shop. Keep the 100-room, ten-chapter run and ten solo bosses explicit as the end goal without treating them as already implemented.
- Core combat must keep player movement, explicit directional aiming, a primary melee combo, a vulnerable heavy attack, a defensive dash, health/damage, six current enemy types with two elite behaviors, placeholder art/audio, and death/restart.
- No block or parry, target auto-selection, mobile-specific controls, progression, upgrades, or generated rooms in Milestone 1.
- Prefer data-driven tuning and authored arena geometry. Keep visual/audio assets behind presentation modules so replacements do not change gameplay rules.
- Always remove dead code as part of each change.
- Ask the user when a design decision materially changes the specified game. At each useful playable milestone, invite the user to play and report what feels wrong.
- UX preference from playtesting: keep one clear input path per action within a control scheme; do not add redundant shortcuts or click alternatives. Preserve the requested arrow and ZQSD keyboard movement bindings.

## Verification notes

Do not claim the bonuses or combat balance are tuned based on implementation alone; user play feedback is the gate to the next milestone.
