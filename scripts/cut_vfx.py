#!/usr/bin/env python3
"""Cuts the effects the bonuses show themselves with out of the VFX packs.

The packs are kept as they ship under `src/assets/packs/`:

- `aw_vfx/` is the AW VFX Mega Pack (artistworld, free for commercial use),
  the base: one row of frames per sheet, 15 fps.
- `pvfx_foundry/` is PVFX Foundry (CC0), for what AW lacks: its `strict`
  tier, whose grid sheets lay the frames eight to a row.

Each clip below is laid out as one row of frames in `effects/vfx/<name>.png`,
cut down to the frames the game plays; `assets/effect_sheets.rs` lists them
with their frame size and pace. Run it with `just vfx` after a clip is added
or recut. Needs ImageMagick.
"""

import subprocess
from pathlib import Path

ASSETS = Path(__file__).resolve().parent.parent / "src/assets"
PACKS = ASSETS / "packs"
OUT = ASSETS / "effects/vfx"

AW = "aw_vfx"
PVFX = "pvfx_foundry"


def aw(effect):
    """The Small sheet of an AW effect, by its category and name."""
    category, name = effect.split("/")
    return f"{category}/{name}/Small/{name}_Small_sheet.png"


def pvfx(effect):
    """The strict grid sheet of a PVFX effect, by its family and name."""
    return f"effects/{effect}/strict/grid/sprite-sheet.png"


# Name: (pack, sheet, frame width, frame height, columns of the sheet, the
# frames kept as a range of indexes).
CLIPS = {
    # The next blow empowered (Riposte, Retaliation): a rune ring round the
    # body, looping.
    "arcane_charge": (AW, aw("05_Charges/Arcane_Charge"), 48, 48, 18, range(18)),
    # Room Ward spent: the bubble cracks and shatters.
    "shield_break": (AW, aw("06_Heal_Shield_Buffs/Shield_Break"), 48, 48, 18, range(14)),
    # Deflector: the bubble ripples where the shot struck it.
    "shield_hit": (AW, aw("06_Heal_Shield_Buffs/Shield_Hit"), 44, 44, 18, range(12)),
    # Long Reach: the arc of a blow, as long as it reaches.
    "slash": (AW, aw("03_Slashes/Slash_Horizontal"), 64, 32, 18, range(8)),
    # Lifesteal, Bloodlust, Hunter's Feast: the health flying to the player.
    "life_orb": (AW, aw("09_Pickups_Sparkles/XP_Orb"), 16, 16, 18, range(18)),
    # Executioner: the skull over a body below its line, popping in and held.
    "skull": (AW, aw("10_UI_Symbols/Icon_Skull"), 28, 28, 10, range(10)),
    # Cold Read, Insight: the "!" over a wind-up being read.
    "alert": (AW, aw("10_UI_Symbols/Icon_Exclamation"), 28, 28, 10, range(10)),
    # Opportunist: the sleep over an enemy that has not noticed anyone.
    "unaware": (AW, aw("08_Status/Sleep_Zzz"), 48, 64, 18, range(18)),
    # Meteor: the ground torn up where the slam lands.
    "meteor_impact": (AW, aw("01_Explosions/Explosion_Debris"), 68, 62, 21, range(21)),
    # High Jump: the dust a lifted jump kicks up.
    "jump_dust": (AW, aw("07_Smoke_Dust/Dust_Jump"), 44, 24, 17, range(14)),
    # Overcharge: the flare of the charged blow ready.
    "charge_flare": (AW, aw("02_Impacts/Hit_Critical"), 56, 56, 7, range(7)),
    # Chain Reaction: sparks crawling over the body while it burns.
    "shock_sparks": (AW, aw("08_Status/Shock_Sparks"), 44, 50, 18, range(18)),
    # Shield Breaker, Breaker: a blow punching through the layer it broke.
    "pierce": (AW, aw("02_Impacts/Hit_Directional"), 48, 32, 4, range(4)),
    # Scavenger: the glint over a drop only its odds let fall.
    "glint": (AW, aw("09_Pickups_Sparkles/Coin_Sparkle"), 16, 16, 12, range(12)),
    # Field Dressing: the wisps rising off a pickup that healed more.
    "heal_burst": (AW, aw("06_Heal_Shield_Buffs/Heal_Burst"), 44, 76, 18, range(18)),
    # Undertow, Riptide: the water turning in on the player, as far as the
    # drag reaches.
    "whirlpool": (PVFX, pvfx("water/whirlpool"), 72, 48, 8, range(20)),
    # Double Jump, Kick Off, Updraft: the air pushed off under the feet.
    "air_burst": (PVFX, pvfx("impacts/shockwave-ring"), 120, 52, 8, range(10)),
}


def main():
    OUT.mkdir(parents=True, exist_ok=True)
    for name, (pack, sheet, width, height, columns, frames) in CLIPS.items():
        source = PACKS / pack / sheet
        crops = []
        for index in frames:
            x = (index % columns) * width
            y = (index // columns) * height
            crops += ["(", str(source), "-crop", f"{width}x{height}+{x}+{y}", "+repage", ")"]
        target = OUT / f"{name}.png"
        subprocess.run(
            ["magick", *crops, "-background", "none", "+append", "+repage", str(target)],
            check=True,
        )
        print(f"{pack}/{sheet} -> {target.relative_to(ASSETS)} ({len(frames)} frames)")


if __name__ == "__main__":
    main()
