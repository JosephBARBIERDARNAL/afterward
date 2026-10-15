# Too Late — Agent Notes

Updated: 2026-10-15

## Project status

The repository began with only `game-specs.md`. The current implementation target is **Milestone 1: Combat**, as required by the spec. Do not add rooms, procedural generation, upgrades, permanent progression, story, or a boss until the arena combat has been tried and judged fun enough to continue.

The first arena is implemented. The user has tried it and reported a restart issue, requested AZERTY controls and a visible dash recharge bar, and flagged two decorative crosses and clipped footer copy. Those items are being addressed before the next user playtest. Browser interaction could not be checked by the agent because browser-control tools were not available in this session.

## Current stack

- Phaser 4.2.1, TypeScript, and Vite.
- Phaser is the browser-first 2D framework. Phaser Editor provides visual scene editing, animation tools, and physics inspectors for Phaser projects.
- Desktop keyboard/mouse and standard browser Gamepad API inputs are supported. Keyboard movement uses arrows or ZQSD for AZERTY. The design remains controller-first.
- The MVP uses generated placeholder shapes and Web Audio tones. Keep rendering and sound code replaceable without coupling it to combat rules.
- Local-only project. No hosted Site or deployment has been requested.

## Project layout

- `src/game/CombatScene.ts` — arena simulation, combat state, HUD, and input wiring.
- `src/game/data/arena.ts` — authored arena geometry and spawn points.
- `src/game/data/balance.ts` — MVP combat tuning values.
- `src/game/presentation/` — placeholder rendering and generated sound effects.
- `game-specs.md` — authoritative product and scope specification.
- `TODO.md` — milestone tracking and open work.

## Run and build

- `bun install` — install dependencies.
- `bun run dev` — start the local browser game.
- `bun run build` — create a static browser build in `dist/`.

## Scope and implementation rules

- Follow the milestone order in `game-specs.md`; keep the current build focused on the combat question.
- The first arena must have player movement, explicit directional aiming, a primary melee combo, a vulnerable heavy attack, a defensive dash, health/damage, one melee pursuer, one ranged enemy, placeholder art/audio, and death/restart.
- No block or parry, target auto-selection, mobile-specific controls, progression, upgrades, or generated rooms in Milestone 1.
- Prefer data-driven tuning and authored arena geometry. Keep visual/audio assets behind presentation modules so replacements do not change gameplay rules.
- Ask the user when a design decision materially changes the specified game. At each useful playable milestone, invite the user to play and report what feels wrong.

## Verification notes

`bun run build` passes after the feedback changes, and `curl -I http://127.0.0.1:5173/` returns HTTP 200. No browser interaction or gameplay was performed by the agent in this session. Do not claim the combat is fun based on implementation alone; user play feedback is the gate to Milestone 2.
