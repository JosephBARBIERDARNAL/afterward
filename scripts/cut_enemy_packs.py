#!/usr/bin/env python3
"""Cuts the character packs of `src/assets/packs/` into enemy sheets.

Each pack ships its clips its own way: one strip per clip, one image per frame,
or a grid with a clip to a row. `TYPES` names, for each enemy type, the frames of each of its clips, and
this script lays every clip out the same way: one strip per clip in
`src/assets/enemies/<type>/<clip>.png`, every frame of a type the same size,
the body's middle (read off its first idle frame) in the middle of the frame,
and the rows nothing of any frame reaches cut off. Every pack draws its
figures facing right, as the game wants them.

What it prints for each type is what `assets/monster_sheets.rs` and
`enemies/enemy_types.yaml` need: the frame size, each clip's frames, and the
padding and head square of its first idle frame. `SHEETS` are the strips
that are not a body (a shot, a shield, spikes): each is written as it is
drawn, frame after frame.

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
        [(source, f"{width}x{height}+{(first + i) * width}+{row * height}", [])]
        for i in range(count)
    ]


def files(folder, first=1, last=None):
    """One image per frame, in the order of the number ending its name."""
    found = sorted(
        (PACKS / folder).glob("*.png"),
        key=lambda path: int(path.stem.rsplit("_", 1)[-1]),
    )
    found = [p for p in found if first <= int(p.stem.rsplit("_", 1)[-1])]
    if last is not None:
        found = [p for p in found if int(p.stem.rsplit("_", 1)[-1]) <= last]
    return [[(path, None, [])] for path in found]


def moved(frame, dx, *extra):
    """A frame's layers moved `dx` px to the right, with `extra` layers (a
    source of None: drawn by their arguments alone) over them."""
    return [(source, crop, args + ["-roll", f"{dx:+d}+0"]) for source, crop, args in frame] + [
        (None, None, layer) for layer in extra
    ]


def streak(size, start, end, colour="#E4ECF2"):
    """A thin line from `start` to `end` on a clear frame of `size`."""
    return [
        "-size",
        f"{size[0]}x{size[1]}",
        "xc:none",
        "-stroke",
        colour,
        "-draw",
        f"line {start[0]},{start[1]} {end[0]},{end[1]}",
    ]


BOSS = "dark_fantasy_boss/dark fantasy big boss"
WETLANDS = "shadow_series/SHADOW Series - The Shadowed Wetlands Boss v1.1/Sprite Sheet"
HALLOKIN = "shadow_series/SHADOW Series - Hallokin (free)/sprite sheet 9x12 (free).png"
HUNTRESS = "shadow_series/SHADOW Series - The Blind Huntress/Sprite Sheet"
DUELIST = "pixel_art_09/Enemy01"
GUNNER = "pixel_art_09/Enemy02"
MOON = "character_pack/Player"
WISP = "pixel_art_05/PlayerAllAnim"
JADE = "pixel_art_09/Player"
SHADEFLIT = "shadow_series/SHADOW Series - Shadeflit (Free)"
SHIELD = "pixel_art_09/Shield"


def wisp(first, last):
    return files(WISP, first, last)


# Shadeflit has no attack: its thrust is its first standing frame drawn back,
# then thrown forward with a streak running on from its spear's tip (at 87, 86
# of the 277 x 161 cell, the spear rising 12 px over 51).
SHADEFLIT_STANCE = strip(f"{SHADEFLIT}/idle.png", 277, 1)[0]
SHADEFLIT_CELL = (277, 161)
SHADEFLIT_THRUST = [
    SHADEFLIT_STANCE,
    moved(SHADEFLIT_STANCE, -3),
    moved(SHADEFLIT_STANCE, -5),
    moved(
        SHADEFLIT_STANCE,
        10,
        streak(SHADEFLIT_CELL, (98, 86), (126, 79)),
        streak(SHADEFLIT_CELL, (98, 87), (118, 82), "#8E9AA6"),
    ),
    moved(SHADEFLIT_STANCE, 11, streak(SHADEFLIT_CELL, (99, 86), (113, 83))),
    moved(SHADEFLIT_STANCE, 7),
    moved(SHADEFLIT_STANCE, 3),
]

# Every clip of every type: its frames, as a list of layers drawn one over the
# other, each a source image, the cell of it (None for the whole image) and
# what is done to it after.
TYPES = {
    # The dark fantasy big boss: a horned brute that strikes the floor
    # (`attack 2`) and calls spikes up out of it (`SHEETS`).
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
        # One wide cut, on its second frame.
        "combo": strip(f"{HUNTRESS}/11 - attack 3.png", 240),
        "dash_attack": strip(f"{HUNTRESS}/12 - dash attack.png", 240),
        "roll": strip(f"{HUNTRESS}/6 - dash.png", 240),
        "hurt": strip(f"{HUNTRESS}/14 - hit.png", 240),
        "dead": strip(f"{HUNTRESS}/15 - death.png", 240),
    },
    # PackCharacterPixelArt09's swordsman: no death of its own, so it dies
    # staggering (its stun). It stands with its sword trailing on the floor
    # until it notices the player; from then on it holds it out ahead, as its
    # attack ends (frames 16-20).
    "crimson_duelist": {
        "idle": files(f"{DUELIST}/Attack", 16, 20),
        "unarmed_idle": files(f"{DUELIST}/Idle"),
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
    # CharacterPack: a player pack. No flinch and no death: it flinches in
    # its first idle frames and dies falling.
    "moon_blade": {
        "idle": files(f"{MOON}/Idle"),
        "walk": files(f"{MOON}/Run"),
        "jump": files(f"{MOON}/JumpUp"),
        "fall": files(f"{MOON}/JumpDown"),
        "attack": files(f"{MOON}/Attack01"),
        # The pack's Shoot fires on its first frame, so two idle frames come
        # first as its wind-up.
        "shoot": files(f"{MOON}/Idle", last=2) + files(f"{MOON}/Shoot"),
        "hurt": files(f"{MOON}/Idle", last=2),
        "dead": files(f"{MOON}/JumpDown"),
    },
    # PackCharacterPixelArt05: a player pack drawn as one run of 199 frames,
    # its clips the tags of its .aseprite (Idle 1-8, Run 10-17, Dash 19-28,
    # Attack01 30-47, Attack02 48-68, Attack03 69-94, Death 95-128;
    # Attack04, 129-198, is left). No flinch: its first idle frames.
    "antler_wisp": {
        "idle": wisp(1, 8),
        "walk": wisp(10, 17),
        "attack": wisp(30, 47),
        "cast": wisp(48, 68),
        "burst": wisp(69, 94),
        "hurt": wisp(1, 2),
        "dead": wisp(95, 128),
    },
    # PackCharacterPixelArt09's hero: it stands with its sword put away
    # until it notices the player, and takes it out (`draw`) before anything
    # else; then it holds it out ahead, as its attacks end (StopAttack's first
    # frames), rather than lowered. A parry split as the duelist's is.
    "jade_swordsman": {
        "idle": files(f"{JADE}/StopAttack", last=4),
        "unarmed_idle": files(f"{JADE}/Idle"),
        "draw": files(f"{JADE}/TakeSword"),
        "walk": files(f"{JADE}/RunwSword"),
        "attack": files(f"{JADE}/Attack01"),
        "thrust": files(f"{JADE}/Attack02"),
        "sweep": files(f"{JADE}/Attack03"),
        "heavy": files(f"{JADE}/AttackHard"),
        "guard": files(f"{JADE}/Parry", last=5),
        "counter": files(f"{JADE}/Parry", first=6),
        "roll": files(f"{JADE}/RollSword"),
        "hurt": files(f"{JADE}/HitwSword"),
        "dead": files(f"{JADE}/Death"),
    },
    # Shadeflit: a spearman drawn only standing and running. It thrusts (made
    # here, above), and flinches and dies in its turn.
    "shadeflit": {
        "idle": strip(f"{SHADEFLIT}/idle.png", 277),
        "walk": strip(f"{SHADEFLIT}/run.png", 277),
        "attack": SHADEFLIT_THRUST,
        "hurt": strip(f"{SHADEFLIT}/run-turn.png", 277),
        "dead": strip(f"{SHADEFLIT}/run-turn.png", 277),
    },
    # PackCharacterPixelArt09's NPC: a hooded figure drawn only standing,
    # holding a shield, its glyphs flickering. It throws in that same loop;
    # its shot and its armor are the pack's shield (`SHEETS`).
    "hooded_warden": {
        "idle": files("pixel_art_09/NPC01/Idle"),
        "walk": files("pixel_art_09/NPC01/Idle"),
        "attack": files("pixel_art_09/NPC01/Idle"),
        "hurt": files("pixel_art_09/NPC01/Idle", last=1),
        "dead": files("pixel_art_09/NPC01/Idle", last=1),
    },
}


def run(*args):
    return subprocess.run(
        ["magick", *map(str, args)], check=True, capture_output=True, text=True
    ).stdout


def frame(layers):
    """The arguments that draw one frame: its layers, one over the other."""
    args = []
    for source, crop, extra in layers:
        drawn = [source] + (["-crop", crop] if crop else []) + ["+repage"] if source else []
        args += ["(", *drawn, *extra, ")"]
    return args + ["-background", "none", "-flatten"]


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
                trim = run(*frame(layers), "-write", f"PNG32:{target}", "-format", "%@ %w %h", "info:")
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

# The strips that are not a body, each frame drawn as it is and laid after the
# one before.
SPIKES = [f"{BOSS} attack 2{variant}.png" for variant in "acb"]
SHEETS = {
    # The gunner's bullet, as it ships.
    "glyph_gunner/shot.png": [[(PACKS / GUNNER / "Enemy02_Bullet.png", None, [])]],
    # Three of the brute's spikes side by side, the tallest in the middle,
    # in one 104 x 48 frame.
    "horned_brute/spikes.png": [
        [
            (None, None, ["-size", "104x48", "xc:none"]),
            *[
                (PACKS / path, f"48x48+{frame * 48}+0", ["-repage", f"+{offset}+0"])
                for path, offset in zip(SPIKES, (0, 28, 56))
            ],
        ]
        for frame in range(16)
    ],
    # The warden's shot: the shield hovering, as it ships.
    "hooded_warden/shot.png": files(f"{SHIELD}/Idle"),
    # Its armor: the shield, then its break.
    "hooded_warden/shield.png": files(f"{SHIELD}/Idle", last=1) + files(f"{SHIELD}/Break"),
}
for target, frames in SHEETS.items():
    args = [arg for layers in frames for arg in ["(", *frame(layers), ")"]]
    run(*args, "+append", "-strip", f"PNG32:{ASSETS / 'enemies' / target}")
    print(f"{target}: {len(frames)} frames")
