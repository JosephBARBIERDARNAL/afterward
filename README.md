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
| Pause menu | — | Esc |
| Start the next run after death / clear | — | R or click the button |

Heavy attacks hit hard, but slow movement through the windup and recovery. Dash briefly avoids damage. The melee enemy marks its strike area before attacking; the ranged enemy aims at the player's earlier position before firing; the heavy warden is slower and announces a wide strike. Damage floats briefly above enemies, while damage to the player appears in bold red.

## Current milestone

The current prototype uses five handcrafted rooms that grow harder. After clearing a room, choose one of three temporary bonuses from a pool of ten. Picked bonuses appear by name on separate HUD lines; press Esc and choose Bonus to see the full catalog and effects. The pause menu also has Continue and Restart. A new run clears the picked bonuses, and health carries between rooms.

Each cleared room earns one Ash. Ashes and purchased upgrades save in this browser, including after death or a page reload. At the end of a run, spend Ashes on permanent maximum health, attack damage, or dash cooldown upgrades; purchases take effect next run.

The end goal is a 100-room run in ten themed chapters, with a visually distinct solo boss in every tenth room. Jungle, Hong Kong, and the moon are example chapter styles; not all 100 rooms or bosses are implemented in this prototype. The ranged enemy's predictable aim is recorded for a later tuning pass. See [`game-specs.md`](./game-specs.md), [`TODO.md`](./TODO.md), and [`AGENTS.md`](./AGENTS.md) for project scope and status.
