# Too Late

Browser combat prototype for the game described in [`game-specs.md`](./game-specs.md).

## Run locally

```sh
bun install
bun run dev
```

Open the local URL printed by Vite. Build a static browser version with `bun run build`; the output goes to `dist/`.

Run `bun run check` for formatting, lint, typecheck, and production-build checks. GitHub Actions runs the same checks for pushes and pull requests.

## Controls

| Action | Controller | Keyboard / mouse |
| --- | --- | --- |
| Move | Left stick | Arrow keys or ZQSD (AZERTY) |
| Aim | Right stick | Mouse |
| Primary combo | X or right bumper | J or left click |
| Heavy attack | Y or left trigger | K or right click |
| Dash | B | Space |
| Room bonus | D-pad + A | Left/right + Enter, 1/2/3, or click |
| Restart after death / clear | — | R or click the prompt |

Heavy attacks hit hard, but slow movement through the windup and recovery. Dash briefly avoids damage. The melee enemy marks its strike area before attacking; the ranged enemy aims at the player's earlier position before firing; the heavy warden is slower and announces a wide strike. Damage floats briefly above enemies, while damage to the player appears in bold red.

## Current milestone

Runs currently use five handcrafted rooms that grow harder. After the first four, choose one of three temporary bonuses from a pool of ten. Some improve stats or health; others widen heavy attacks, add a burning combo hit or burst, or restore health when attacking. The HUD tracks your picks until a new run starts. Health carries between rooms. The ranged enemy's predictable aim is recorded for a later tuning pass. Permanent progression, story, and the boss remain later work. See [`TODO.md`](./TODO.md) and [`AGENTS.md`](./AGENTS.md) for the current project state.
