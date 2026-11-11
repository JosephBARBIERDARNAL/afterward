#!/usr/bin/env python3
"""Draws the dark robots from the ordinary ones.

Every sheet the game plays from `src/assets/enemies/type1/` is recoloured into
the same path under `src/assets/enemies/type1_dark/`: the chassis is taken down
to a violet near-black, the warm paint on it turns to a cold magenta, and the
cold lights (lens, blade, muzzle) are left as bright as they were, so they are
what the eye finds. Shapes and frames are untouched, which is why the dark
types can share the frame counts of `assets/enemy_sheets.rs`.

Run it with `just dark-robots` after a source sheet changes. Needs ImageMagick.
"""

import colorsys
import re
import subprocess
from pathlib import Path

ENEMIES = Path(__file__).resolve().parent.parent / "src/assets/enemies"
# The sheets `assets/enemy_sheets.rs` loads for the three robots.
SHEETS = {
    "Swordsman": ["Idle", "Attack_1", "Hurt", "Dead"],
    "Infantryman": ["Idle", "Walk", "Shot_1", "Hurt", "Dead"],
    "Destroyer": ["Idle", "Walk", "Shot_1", "Hurt", "Dead"],
}

# The chassis: how dark it goes, and the violet it leans into.
CHASSIS_VALUE = 0.46
CHASSIS_GAMMA = 0.85
CHASSIS_TINT = (0.62, 0.50, 1.0)
CHASSIS_TINT_MIX = 0.45
# The warm paint (yellow to orange) becomes this hue, a little dimmer.
PAINT_HUE = 300 / 360
PAINT_VALUE = 0.85

PIXEL = re.compile(r"^(\d+,\d+): \((\d+),(\d+),(\d+),(\d+)\)")


def darker(red, green, blue):
    hue, saturation, value = colorsys.rgb_to_hsv(red / 255, green / 255, blue / 255)
    light = value > 0.8 and saturation > 0.15 and 140 / 360 <= hue <= 200 / 360
    if not light and (saturation < 0.45 or value < 0.45):
        level = CHASSIS_VALUE * value**CHASSIS_GAMMA
        grey = [level * channel for channel in CHASSIS_TINT]
        plain = colorsys.hsv_to_rgb(hue, saturation, level)
        mixed = [
            tinted * CHASSIS_TINT_MIX + kept * (1 - CHASSIS_TINT_MIX)
            for tinted, kept in zip(grey, plain)
        ]
    elif 20 / 360 <= hue <= 75 / 360:
        mixed = colorsys.hsv_to_rgb(PAINT_HUE, saturation * 0.8, value * PAINT_VALUE)
    else:
        mixed = (red / 255, green / 255, blue / 255)
    return [round(channel * 255) for channel in mixed]


for robot, sheets in SHEETS.items():
    for sheet in sheets:
        source = ENEMIES / "type1" / robot / f"{sheet}.png"
        target = ENEMIES / "type1_dark" / robot / f"{sheet}.png"
        target.parent.mkdir(parents=True, exist_ok=True)
        text = subprocess.run(
            ["magick", str(source), "-depth", "8", "txt:-"],
            check=True,
            capture_output=True,
            text=True,
        ).stdout.splitlines()
        lines = [text[0]]
        for line in text[1:]:
            spot, red, green, blue, alpha = PIXEL.match(line).groups()
            if alpha == "0":
                channels = [0, 0, 0, 0]
            else:
                channels = darker(int(red), int(green), int(blue)) + [int(alpha)]
            lines.append(f"{spot}: ({','.join(map(str, channels))})")
        subprocess.run(
            ["magick", "txt:-", "-strip", f"PNG32:{target}"],
            check=True,
            input="\n".join(lines) + "\n",
            text=True,
        )
        print(target.relative_to(ENEMIES.parent.parent.parent))
