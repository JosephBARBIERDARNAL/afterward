# Too Late

Browser combat prototype for the game described in [`game-specs.md`](./game-specs.md).

## Run locally

```sh
bun install
bun run dev
```

Open the local URL printed by Vite. Build a static browser version with `bun run build`; the output goes to `dist/`.

## Controls

| Action | Controller | Keyboard / mouse |
| --- | --- | --- |
| Move | Left stick | Arrow keys or ZQSD (AZERTY) |
| Aim | Right stick | Mouse |
| Primary combo | X or right bumper | J or left click |
| Heavy attack | Y or left trigger | K or right click |
| Dash | B | Space |
| Restart after death / clear | — | R or click the prompt |

Heavy attacks hit hard, but slow movement through the windup and recovery. Dash briefly avoids damage. The melee enemy marks its strike area before attacking; the ranged enemy aims at the player's earlier position before firing.

## Current milestone

This is Milestone 1 only: one arena, one melee pursuer, one ranged attacker, temporary combat actions, health, death, and restart. Roguelite rooms, run generation, upgrades, permanent progression, story, and the boss are intentionally later work. See [`TODO.md`](./TODO.md) and [`AGENTS.md`](./AGENTS.md) for the current project state.
