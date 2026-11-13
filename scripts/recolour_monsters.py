#!/usr/bin/env python3
"""Draws the horror pack's monsters in their other palettes.

The pack's sheets (`src/assets/enemies/horror/*.png`) are drawn in four flat
colours: the body, the red of the eye and of what an attack throws, the grey of
a flinch and the white of a flash. Each palette of `PALETTES` swaps the first
two and leaves the rest, and its sheets are written to a folder of its own name
beside the originals. Shapes and frames are untouched, which is why the
recoloured types share the clips of `assets/monster_sheets.rs`.

Run it with `just monsters` after a source sheet or a palette changes. Needs
ImageMagick.
"""

import subprocess
from pathlib import Path

HORROR = Path(__file__).resolve().parent.parent / "src/assets/enemies/horror"
BODY = "#181425"
EYE = "#FF0044"
# Each palette's body and eye.
PALETTES = {
    "crimson": ("#E0142A", "#FFE83D"),
    "cobalt": ("#1447FF", "#FF9A1F"),
    "emerald": ("#00B84A", "#FF2BD6"),
}

for palette, (body, eye) in PALETTES.items():
    for source in sorted(HORROR.glob("*.png")):
        target = HORROR / palette / source.name
        target.parent.mkdir(exist_ok=True)
        subprocess.run(
            ["magick", str(source)]
            + ["-fill", body, "-opaque", BODY]
            + ["-fill", eye, "-opaque", EYE]
            + ["-strip", f"PNG32:{target}"],
            check=True,
        )
        print(target.relative_to(HORROR.parent.parent.parent.parent))
