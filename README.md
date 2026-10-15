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
| Primary combo | X | J |
| Heavy attack | Y | K |
| Dash | B | Space |
| Open cleared exit | A near the exit | E near the exit |
| Choose room bonus | D-pad + A | Left/right + Enter |
| Pause menu | — | Esc |
| Open shop / buy upgrade | — | Click shop, then click a purchase |
| Start a run | A near door | E near door |

Heavy attacks hit hard, but slow movement through the windup and recovery. Dash briefly avoids damage. The melee enemy marks its strike area before attacking; the ranged enemy aims at the player's earlier position before firing; the heavy warden is slower and announces a wide strike. Damage floats briefly above enemies, while damage to the player appears in bold red.

## Current milestone

The current prototype uses five handcrafted rooms that grow harder and six enemy types: melee, ranged, heavy, charger, skirmisher, and bomber. The final room introduces two marked elite behaviors: a faster, tougher Vanguard charger and a Volley ranged enemy that fires three projectiles. A safe room appears when the game starts and after each run. Move with arrows or ZQSD, click the shop fixture to reveal permanent upgrade options, then approach the door and press E to begin a run. After clearing a non-final room, approach the east door and press E to open it; this presents one of three temporary bonuses from a pool of ten. Use left/right and Enter to choose. The east exit can be crossed after the choice. Picked bonuses appear by name on separate HUD lines; press Esc and choose Bonus to see the full catalog and effects. The pause menu has Continue, Restart Run, Bonus, and Back Home. A new run clears the picked bonuses, and health carries between rooms.

Each cleared room earns $1. Money and purchased upgrades save in this browser, including after death or a page reload. Spend money in the safe room on permanent maximum health, attack damage, or dash cooldown upgrades; purchases take effect next run.

The end goal is a 100-room run in ten themed chapters, with a visually distinct solo boss in every tenth room. Jungle, Hong Kong, and the moon are example chapter styles; not all 100 rooms or bosses are implemented in this prototype. The ranged enemy's predictable aim is recorded for a later tuning pass. See [`game-specs.md`](./game-specs.md), [`TODO.md`](./TODO.md), and [`AGENTS.md`](./AGENTS.md) for project scope and status.
