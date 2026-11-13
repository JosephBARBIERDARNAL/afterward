#!/usr/bin/env python3
"""Cuts the Black Knight's atlas into one strip per clip.

The pack ships every animation in one sheet, `source_atlas.png`: an 18 by 18
grid of 144x64 cells read left to right and top to bottom, each cell ruled
with a 1 px border in its clip's colour, and a clip running on from the end of
one row into the next. The game's sheets are single rows of bare frames, so
each clip below is cut out cell by cell, its borders dropped, and laid in a
row under `src/assets/enemies/black_knight/<clip>.png`.

A cell's figure stands 4 px left of its middle. Every frame is widened on the
left until the figure is centred, so the knight turns around on the spot
instead of stepping sideways when its sprite is flipped. The frame size here
is the one `assets/boss_sheets.rs` reads the strips with.

Run it with `just black-knight` after the atlas or a clip changes. Needs
ImageMagick.
"""

import subprocess
from pathlib import Path

KNIGHT = Path(__file__).resolve().parent.parent / "src/assets/enemies/black_knight"
COLUMNS = 18
CELL_WIDTH, CELL_HEIGHT = 144, 64
FRAME_WIDTH, FRAME_HEIGHT = 150, 62

# Each clip is the cells it is made of, as (first cell, count) runs counted
# from the top left. The pack's own name for each run is given beside it.
CLIPS = {
    # Unarmed idle: before it has noticed anyone, the sword not yet drawn.
    "unarmed_idle": [(38, 6)],
    # Weapon on: the sword drawn, the moment it notices the player.
    "draw": [(0, 18)],
    # Weapon idle and weapon run.
    "idle": [(32, 6)],
    "walk": [(160, 12)],
    # Weapon jump, ending on the one mid-air frame, which is held.
    "jump": [(197, 7)],
    # Weapon fall (light) and weapon land.
    "fall": [(204, 6)],
    "land": [(216, 6)],
    # Attack 1 without its recovery, running on into attack 2: two swings.
    "combo": [(44, 13), (63, 8)],
    # Attack 3: five frames coiled, then the dash.
    "lunge": [(71, 13)],
    # Parry start, the parry itself, and the miss: the guard lowered unanswered.
    "guard": [(257, 6)],
    "counter": [(263, 15)],
    "guard_miss": [(278, 3)],
    # Weapon roll, weapon buff, weapon hurt and death.
    "roll": [(124, 12)],
    "buff": [(312, 12)],
    "hurt": [(286, 5)],
    "dead": [(291, 9)],
}


def main():
    atlas = KNIGHT / "source_atlas.png"
    for clip, runs in CLIPS.items():
        command = ["magick"]
        for first, count in runs:
            for cell in range(first, first + count):
                left = cell % COLUMNS * CELL_WIDTH + 1
                top = cell // COLUMNS * CELL_HEIGHT + 1
                command += [
                    "(",
                    str(atlas),
                    "-crop",
                    f"{CELL_WIDTH - 2}x{CELL_HEIGHT - 2}+{left}+{top}",
                    "+repage",
                    "-background",
                    "none",
                    "-splice",
                    f"{FRAME_WIDTH - CELL_WIDTH + 2}x0",
                    ")",
                ]
        command += ["+append", "-strip", f"PNG32:{KNIGHT / clip}.png"]
        subprocess.run(command, check=True)
        print(f"{clip}: {sum(count for _, count in runs)} frames")


if __name__ == "__main__":
    main()
