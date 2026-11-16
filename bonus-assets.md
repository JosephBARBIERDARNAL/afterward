# Bonus assets wanted

Every bonus now shows itself in the game (`src/game/tells/`), but several of
those tells are stand-ins I drew in code or Typst: a tinted glow, a plain
ring, fading copies of a sprite. The VFX packs under `src/assets/packs/`
replaced most of them (see "In the game now"); what they do not cover is
under "Still wanted". Send examples (a pack page, a GIF, a sprite sheet) for any
line still wanted and I'll cut it in and iterate.

What the game expects of an effect, so an example can be judged quickly:

- **Pixel art**, crisp, close to the hero's scale: the hero is about 64 px
  tall per H; effects so far are 16–64 px frames.
- **A horizontal strip of frames** (or a sheet I can cut), 6–12 frames,
  played once or looped as noted.
- **White or greyscale** where noted "tinted": the game paints it in one of
  the tells' colours (`speed_color`, `power_color`, `insight_color` of
  `effects.tells`) or a rarity's, so one sheet serves several bonuses.
- Transparent background, no baked shadow.

## Still wanted

Nothing in the packs fits these. Each has a stand-in in the game; real art
would read better.

| #   | What                        | Used by                   | Should look like                                                                                                                                                                                                      | Size, frames                | Play                    |
| --- | --------------------------- | ------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------- | ----------------------- |
| 2   | **Mark reticle, animated**  | Marked Prey, Open Wound   | A crosshair closing in and locking onto the enemy as the mark lands, then a slow spin while it holds. The packs only have 64–96 px lock-on rings (Holy pack `00.png` row 2, PVFX legacy `focus-charge`): far too big. | 16–24 px, 4–6 lock + 4 hold | once, then loop, tinted |
| 11  | **Adrenaline heartbeat**    | Adrenaline                | A red throb around the body, or a screen-edge vignette, beating while under 30% health. Nothing throbs in any pack: a vignette shader may be the better answer.                                                       | ~64×64, 6                   | loop                    |
| 13  | **Speed lines**             | Fleet Feet, Tailwind      | Thin horizontal streaks trailing the runner. The packs only have one-shot dash dust and ornate wind gusts.                                                                                                            | ~32×16, 4                   | loop, tinted            |
| 17  | **Invulnerability shimmer** | Slippery (and every dash) | A ghostly flicker over the whole body while nothing can reach it mid-dash. Nothing body-shaped exists; the afterimages may stay.                                                                                      | full body, 4                | loop, tinted            |
| 8b  | **A pull that contracts**   | Undertow, Riptide         | A ring closing _in_ on the player. The packs have a flat whirlpool (fine as a stand-in) but nothing that shrinks toward the middle.                                                                                   | ~128×48 flat, 6–8           | once, tinted            |

## In the game now

Cut by `just vfx` (`scripts/cut_vfx.py`) from the packs kept under
`src/assets/packs/`: AW is `aw_vfx/` (the AW VFX Mega Pack, the base), PVFX is
`pvfx_foundry/` (PVFX Foundry, `strict` tier, for what AW lacks).

| #   | Need                     | Effect used                                                | Shown                                                    |
| --- | ------------------------ | ---------------------------------------------------------- | -------------------------------------------------------- |
| 1   | Empowered aura           | AW `Arcane_Charge`                                         | looping over the player until the empowered blow lands   |
| 3   | Shield bubble pop        | AW `Shield_Break` (Room Ward), AW `Shield_Hit` (Deflector) | once over the player                                     |
| 4   | Slash arcs               | AW `Slash_Horizontal`, frames 0–7                          | Long Reach: once, stretched over the blow's whole reach  |
| 5   | Life orbs                | AW `XP_Orb`                                                | flying from the enemy to the player as health comes back |
| 6   | Execute icon             | AW `Icon_Skull`                                            | beside the health bar of a body below the line           |
| 7   | Meteor crater            | AW `Explosion_Debris`                                      | once where the slam lands                                |
| 8   | Pull vortex              | PVFX `water/whirlpool`                                     | once at the feet, Undertow and Riptide                   |
| 9   | Air burst under the feet | PVFX `impacts/shockwave-ring`, frames 0–9                  | mid-air jumps and dashes, Updraft                        |
| 10  | Charge-ready flash       | AW `Hit_Critical`                                          | Overcharge, as a heavy blow is thrown                    |
| 12  | Chain Reaction crackle   | AW `Shock_Sparks`                                          | looping over the player while it burns                   |
| 14  | Shield pierce            | AW `Hit_Directional`                                       | Shield Breaker, Breaker                                  |
| 15  | Alert "!" and unaware    | AW `Icon_Exclamation`, AW `Sleep_Zzz`                      | over the enemy, beside the wind-up bar / while unaware   |
| 16  | Loot sparkle             | AW `Coin_Sparkle`                                          | Scavenger's drop                                         |
| –   | High Jump dust           | AW `Dust_Jump`                                             | at a lifted takeoff                                      |
| –   | Field Dressing           | AW `Heal_Burst`                                            | over the player as the boosted pickup heals              |

Not used yet, worth a look later: AW `Fire_Fireball` turned to point down,
as Meteor's trail (the drop still leaves orange afterimages); AW
`Hit_Spark_Small`, `Hit_Heavy` and PVFX `impacts/hit-spark` / `punch-hit` for
every blow, not only bonuses; AW `Dust_Land` / `Dust_Dash` for landings and
dashes.

## Already fine

No new art needed: Decoy (its summon clip), Aftershock (the kill ring),
Recovery (the healing plusses), Last Stand (the fall-and-rise clips), the
finishers Power Strike, Fire Kick and Explosive Strike (their own strike
clips), and the aura worn when a bonus is taken.
