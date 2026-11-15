# Bonus assets wanted

Every bonus now shows itself in the game (`src/game/tells/`), but several of
those tells are stand-ins I drew in code or Typst: a tinted glow, a plain
ring, fading copies of a sprite. This is the art that would make them read
better. Send examples (a pack page, a GIF, a sprite sheet) for any line and
I'll cut it in and iterate.

What the game expects of an effect, so an example can be judged quickly:

- **Pixel art**, crisp, close to the hero's scale: the hero is about 64 px
  tall per H; effects so far are 16–64 px frames.
- **A horizontal strip of frames** (or a sheet I can cut), 6–12 frames,
  played once or looped as noted.
- **White or greyscale** where noted "tinted": the game paints it in the
  bonus category's colour (combat red, defense blue, agility green,
  intelligence yellow, combined orange), so one sheet serves several bonuses.
- Transparent background, no baked shadow.

## Most wanted

These replace the weakest placeholders.

| # | What | Used by | Should look like | Size, frames | Play |
|---|------|---------|------------------|--------------|------|
| 1 | **Empowered aura** | Riposte, Retaliation | Energy or flame licking up around the body, clearly "your next hit is loaded". Now: a soft glow. | ~64×64, 6–8 | loop, tinted |
| 2 | **Mark reticle, animated** | Marked Prey, Open Wound | A target closing in and locking onto the enemy as the mark lands, then a slow spin while it holds. Now: a still 13 px reticle. | 16–24 px, 4–6 lock + 4 hold | once, then loop, tinted |
| 3 | **Shield bubble pop** | Room Ward, Deflector | A bubble or hex shield flashing around the player and shattering outward. Now: a plain ring. | ~64×64, 6–8 | once, tinted |
| 4 | **Slash arcs** | Long Reach, Quick Hands, the chain | A swoosh crescent per blow, its length matching the hitbox, so a longer reach is *seen*. Now: a thin white streak. | ~64×32, 3–5 | once, tinted |
| 5 | **Life orbs** | Lifesteal, Bloodlust, Hunter's Feast | Small red/green motes drifting from the struck enemy to the player. Now: only a green "+N". | 8–12 px orb, 4 | loop (moved by code) |
| 6 | **Execute icon** | Executioner, Open Wound | A small skull or blade over an enemy below the execute line, pulsing. Now: a white line on the health bar and a flashing fill. | 9–13 px, 2–4 | loop |

## Nice to have

| # | What | Used by | Should look like | Size, frames | Play |
|---|------|---------|------------------|--------------|------|
| 7 | **Meteor trail and crater** | Meteor | A fiery streak behind the falling hero, then a ground shockwave with debris. Now: orange copies of the sprite and the hit burst. | trail ~32×64, impact ~96×32, 6–8 | loop / once |
| 8 | **Pull vortex** | Undertow, Riptide | A ring of water or wind *contracting* toward the player (the opposite of a blast). Now: an expanding ring showing the reach. | ~128×48 flat, 6–8 | once, tinted |
| 9 | **Air burst under the feet** | Double Jump, Kick Off, Updraft | A puff or shock ring kicked downward on a mid-air jump. Now: a flat ring I drew (`art/effects/air_ring.typ`). | ~48×16, 5–6 | once, tinted |
| 10 | **Charge-ready flash** | Overcharge (the charged blow) | A star glint or flare at the fist the moment the heavy blow is ready. Now: a ring. | ~32×32, 4–6 | once |
| 11 | **Adrenaline heartbeat** | Adrenaline | A red throb around the body (or a screen-edge vignette) beating while under 30% health. Now: a red glow. | ~64×64, 6 | loop |
| 12 | **Chain Reaction crackle** | Chain Reaction, Bloodrush | Small sparks or arcs crawling over the body while the kill buff burns. Now: a red glow. | ~64×64, 6–8 | loop, tinted |
| 13 | **Speed lines / wind streaks** | Fleet Feet, Tailwind | Thin horizontal streaks trailing the runner. Now: green copies of the sprite. | ~32×16, 4 | loop, tinted |
| 14 | **Shield pierce** | Shield Breaker, Breaker | The enemy shield cracking with a blow punching through it. The shield's own break plays; this would be the "through" part. | ~32×32, 5–6 | once |
| 15 | **Alert "!" and unaware icons, animated** | Cold Read, Insight, Opportunist | A bouncing "!" over a wind-up, and a sleepy "z" or closed eye over an unaware enemy. Now: still icons I drew (`art/ui/alert.typ`, `art/ui/unaware.typ`). | 8–12 px, 2–4 | loop |
| 16 | **Loot sparkle** | Scavenger | A glint over a health drop that only Scavenger's odds let fall. Now: a callout only. | ~16×16, 4–6 | once |
| 17 | **Invulnerability shimmer** | Slippery (and every roll) | A ghostly flicker over the hero while nothing can reach them mid-roll. Now: pale green copies of the sprite. | full body, 4 | loop, tinted |

## Already fine

No new art needed: Decoy (its summon clip), Aftershock (the kill ring),
Recovery (the healing plusses), Last Stand (the fall-and-rise clips), the
finishers Power Strike, Fire Kick and Explosive Strike (their own strike
clips), and the category auras when a bonus is taken.
