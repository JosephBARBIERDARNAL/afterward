#!/usr/bin/env python3
"""Cuts the sci-fi platformer pack into the game's materials, props, devices,
enemies, effects and background.

The pack (premium version, `src/assets/packs/sci_fi/`, kept as it ships) is
drawn on a 16 px grid. Every piece below is cut on that grid, so each is a
whole number of cells (0.5 H each at the props' 32 px per H) and pieces placed
side by side meet edge to edge, the way the pack builds its own scenes. A
piece smaller than its cells keeps their transparent margin.

- `TEXTURES` are seamless 16 px squares, listed as materials in
  `surfaces/materials.yaml` (`tile_h: 0.5`). A trim run is its jointed tile
  with the joint painted over (`PAINT`), so a row of it runs unbroken.
- `PROPS` are still pieces, listed in `props/props.yaml`.
- `STRIPS` are animations and frame sets: each frame's box, laid left to right
  in one row, read by `props/props.yaml`, the device art of
  `devices/devices.yaml`, the enemies and the effects.
- `COPIES` are taken whole: the background layers.

Run it with `just sci-fi` after the pack or a piece changes. Needs
ImageMagick.
"""

import shutil
import subprocess
from pathlib import Path

ASSETS = Path(__file__).resolve().parent.parent / "src/assets"
PACK = ASSETS / "packs/sci_fi"

TILES = "tiles/tiles.png"
MISC = "background-wall/misc.png"
WALLS = "background-wall/wall-tiles.png"
PIPES = "background-wall/pipes.png"
PLATFORMS = "platforms/platforms.png"
PROP_SHEET = "props/props.png"

# Name: (sheet, x, y, width, height).
TEXTURES = {
    "dark": (TILES, 48, 112, 16, 16),
    "dark_rivets": (TILES, 0, 112, 16, 16),
    "riveted_wall": (TILES, 0, 96, 16, 16),
    "conveyor": (TILES, 16, 64, 16, 16),
    # Trim runs, one per side a surface can face: green along the top of a
    # floor, the underside of a ceiling, the left or right face of a wall.
    "green_edge": (TILES, 144, 0, 16, 16),
    "trim_ceiling": (TILES, 208, 64, 16, 16),
    "trim_wall_left": (TILES, 128, 64, 16, 16),
    "trim_wall_right": (TILES, 224, 0, 16, 16),
}

# Joints painted over in a texture: (texture, the box copied, where it goes).
PAINT = [
    ("green_edge", (4, 0, 3, 9), (7, 0)),
    ("trim_ceiling", (4, 7, 3, 9), (7, 7)),
    ("trim_wall_left", (0, 4, 9, 3), (0, 7)),
    ("trim_wall_right", (7, 4, 9, 3), (7, 7)),
]

PROPS = {
    # Tiles: the green window set in a riveted wall, panels, crates, the steel
    # door block, and the trim's special cells, placed where a run needs one.
    "green_window": (TILES, 0, 0, 48, 48),
    "green_panel": (TILES, 48, 0, 32, 32),
    "steel_crate": (TILES, 80, 0, 32, 32),
    "steel_door_block": (TILES, 80, 48, 32, 32),
    "green_pipe": (TILES, 144, 0, 16, 16),
    "steel_block": (TILES, 208, 0, 16, 16),
    "conveyor": (TILES, 0, 64, 48, 16),
    "conveyor_middle": (TILES, 16, 64, 16, 16),
    "trim_floor_ramp_down": (TILES, 160, 0, 16, 16),
    "trim_floor_notch": (TILES, 176, 0, 16, 16),
    "trim_floor_ramp_up": (TILES, 192, 0, 16, 16),
    "trim_ceiling_ramp_down": (TILES, 160, 64, 16, 16),
    "trim_ceiling_notch": (TILES, 176, 64, 16, 16),
    "trim_ceiling_ramp_up": (TILES, 192, 64, 16, 16),
    "trim_ceiling_joint": (TILES, 208, 64, 16, 16),
    "trim_wall_left_notch": (TILES, 128, 16, 16, 16),
    "trim_wall_left_ramp": (TILES, 128, 32, 16, 16),
    "trim_wall_left_ramp_back": (TILES, 128, 48, 16, 16),
    "trim_wall_left_joint": (TILES, 128, 64, 16, 16),
    "trim_wall_right_joint": (TILES, 224, 0, 16, 16),
    "trim_wall_right_notch": (TILES, 224, 16, 16, 16),
    "trim_wall_right_ramp": (TILES, 224, 32, 16, 16),
    "trim_wall_right_ramp_back": (TILES, 224, 48, 16, 16),
    "steel_cap_top_left": (TILES, 128, 0, 16, 16),
    "steel_cap_bottom_right": (TILES, 224, 64, 16, 16),
    "steel_plate": (TILES, 144, 64, 16, 16),
    "dark_bevel": (TILES, 32, 96, 16, 16),
    # Background walls: scenery hung on the walls behind the action.
    "elevator_door": (MISC, 0, 0, 48, 48),
    "wall_window": (MISC, 48, 16, 64, 32),
    "bolt_plate": (MISC, 112, 16, 32, 32),
    "vent_grille": (MISC, 144, 16, 32, 32),
    "fuse_box": (MISC, 176, 16, 32, 32),
    "pipe_bend": (MISC, 208, 16, 16, 32),
    "shutter": (MISC, 0, 64, 48, 32),
    "pipe_elbow": (MISC, 48, 64, 16, 32),
    "light_strip_vertical": (MISC, 64, 64, 16, 48),
    "light_strip": (MISC, 80, 64, 48, 16),
    "round_vent": (MISC, 80, 80, 32, 32),
    "grate_panel": (MISC, 128, 64, 48, 32),
    "strut": (MISC, 160, 64, 16, 16),
    "rivets": (MISC, 160, 80, 16, 16),
    "steel_frame": (MISC, 176, 64, 48, 48),
    "dark_panel": (WALLS, 0, 0, 32, 32),
    "rounded_frame": (WALLS, 32, 0, 48, 48),
    "square_frame": (WALLS, 80, 0, 32, 32),
    "pipe_vertical": (PIPES, 0, 0, 16, 48),
    "pipe_vertical_short": (PIPES, 16, 0, 16, 48),
    "pipe_junction": (PIPES, 32, 0, 48, 48),
    "pipe_cross": (PIPES, 80, 16, 16, 16),
    "pipe_horizontal": (PIPES, 96, 0, 48, 16),
    "pipe_horizontal_short": (PIPES, 96, 16, 48, 16),
    # Platforms and props.
    "green_slab": (PLATFORMS, 0, 0, 48, 16),
    "bench": (PLATFORMS, 48, 0, 48, 16),
    "green_step": (PLATFORMS, 96, 0, 32, 16),
    "hover_pad_left": (PLATFORMS, 0, 16, 48, 16),
    "hover_pad_right": (PLATFORMS, 48, 16, 48, 16),
    "small_crate": (PROP_SHEET, 0, 16, 16, 16),
    "crate_stack": (PROP_SHEET, 16, 0, 32, 32),
    "barrel": (PROP_SHEET, 48, 16, 16, 16),
    "terminal": (PROP_SHEET, 64, 0, 32, 32),
    "lamp_off": ("props/lamp.png", 0, 0, 32, 16),
    "ceiling_lamp": ("props/lamp.png", 48, 0, 32, 32),
    "spikes": ("traps/spikes.png", 0, 0, 96, 16),
}


def row(sheet, x, y, width, height, count, step=None):
    """`count` frames of one row, `step` px apart (their width by default)."""
    return [(sheet, x + i * (step or width), y, width, height) for i in range(count)]


# Name: (folder under src/assets, frames).
STRIPS = {
    # Animated scenery.
    "fan": ("props/sci_fi", row("background-wall/fans.png", 0, 0, 32, 32, 4)),
    "framed_fan": ("props/sci_fi", row("background-wall/fans.png", 0, 32, 32, 32, 4)),
    "laser": ("props/sci_fi", row("traps/lasers.png", 0, 0, 16, 48, 4)),
    "hover_platform": ("props/sci_fi", row("platforms/hovering-platforms.png", 0, 0, 64, 32, 5)),
    "hover_platform_small": (
        "props/sci_fi",
        row("platforms/hovering-platforms.png", 0, 32, 32, 32, 5),
    ),
    # Wall lights: each off, then lit.
    **{
        f"wall_light_{name}": (
            "props/sci_fi",
            [("background-wall/wall-lights.png", x, y, 32, 32) for y in (0, 32)],
        )
        for name, x in (("capsule", 0), ("small", 48), ("dashes", 96))
    },
    # Devices: traps and interactables, their frames in the order the game
    # plays them.
    "laser_red": ("devices/sci_fi", row("traps/lasers.png", 0, 0, 16, 48, 4)),
    "laser_blue": ("devices/sci_fi", row("traps/lasers.png", 0, 64, 16, 48, 4)),
    **{
        f"fire_{colour}": (
            "devices/sci_fi",
            row("traps/fire-traps.png", 0, top, 16, 32, 3)
            + row("traps/fire-traps.png", 0, top + 32, 16, 32, 5)
            + row("traps/fire-traps.png", 0, top + 64, 16, 32, 4),
        )
        for colour, top in (("orange", 0), ("blue", 112))
    },
    "press": (
        "devices/sci_fi",
        row("traps/press-trap.png", 0, 0, 48, 80, 9) + row("traps/press-trap.png", 0, 80, 48, 80, 6),
    ),
    "blade_gear": ("devices/sci_fi", row("traps/spinning-blade.png", 0, 0, 32, 32, 3)),
    "blade_disc": ("devices/sci_fi", row("traps/spinning-blade.png", 96, 0, 32, 32, 2)),
    "turret": (
        "devices/sci_fi",
        row("traps/turret.png", 0, 0, 32, 16, 5) + row("traps/turret.png", 0, 32, 32, 16, 4),
    ),
    "conveyor_belt": ("devices/sci_fi", row("traps/conveyor-belt.png", 0, 0, 48, 16, 4)),
    # Six cells of spikes, laid one after the other along a row of them.
    "spikes": ("devices/sci_fi", row("traps/spikes.png", 0, 0, 16, 16, 6)),
    "folding_green": ("devices/sci_fi", row("platforms/folding-platforms.png", 0, 0, 48, 16, 4)),
    "folding_steel": ("devices/sci_fi", row("platforms/folding-platforms.png", 0, 16, 48, 16, 4)),
    "button": ("devices/sci_fi", row("interactables/button.png", 0, 14, 16, 16, 4)),
    "door": ("devices/sci_fi", row("interactables/door.png", 0, 0, 16, 32, 7)),
    # The ladder's top and a middle rung, from its one column.
    "ladder": ("devices/sci_fi", [("interactables/ladder.png", 0, y, 16, 16) for y in (0, 16)]),
    # Enemies: a frame per clip cell, one strip per clip.
    "orb_idle": ("enemies/sci_fi", row("enemies/enemy-01.png", 0, 0, 48, 48, 3) + row("enemies/enemy-01.png", 0, 48, 48, 48, 2)),
    "walker_idle": ("enemies/sci_fi", row("enemies/enemy-02.png", 0, 0, 48, 48, 5)),
    "walker_walk": ("enemies/sci_fi", row("enemies/enemy-02.png", 0, 48, 48, 48, 6)),
    "walker_shoot": ("enemies/sci_fi", row("enemies/enemy-02.png", 0, 144, 48, 48, 6)),
    "drone_fly": ("enemies/sci_fi", row("enemies/enemy-03.png", 0, 0, 48, 48, 4)),
    "drone_shoot": ("enemies/sci_fi", row("enemies/enemy-03.png", 0, 48, 48, 48, 4)),
    # Effects.
    "dust": ("effects/sci_fi", row("effects/dust.png", 0, 0, 16, 16, 5)),
    "explosion": ("effects/sci_fi", row("effects/explosion.png", 0, 0, 48, 48, 9)),
    "shot_red": ("effects/sci_fi", row("effects/projectiles.png", 0, 16, 16, 16, 4)),
    "shot_blue": ("effects/sci_fi", row("effects/projectiles.png", 0, 48, 16, 16, 4)),
    "shot_burst_red": ("effects/sci_fi", row("effects/projectile-explosion.png", 0, 0, 16, 16, 4)),
    "shot_burst_blue": ("effects/sci_fi", row("effects/projectile-explosion.png", 0, 16, 16, 16, 4)),
    "muzzle_red": ("effects/sci_fi", row("effects/muzzle-flashes.png", 0, 0, 16, 16, 4)),
    "muzzle_blue": ("effects/sci_fi", row("effects/muzzle-flashes.png", 0, 48, 16, 16, 4)),
}

COPIES = {f"background/sci_fi/layer-{n}.png": f"background/layer-{n}.png" for n in range(6)}


def crop(sheet, x, y, width, height):
    return ["(", PACK / sheet, "-crop", f"{width}x{height}+{x}+{y}", "+repage", ")"]


def magick(*args):
    subprocess.run(["magick", *map(str, args)], check=True)


def main():
    textures = ASSETS / "surfaces/textures/sci_fi"
    textures.mkdir(parents=True, exist_ok=True)
    for name, box in TEXTURES.items():
        magick(*crop(*box), "-strip", textures / f"{name}.png")
    for name, (x, y, width, height), (to_x, to_y) in PAINT:
        target = textures / f"{name}.png"
        magick(
            target,
            "(", "+clone", "-crop", f"{width}x{height}+{x}+{y}", "+repage", ")",
            "-geometry", f"+{to_x}+{to_y}", "-composite", "-strip", target,
        )
    props = ASSETS / "props/sci_fi"
    props.mkdir(parents=True, exist_ok=True)
    for name, box in PROPS.items():
        magick(*crop(*box), "-strip", props / f"{name}.png")
    for name, (folder, frames) in STRIPS.items():
        target = ASSETS / folder / f"{name}.png"
        target.parent.mkdir(parents=True, exist_ok=True)
        parts = [part for frame in frames for part in crop(*frame)]
        magick(*parts, "-background", "none", "+append", "-strip", target)
    for target, source in COPIES.items():
        (ASSETS / target).parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(PACK / source, ASSETS / target)


if __name__ == "__main__":
    main()
