#!/usr/bin/env python3
"""Cuts the character packs of `src/assets/packs/` into enemy sheets.

Each pack ships its clips its own way: one strip per clip, one image per frame,
a grid with a clip to a row, or (the Zweihander) a body and its effects drawn
apart. `TYPES` names, for each enemy type, the frames of each of its clips, and
this script lays every clip out the same way: one strip per clip in
`src/assets/enemies/<type>/<clip>.png`, every frame of a type the same size,
the body's middle (read off its first idle frame) in the middle of the frame,
and the rows nothing of any frame reaches cut off. Every pack draws its
figures facing right, as the game wants them.

What it prints for each type is what `assets/monster_sheets.rs` and
`enemies/enemy_types.yaml` need: the frame size, each clip's frames, and the
padding and head square of its first idle frame.

Run it with `just enemy-packs` after a pack or a clip changes. Needs
ImageMagick.
"""

import subprocess
import tempfile
from pathlib import Path

ASSETS = Path(__file__).resolve().parent.parent / "src/assets"
PACKS = ASSETS / "packs"


def strip(path, width, count=None, first=0, row=0, height=None):
    """Frames `first..first+count` of a row of `width` px cells."""
    source = PACKS / path
    size = subprocess.run(
        ["magick", "identify", "-format", "%w %h", str(source)],
        check=True,
        capture_output=True,
        text=True,
    ).stdout.split()
    height = height or int(size[1])
    count = count or int(size[0]) // width - first
    return [
        [(source, f"{width}x{height}+{(first + i) * width}+{row * height}")]
        for i in range(count)
    ]


def files(folder, first=1, last=None, name=None):
    """One image per frame, in the order of the number ending its name."""
    found = sorted(
        (PACKS / folder).glob(name or "*.png"),
        key=lambda path: int(path.stem.rsplit("_", 1)[-1]),
    )
    found = [p for p in found if first <= int(p.stem.rsplit("_", 1)[-1])]
    if last is not None:
        found = [p for p in found if int(p.stem.rsplit("_", 1)[-1]) <= last]
    return [[(path, None)] for path in found]


def over(body, effect):
    """The body's frames with the effect's drawn over them, cell for cell."""
    return [layers + fx for layers, fx in zip(body, effect)]


BOSS = "dark_fantasy_boss/dark fantasy big boss"
WETLANDS = "shadow_series/SHADOW Series - The Shadowed Wetlands Boss v1.1/Sprite Sheet"
HALLOKIN = "shadow_series/SHADOW Series - Hallokin (free)/sprite sheet 9x12 (free).png"
HUNTRESS = "shadow_series/SHADOW Series - The Blind Huntress/Sprite Sheet"
DUELIST = "pixel_art_09/Enemy01"
GUNNER = "pixel_art_09/Enemy02"
ZWEI = "zweihander"
MOON = "character_pack/Player"
WISP = "pixel_art_05/PlayerAllAnim"


def zwei(clip):
    return over(
        strip(f"{ZWEI}/Player/Zweihander_{clip}.png", 256),
        strip(f"{ZWEI}/fx/Zweihander_FX_{clip}.png", 256),
    )


def wisp(first, last):
    return files(WISP, first, last)


# Every clip of every type: its frames, as a list of layers drawn one over the
# other, each a source image and the cell of it (None for the whole image).
TYPES = {
    # The dark fantasy big boss: a horned brute that calls spikes out of the
    # floor (`attack 2`; the spikes themselves, `attack 2a`-`2c`, are left).
    "horned_brute": {
        "idle": strip(f"{BOSS} idle.png", 80),
        "walk": strip(f"{BOSS} walk.png", 80),
        "attack": strip(f"{BOSS} attack 2.png", 80),
        "hurt": strip(f"{BOSS} hit.png", 80),
        "dead": strip(f"{BOSS} death.png", 80),
    },
    # The Shadowed Wetlands Boss: a slash (Attack 1; Attack 2, a lash behind
    # it, is left), and a roll at the player out of the three-part third
    # attack (curling up, the ball, the unrolling slash).
    "wetlands_boss": {
        "idle": strip(f"{WETLANDS}/idle.png", 247),
        "walk": strip(f"{WETLANDS}/walk.png", 247),
        "attack": strip(f"{WETLANDS}/Attack 1.png", 247),
        "roll_attack": strip(f"{WETLANDS}/new/Pre-Attack 3.png", 247)
        + strip(f"{WETLANDS}/new/mid-Attack 3.png", 247)
        + strip(f"{WETLANDS}/new/end-Attack 3.png", 247),
        "hurt": strip(f"{WETLANDS}/new/hit.png", 247),
        "dead": strip(f"{WETLANDS}/new/death.png", 247),
    },
    # Hallokin: one grid of 192 x 128 cells, a clip to a row.
    "hallokin": {
        "idle": strip(HALLOKIN, 192, 6, row=0, height=128),
        "walk": strip(HALLOKIN, 192, 4, row=1, height=128),
        "jump": strip(HALLOKIN, 192, 3, row=2, height=128),
        "fall": strip(HALLOKIN, 192, 3, row=3, height=128),
        "attack": strip(HALLOKIN, 192, 7, row=4, height=128),
        "pounce": strip(HALLOKIN, 192, 5, row=5, height=128),
        "hurt": strip(HALLOKIN, 192, 1, row=7, height=128),
        "dead": strip(HALLOKIN, 192, 12, row=8, height=128),
    },
    "blind_huntress": {
        "idle": strip(f"{HUNTRESS}/1 - Idle.png", 240),
        "walk": strip(f"{HUNTRESS}/2 - Run.png", 240),
        "jump": strip(f"{HUNTRESS}/3 - jump.png", 240),
        "fall": strip(f"{HUNTRESS}/5 - fall.png", 240),
        # It slashes on its first frame: its first idle frame is the wind-up.
        "attack": strip(f"{HUNTRESS}/1 - Idle.png", 240, 1)
        + strip(f"{HUNTRESS}/10 - attack 1.png", 240),
        "combo": strip(f"{HUNTRESS}/11 - attack 3.png", 240),
        "dash_attack": strip(f"{HUNTRESS}/12 - dash attack.png", 240),
        "roll": strip(f"{HUNTRESS}/6 - dash.png", 240),
        "hurt": strip(f"{HUNTRESS}/14 - hit.png", 240),
        "dead": strip(f"{HUNTRESS}/15 - death.png", 240),
    },
    # PackCharacterPixelArt09's swordsman: no death of its own, so it dies
    # staggering (its stun).
    "crimson_duelist": {
        "idle": files(f"{DUELIST}/Idle"),
        "walk": files(f"{DUELIST}/Run"),
        "attack": files(f"{DUELIST}/Attack"),
        # The parry is the guard raised (its first five frames) and the clash
        # it answers a blow with (the rest).
        "guard": files(f"{DUELIST}/Parry", last=5),
        "counter": files(f"{DUELIST}/Parry", first=6),
        "hurt": files(f"{DUELIST}/Hit"),
        "dead": files(f"{DUELIST}/Stun"),
    },
    # Its gunner: the gun raised and fired in one clip, and its bullet.
    "glyph_gunner": {
        "idle": files(f"{GUNNER}/Idle"),
        "walk": files(f"{GUNNER}/Run"),
        "attack": files(f"{GUNNER}/PrepareShoot") + files(f"{GUNNER}/Shoot"),
        "hurt": files(f"{GUNNER}/Hit", last=3),
        "dead": files(f"{GUNNER}/Hit"),
    },
    # Zweihander Combat: a player pack, its swings and their trails drawn
    # apart. No walk and no death: it walks in its stance and dies flinching.
    "zweihander": {
        "idle": strip(f"{ZWEI}/Player/Zweihander_Idle.png", 256),
        "walk": strip(f"{ZWEI}/Player/Zweihander_Idle.png", 256),
        "attack": zwei("Combo_01"),
        "combo": zwei("Combo_02"),
        "heavy": zwei("Heavy_Attack"),
        "guard": strip(f"{ZWEI}/Player/Zweihander_Parry.png", 256),
        "counter": zwei("Counter"),
        "roll": zwei("Dash_Front"),
        "hurt": zwei("Hit"),
        "dead": zwei("Hit"),
    },
    # CharacterPack: a player pack. No flinch and no death: it flinches in
    # its first idle frames and dies falling.
    "moon_blade": {
        "idle": files(f"{MOON}/Idle"),
        "walk": files(f"{MOON}/Run"),
        "jump": files(f"{MOON}/JumpUp"),
        "fall": files(f"{MOON}/JumpDown"),
        "attack": files(f"{MOON}/Attack01"),
        # The pack's Shoot is a dash cut through the air; it cuts on its first
        # frame, so two idle frames come first as its wind-up.
        "dash_slash": files(f"{MOON}/Idle", last=2) + files(f"{MOON}/Shoot"),
        "hurt": files(f"{MOON}/Idle", last=2),
        "dead": files(f"{MOON}/JumpDown"),
    },
    # PackCharacterPixelArt05: a player pack drawn as one run of 199 frames,
    # its clips the tags of its .aseprite (Idle 1-8, Run 10-17, Dash 19-28,
    # Attack01 30-47, Attack02 48-68, Death 95-128). No flinch: its first
    # idle frames.
    "antler_wisp": {
        "idle": wisp(1, 8),
        "walk": wisp(10, 17),
        "attack": wisp(30, 47),
        "cast": wisp(48, 68),
        "hurt": wisp(1, 2),
        "dead": wisp(95, 128),
    },
}


def run(*args):
    return subprocess.run(
        ["magick", *map(str, args)], check=True, capture_output=True, text=True
    ).stdout


def box(geometry):
    """`WxH+X+Y` as (left, top, right, bottom), exclusive of right/bottom."""
    size, x, y = geometry.split("+")
    width, height = map(int, size.split("x"))
    return int(x), int(y), int(x) + width, int(y) + height


with tempfile.TemporaryDirectory() as temp:
    temp = Path(temp)
    for type_id, clips in TYPES.items():
        flat = {}
        boxes = []
        for clip, frames in clips.items():
            flat[clip] = []
            for index, layers in enumerate(frames):
                target = temp / f"{type_id}_{clip}_{index}.png"
                args = []
                for source, crop in layers:
                    args += ["(", source] + (["-crop", crop] if crop else []) + ["+repage", ")"]
                if len(layers) > 1:
                    args += ["-background", "none", "-flatten"]
                trim = run(*args, "-write", f"PNG32:{target}", "-format", "%@ %w %h", "info:")
                geometry, width, height = trim.split()
                flat[clip].append(target)
                if not geometry.startswith("0x0"):
                    boxes.append(box(geometry))
        idle_trim = run(flat["idle"][0], "-format", "%@ %w %h", "info:").split()
        left, top, right, bottom = box(idle_trim[0])
        # The body's middle is where the idle frame is most solid, column by
        # column: a sword held out to one side is thin, the trunk is not.
        columns = [
            float(line.split("(")[1].split(")")[0].split(",")[0])
            for line in run(
                flat["idle"][0], "-alpha", "extract", "-scale", f"{idle_trim[1]}x1!", "txt:"
            ).splitlines()[1:]
        ]
        smooth = [sum(columns[max(0, x - 3) : x + 4]) for x in range(len(columns))]
        anchor = max(range(len(smooth)), key=smooth.__getitem__)
        reach = max(max(anchor - b[0], b[2] - anchor) for b in boxes)
        frame_width = 2 * reach
        top_all = min(b[1] for b in boxes)
        bottom_all = max(b[3] for b in boxes)
        frame_height = bottom_all - top_all
        folder = ASSETS / "enemies" / type_id
        folder.mkdir(parents=True, exist_ok=True)
        print(f"{type_id}: {frame_width} x {frame_height}")
        for clip, frames in flat.items():
            target = folder / f"{clip}.png"
            run(
                *frames,
                "-background",
                "none",
                "-gravity",
                "NorthWest",
                "-extent",
                f"{frame_width}x{frame_height}{anchor - reach:+d}{top_all:+d}",
                "+append",
                "-strip",
                f"PNG32:{target}",
            )
            print(f"  {clip}: {len(frames)} frames")
        figure = bottom - top
        head = min(right - left, figure // 2)
        print(
            f"  top_padding_px: {top - top_all}, bottom_padding_px: {bottom_all - bottom}, "
            f"origin_above_feet_px: {figure // 2}, "
            f"head_px: {{ x: {reach - head // 2}, y: {top - top_all}, size: {head} }}"
        )

# The gunner's bullet, its shot, as it ships.
run(PACKS / GUNNER / "Enemy02_Bullet.png", "-strip", f"PNG32:{ASSETS / 'enemies/glyph_gunner/shot.png'}")
