# Art direction

The base every asset in Afterward is made against: surfaces, backdrops, characters,
UI, shaders, effects. When a new asset is drawn or an old one reworked, it answers
to this file. Tuning values stay in the YAML under `src/assets/`; this file says what
they should mean.

## The world

The city did not end in a war. The power stayed on and the people did not. What
came afterward was the water: the floods rose, took the lower city, and took the
colour of everything the city leaked into them — coolant, fuel, fuel light. It
sits in the streets now, deep and wine-dark, lit from under by whatever is still
energised in it, and it is rising.

The run is a climb out. Ten chapters, each further from the flood than the last:
the drowned streets, then the levels above them, then the city's edge, then
whatever is past it. The water is in every chapter — behind the level, under the
grates, filling the drop — because it is the thing being escaped. It is the one
motif that holds ten environments together, and it is never touched: the player
never swims, never wades, never dies in it. It is always below, always behind,
always visible.

What walks the rooms is what the city never switched off. Security units,
maintenance frames, enforcement chassis, standing idle in the dark until
something crosses their line of sight. They are not evil and they are not alive.
They are machinery completing an instruction nobody cancelled.

Mood words, for when a choice is close: **cold, still, wet, abandoned, patient,
beautiful.** Not: gritty, chaotic, crowded, neon-saturated, playful.

## Three rules

Everything below follows from these. If a new asset breaks one, the asset is
wrong, not the rule.

**1. The world is cold. Only danger and help are warm.**
Every structure, backdrop and machine in the frame sits in violet, blue-violet
and cold grey. Orange, red and warm white are _reserved_: an attack wind-up, a
projectile, a muzzle flash, a damage number, a pickup. A player who sees warm
knows something is about to happen to them. Never spend warm on decoration — not
on a rusty pipe, not on a lamp, not on a sunset.

**2. The world is dark. Nothing in it outshines the player or the action.**
No world pixel goes brighter than **`#6A6288`**. The exceptions are declared and
few: a light source (a strip, a sign, a screen), the flood's crests, the player,
and anything warm under rule 1. This is what makes a dark game readable — not
lighting the world, but refusing to.

**3. The flood is in every frame, or its light is.**
If the water itself is not on screen, its colour is: a stain on a wall at the
old waterline, a reflection on wet ground, light thrown up from a stairwell
going down. A room that could belong to any game has not been finished.

## Palette

The master palette. Every family palette below is mixed from it; nothing
introduces a hue that is not here.

### World — cold, desaturated

| Role        | Hex       | Use                                            |
| ----------- | --------- | ---------------------------------------------- |
| `void`      | `#0B0A14` | deepest gaps, sky, the back of everything      |
| `structure` | `#1C1A2E` | platform bodies, the default solid             |
| `mid`       | `#2E2A47` | lit faces, the readable middle                 |
| `light`     | `#4A4270` | top edges, rim light, the edge of a silhouette |
| `ceiling`   | `#6A6288` | the brightest a world pixel is allowed to be   |

### The flood — hot hue, dark value

| Role    | Hex       | Use                                                   |
| ------- | --------- | ----------------------------------------------------- |
| `depth` | `#0F0514` | the heart of the water, where it is deepest           |
| `body`  | `#2A0E2B` | deep wine, the surface read at a glance               |
| `crest` | `#4A1F4A` | the lighter water toward a wave's edge                |
| `foam`  | `#7FE6E8` | pale cyan light on the surface — the only bright part |

Hot in hue, dark in value. That is how the water can be red-purple without
breaking rule 1: it never competes with the action because it is never bright.
The light _on_ it is cold.

### City light — cold and failing

| Role    | Hex       | Use                                      |
| ------- | --------- | ---------------------------------------- |
| `strip` | `#6FD3E8` | emergency lighting, conduits, live seams |
| `sign`  | `#C8389B` | dying signage, indicator lamps, screens  |
| `dead`  | `#9FB4C4` | fluorescent with no colour left in it    |

### Action — hot, reserved

| Role        | Hex       | Use                                          |
| ----------- | --------- | -------------------------------------------- |
| `telegraph` | `#FF7A2F` | attack wind-up, the frame before it lands    |
| `danger`    | `#FF3B3B` | active hitboxes, projectiles, damage numbers |
| `muzzle`    | `#FFF4E6` | the player's shots, impacts, sparks          |

## The flood

**Where it is.** Below the playable floor, so every pit bottoms out in it. Behind
the level, as the horizon of a flooded street. Under grates and through broken
floor, as a lit seam. In tanks, pipes and stairwells going down. Never in the
playable space — it is scenery in every chapter, and no room asks the player to
avoid it.

**How it is drawn.** One shader, shared with the main menu. `menu_water.wgsl`
already does exactly the right thing — a cellular field with lit borders,
quantized to a pixel grid and four tones — so it becomes the flood shader, and
its four tones take the flood colours above. The main menu then is not "water":
it is the flooded city, seen from the surface. The game opens on the thing it is
about.

**Rising.** The water is higher in each chapter than the last at the same
altitude, and the player climbs faster than it rises. Room 100 is above it. A
chapter can show this with a waterline stain on the structures: a band of
`#2A0E2B` with a sharp `#4A1F4A` edge, sitting higher in the frame each chapter.

## Light

Nobody is maintaining anything. Light is cold, comes from infrastructure, and is
unreliable. Emergency strips along floors and ceilings, signage with half its
elements dead, screens still looping something for an audience that left. The
`city_lights` shader already breathes brightness into any pixel that stands
above its neighbours on a dark surround — that is exactly the right behaviour:
lights are _found_ in the art, not authored, and they are never quite steady.

Light never explains the room. It marks edges, picks out a seam, and leaves the
middle dark. Where a room must be read, the structure reads by silhouette
against a slightly lighter backdrop band, not by being lit.

The one warm light in the game is the player's gun.

## Air

Three layers, all cheap, all in service of depth:

- **Rain.** Intermittent, not constant. It justifies the wet ground, ties the sky
  to the flood, and gives a still room motion. Cold, thin, near-`#4A4270`,
  never white streaks.
- **Haze.** Each backdrop band fades toward `void`, so distance reads as loss of
  contrast rather than loss of detail. This is what makes flat silhouette bands
  look like a city instead of cardboard.
- **Particles.** Slow drift — ash, spray off the water, dust shaken from a
  ceiling. A few per screen, below the value ceiling, moving at a pace that is
  noticed only when the player stands still.

## Surfaces

The Typst pipeline stays exactly as it is: seamless 16 px tiles, one tile per
0.25 H, palettes of at most five keys (`#` seam, `d` shadow, `m` body, `l` light,
`r` highlight), axis-aligned squares only. What changes is the colour in them.

### Which families belong to the city

`city` and `metal` are the flooded chapters. `carved` and `wood` are not cyberpunk
and must not leak into chapters 1–4 — they belong to whatever is past the city,
and their palettes stay unconverted until the chapter that needs them is
designed. Chapter 1 is `city` plus `metal`, nothing else.

### Recoloured family palettes

Drop-in replacements for the current ones, mixed from the master palette.

```
// art/surfaces/textures/city/_city.typ

concrete  — poured slabs, the street itself
  "#" #0E0D18   "d" #1F1C30   "m" #2E2A47   "l" #413A5E   "r" #5A5179

conduit   — dark panels lit by live seams   (was `neon`)
  "#" #08070F   "d" #16142A   "m" #221F3B   "l" #3FA8BF   "r" #9FE4EE

alloy     — hull plating with indicator bolts
  "#" #0D0C16   "d" #221F33   "m" #332F4C   "l" #4C4670   "r" #C8389B
```

```
// art/_palettes.typ

steel     — cold chassis plate
  "#" #0D0C16   "d" #2A2740   "m" #403B5C   "l" #5C5680   "r" #8A84A8

corrosion — what the flood did to metal   (was `rust`)
  "#" #150A12   "d" #3A1524   "m" #5C2436   "l" #81414B   "r" #A9646A
```

`rust` becomes `corrosion` and stops being orange. Rust is warm, and warm is
reserved; what eats metal in this world is the water, so the stain is wine, not
oxide. It is the one place the flood's hue appears on a solid surface, and that
is deliberate: corrosion marks how high the water has been.

### Wet, glass, liquid

Three distinct things, three distinct techniques:

**Wet ground** is the cheapest and the most important. It is already half
implemented: `edges.highlight` in `surfaces/materials.yaml` draws a bright line
along every uncovered edge. Make that line the sheen — cold `#6FD3E8` at low
alpha instead of white — and every platform top reads as wet without a single
new texture. Reflections under a light source are hand-placed background
decoration panels carrying a vertical-streak texture; they are art, not physics,
so they go where they look right.

**Translucent panels** are decoration blocks that let the room show through:
tinted glass, a dead screen, a partition. This needs a per-block `alpha` the data
does not have yet — a code change, listed below. Glass is `mid` at low alpha with
a single `light` edge, never a gradient.

**Flowing liquid** is the flood shader on a quad, scaled down: a tank, a pipe
run, a channel in the floor. Same four tones, so a vat of it and the ocean of it
are visibly the same substance.

## Backdrops

The cloud packs go. They are bright, blue and daytime, and no amount of recolour
makes a pastel sky belong to this. In their place: flat silhouette bands of the
palette, three or four deep, each one closer and darker, with the `city_lights`
shader finding windows in the far ones.

```
sky         #0B0A14  at the top, fading to #221B33 at the horizon
band 3 far  #1A1729  half-sunk towers, most windows lit
band 2      #15131F  nearer blocks, few windows
band 1 near #0E0D16  the strongest silhouette, almost no light
```

Nearest is darkest, so the playable layer always reads in front of it. Bands are
drawn in Typst like everything else, which finally makes backdrops and surfaces
one coherent set.

A chapter gets a backdrop, not a room. Chapter 1 currently spends eleven
different profiles across eleven rooms — four city skylines and seven cloud
packs — so no two rooms look like the same place. One chapter, one backdrop,
varying only in how far the camera is from the water.

## Characters

### The player

The brightest _cold_ thing on screen. Where the world stops at `#6A6288`, the
player's lit side goes up to about `#B9C4E8` — so the eye finds them instantly in
a dark room without a single warm pixel being spent. Cold-neutral clothing, a
limited palette, a silhouette that reads at 1 H tall. The only warmth they ever
emit is the gun: muzzle flash, ejected light, the warm flash of taking damage.

### The machines

Chassis in `steel`, grounded in the world's values, with one cold accent — a lens,
a strip, an eye. That accent is the game's clearest piece of information and it
should carry state:

- **idle**, not yet aware: `strip` `#6FD3E8`, dim and steady
- **alerted**, hunting: the accent brightens and holds
- **winding up** to attack: `telegraph` `#FF7A2F`

That is rule 1 doing gameplay work. Warm appearing on an enemy means an attack
is coming, with no other cue needed.

### Reinterpreting the current cast

The sprites stay. The animations stay. What changes is colour and name, so the
roster stops being a fantasy pack:

| was           | is             | read                                      |
| ------------- | -------------- | ----------------------------------------- |
| `swordsman`   | `blade_unit`   | melee chassis, bladed arm                 |
| `infantryman` | `rifle_unit`   | ranged chassis                            |
| `destroyer`   | `heavy_unit`   | armoured, slow, charges its shot          |
| `gorgon_1`    | `stalker_unit` | fast, lunges                              |
| `gorgon_2`    | `lancer_unit`  | ranged, keeps its distance                |
| `gorgon_3`    | `breaker_unit` | heavy sweep, slow                         |
| `dark_ninja`  | `scout_unit`   | fastest, lightest frame                   |
| `evil_wizard` | `emitter_unit` | the Chapter 1 boss; its shots are blasts  |

The `flesh` material is now `polymer`: nothing in this world bleeds, and the
distinction it actually encodes is armoured chassis versus light frame. Armor
is its own setting now (`armor_layers`), which the `metal` types carry one
layer of.

The sprite sheets keep their vendor names and paths
(`enemies/type1/Swordsman/` and the rest). They name files on disk, not the
cast, and renaming them would dress a placeholder up as something settled.

Recolouring buys coherence, not correctness. A recoloured swordsman still swings
a sword. The real cast is machines, and these sprites are a stopgap — a good
one, but this file says plainly that they are placeholder shapes wearing the
right colours.

## UI, pickups, glow

UI is cold, thin and quiet: `structure` panels, `light` borders, `dead` text.
Nothing in the interface is warm except the numbers that mean damage.

The five bonus categories are help, so warm is legitimate there — they are the
one place the palette opens up, and they should feel like the only colour left in
the world. Their current colours survive the direction; they want brightening and
cleaning so they pop as a family against the dark, not retinting.

Glows (`effects.glow`) mark what to walk toward. The door's `#7FD4FF` is right —
cold, calm, the way out. The health pickup's `#4CF07A` is the one colour in the
game that belongs to no part of this palette; it needs a decision (below).

## The chapter arc

Provisional. Only Chapter 1 is being built, and this exists so Chapter 1's
choices do not paint later ones into a corner. Each step is further above the
water than the last.

1. **Drowned streets** — the flood at its deepest, the city at its most intact
2. **Undercity** — drains and service levels, water on three sides
3. **Transit** — rail and tunnel, the first long climb
4. **Tower interiors** — glass, screens, the city's offices
5. **Rooftops** — the first chapter above the waterline, rain and sky
6. **Industrial edge** — where the city stops being designed
7. **Outskirts** — drowned ground outside the city, the water still there
   8–10. **Past the city** — unwritten. This is where `carved` and `wood` may earn
   their place, and where the palette is allowed to shift, as long as the water is
   still visible somewhere in the frame.

## Checking a frame

Three tests, in order:

1. **Squint.** Everything still visible should be either a light, the player, or
   something that can hurt or help. If a decoration survives the squint, it is
   too bright.
2. **Pick the warmest pixel.** It must belong to danger, help, or the gun. Any
   other answer is a bug in the art.
3. **Find the water.** The flood, its light, its stain, or its reflection. If it
   is nowhere, the frame is not from this game.

## What this means for the files today

Done:

- `src/assets/config/gameplay.yaml` — `effects.menu_water` is `effects.flood`
  and carries the flood tones; the shader, its material and its component
  followed (`shaders/flood.wgsl`, `src/game/shaders/flood.rs`)
- `src/assets/background/background_profiles.yaml` — the `drowned_city`
  profile, rendered from `art/background/`; every Chapter 1 room is on it
- `src/assets/surfaces/materials.yaml` — `edges.highlight_color` is the cold
  sheen, `edges.outline_color` and `decoration_layers.background.tint` are
  re-derived from the palette, and two materials are renamed
- `art/_palettes.typ`, `art/surfaces/textures/city/_city.typ` and
  `art/doors/exit_door.typ` — the recoloured palettes above
- `src/assets/rooms/*.yaml` — the bare `color: "#292929"` blocks are `structure`
- `src/assets/enemies/enemy_types.yaml` — the type renames and `polymer`

Still to do:

- per-block `alpha` in the surfaces data, for translucent panels
- the flood shader instanced on world quads, so a room can show water in a
  tank, a channel, or the drop below its floor
- rain and drifting particles
- the character palette pass, player first
- the waterline stain on structures, sitting higher each chapter
- the cloud and skyline packs under `src/assets/background/` are unused now;
  they are third-party assets, so they stay on disk pending a decision

## Open decisions

- **Health pickup glow.** `#4CF07A` is the only green in the game. Warm reads as
  danger under rule 1, cold reads as inert, and green belongs to neither. The
  cleanest answer is probably `muzzle` `#FFF4E6` — warm, meaning help, consistent
  with the gun — but it then shares a colour with the gun. Alternative: keep a
  green but pull it to a cold mint around `#6FE8A8`.
- **Enemy shield colour.** Currently gold. Gold is warm, and a shield is
  information, not a threat. Pale cyan-white that shatters into warm sparks would
  follow the rules — but it is existing, working art.
- **Rain.** Decided as yes, undecided as how: a shader pass over the frame, or
  particles in the world that the parallax layers can move at their own rate.
- **The value ceiling as a check.** `#6A6288` is a rule nothing currently
  enforces. It could be a test over the rendered PNGs in CI, the way `just art`
  already round-trips them.
