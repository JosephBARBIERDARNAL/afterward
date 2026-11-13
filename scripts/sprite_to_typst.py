#!/usr/bin/env python3
"""Writes a player sprite sheet as Typst text art.

    python3 scripts/sprite_to_typst.py pack/Run.png art/player/sheets/run.typ

The PNG is a row of 128 x 128 frames. Each frame becomes a block of text, one
character per pixel, cropped to the window that holds every frame of the sheet.
Colours are looked up in `art/player/_player.typ`, by the colours the packs
draw the body and its effects in (`PACK`, below, for the body, whose parts that
file now colours its own way); one that is not there is declared in the sheet
itself, under a key of its own.

This is how a sheet of the Craftpix packs first came into `art/player/sheets/`.
It overwrites its target, so it is for bringing a new sheet in, not for a sheet
that has been redrawn by hand since. Render with `just art`. Needs ImageMagick.
"""

import re
import subprocess
import sys
from pathlib import Path

FRAME = 128
SHARED = Path(__file__).resolve().parent.parent / "art/player/_player.typ"
# Keys handed to a sheet's own colours, in order, skipping the shared ones.
SPARE_KEYS = "0123456789ABCDEFGHIJKLMNPQRSTUVXZefijkpqrtuvxz"

source, target = Path(sys.argv[1]), Path(sys.argv[2])
# The part of the body each of the packs' five body colours stands for.
PACK = {"e6ecf5": "h", "4660a8": "l", "3a509a": "m", "2a3a78": "d", "24306a": "s"}
effects = SHARED.read_text().split("#let effects")[1]
shared = PACK | {
    colour: key for key, colour in re.findall(r'"(.)": rgb\("#([0-9a-f]{6})"\)', effects)
}

width, height = map(
    int, subprocess.check_output(["magick", "identify", "-format", "%w %h", source]).split()
)
if width % FRAME or height != FRAME:
    sys.exit(f"{source}: {width}x{height} is not a row of {FRAME}x{FRAME} frames")
raw = subprocess.check_output(["magick", source, "-depth", "8", "rgba:-"])

# pixel[y][x] is a colour as "rrggbb", or None where nothing is drawn.
pixel = []
for y in range(height):
    row = []
    for x in range(width):
        r, g, b, a = raw[(y * width + x) * 4 :][:4]
        if a not in (0, 255):
            sys.exit(f"{source}: a half-transparent pixel at {x},{y}; the sheets are flat colour")
        row.append(f"{r:02x}{g:02x}{b:02x}" if a else None)
    pixel.append(row)

drawn = [(x % FRAME, y) for y in range(height) for x in range(width) if pixel[y][x]]
left, right = min(x for x, _ in drawn), max(x for x, _ in drawn)
top, bottom = min(y for _, y in drawn), max(y for _, y in drawn)

# The sheet's own colours, most used first.
count = {}
for row in pixel:
    for colour in row:
        if colour and colour not in shared:
            count[colour] = count.get(colour, 0) + 1
spare = [key for key in SPARE_KEYS if key not in shared.values()]
own = dict(zip(sorted(count, key=count.get, reverse=True), spare, strict=False))
if len(own) < len(count):
    sys.exit(f"{source}: {len(count)} colours of its own, more than there are keys")
key_of = shared | own

frames = width // FRAME
lines = [
    f"// The hero's `{target.stem}` sheet: {frames} frames, each drawn in the window",
    f"// of its cell that starts at ({left}, {top}). See ../_player.typ.",
]
if own:
    lines += ['#import "../_player.typ": sheet', "", "#let own = ("]
    lines += [f'  "{key}": rgb("#{colour}"),' for colour, key in own.items()]
    lines += [")", "", f"#sheet(at: ({left}, {top}), own: own, ("]
else:
    lines += ['#import "../_player.typ": sheet', "", f"#sheet(at: ({left}, {top}), ("]
for frame in range(frames):
    lines += [f"  // Frame {frame}", "  ```"]
    for y in range(top, bottom + 1):
        cells = pixel[y][frame * FRAME + left : frame * FRAME + right + 1]
        lines.append("  " + "".join(key_of[c] if c else "." for c in cells))
    lines.append("  ```,")
lines.append("))")
target.write_text("\n".join(lines) + "\n")
print(f"{source} -> {target} ({frames} frames, {right - left + 1} x {bottom - top + 1})")
