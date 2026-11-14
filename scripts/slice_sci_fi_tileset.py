#!/usr/bin/env python3
"""Cuts the sci-fi platformer pack's tileset into one PNG per piece.

The pack ships everything in one 288x160 sheet, `source_tileset.png`, drawn on
a 16 px grid, its robot one tile tall. The tiles keep that scale (16 px per H);
the props are drawn at `px_per_h` of `props/props.yaml`.
Each piece below is cut out as it stands, with no padding:

- `TILES` are seamless 16 px squares, written under
  `src/assets/surfaces/textures/sci_fi/` and listed as materials in
  `surfaces/materials.yaml` (with `tile_h: 1`, the pack's scale).
- `PROPS` are everything else, written under `src/assets/props/sci_fi/` and
  listed in `props/props.yaml`, with the size printed here; `ANIMATIONS` are
  props whose frames are laid side by side in one strip.

Run it with `just sci-fi` after the sheet or a piece changes. Needs
ImageMagick.
"""

import subprocess
from pathlib import Path

ASSETS = Path(__file__).resolve().parent.parent / "src/assets"
SOURCE = ASSETS / "props/sci_fi/source_tileset.png"

# Name: (x, y, width, height) in the sheet.
TILES = {
    "dark": (0, 0, 16, 16),
    "dark_rivets": (16, 0, 16, 16),
}

# The riveted wall is never drawn whole in the sheet: every one of its tiles
# is overlapped by the green window somewhere. Its left half is taken from
# the tile left of the window, its right half from the tile right of it.
RIVETED_WALL_HALVES = [(0, 16, 8, 16), (40, 16, 8, 16)]

PROPS = {
    # The green window set in a riveted wall, and the walls' own panels.
    "green_window": (0, 16, 48, 48),
    "green_panel": (48, 16, 32, 32),
    "steel_crate": (80, 16, 32, 32),
    "dark_panel": (0, 64, 32, 32),
    "steel_frame": (53, 69, 38, 38),
    "strut": (35, 67, 11, 11),
    "rivets": (34, 83, 13, 12),
    # A pipe and a belt that tile sideways, the belt's capped ends, a bolted block.
    "green_pipe": (128, 16, 16, 16),
    "steel_block": (144, 16, 16, 16),
    "conveyor": (128, 32, 48, 16),
    "conveyor_middle": (144, 32, 16, 16),
    # Lights, crates and spikes.
    "ceiling_lamp": (194, 16, 28, 27),
    "small_crate": (224, 22, 16, 10),
    "crate_stack": (242, 12, 27, 20),
    "spikes": (192, 56, 96, 8),
    # Platforms: the hover pads, green at one end or the other, and the bench.
    "hover_pad_left": (0, 112, 42, 13),
    "hover_pad_right": (54, 112, 42, 13),
    "bench": (0, 144, 48, 15),
}

# Animated props: each frame's box in the sheet, laid left to right in one
# strip, which `props/props.yaml` gives a frame count and a pace.
ANIMATIONS = {
    # A laser gate: off, then lit thin, double and bright.
    "laser": [(194 + 16 * frame, 80, 13, 48) for frame in range(4)],
}


def cut(name, box, folder):
    x, y, width, height = box
    target = folder / f"{name}.png"
    folder.mkdir(parents=True, exist_ok=True)
    subprocess.run(
        [
            "magick",
            SOURCE,
            "-crop",
            f"{width}x{height}+{x}+{y}",
            "+repage",
            "-strip",
            target,
        ],
        check=True,
    )
    return target


def main():
    textures = ASSETS / "surfaces/textures/sci_fi"
    for name, box in TILES.items():
        cut(name, box, textures)
    halves = [
        ["(", SOURCE, "-crop", f"{w}x{h}+{x}+{y}", "+repage", ")"]
        for x, y, w, h in RIVETED_WALL_HALVES
    ]
    subprocess.run(
        ["magick", *halves[0], *halves[1], "+append", "-strip", textures / "riveted_wall.png"],
        check=True,
    )
    for name, box in PROPS.items():
        cut(name, box, ASSETS / "props/sci_fi")
        print(f"sci_fi_{name}: {box[2]}x{box[3]} px")
    for name, boxes in ANIMATIONS.items():
        frames = [
            part
            for x, y, w, h in boxes
            for part in ["(", SOURCE, "-crop", f"{w}x{h}+{x}+{y}", "+repage", ")"]
        ]
        subprocess.run(
            ["magick", *frames, "+append", "-strip", ASSETS / f"props/sci_fi/{name}.png"],
            check=True,
        )
        print(f"sci_fi_{name}: {len(boxes)} frames of {boxes[0][2]}x{boxes[0][3]} px")


if __name__ == "__main__":
    main()
