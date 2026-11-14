use super::super::{SpriteSheetSpec, UVec2};

/// The monsters: sheets named `<type>_<clip>`, which is how
/// `set_enemy_animation` finds the clip of an action and how an attack of
/// `enemies/enemy_types.yaml` names the one it plays. All of them are drawn
/// facing right.
pub(super) fn sprites() -> Vec<SpriteSheetSpec> {
    // The horror pack (`enemies/horror/`, its ReadMe beside them) ships one
    // row of frames per monster, every clip after the other: each clip here
    // is its name, the cell it starts on, its frames and its pace. The
    // abomination's frames are the pack's 112 px widened by 64 px on the
    // left, so its body stands in the middle and it turns around on the spot.
    // Left uncut: the cat's stare, stand up, turn and sit down (cells 0-2 and
    // 7-13).
    let horror = [
        (
            "abomination",
            UVec2::new(176, 48),
            [
                ("idle", 0, 4, 6.0),
                ("walk", 4, 4, 7.0),
                ("attack", 8, 10, 12.0),
                ("hurt", 18, 2, 10.0),
                ("dead", 20, 9, 10.0),
            ],
        ),
        (
            "mage",
            UVec2::new(112, 48),
            [
                ("idle", 0, 4, 6.0),
                ("walk", 4, 4, 7.0),
                ("attack", 8, 10, 12.0),
                ("hurt", 18, 2, 10.0),
                ("dead", 20, 9, 10.0),
            ],
        ),
        (
            "catto",
            UVec2::new(48, 32),
            [
                ("idle", 14, 4, 5.0),
                ("walk", 3, 4, 10.0),
                ("attack", 18, 6, 12.0),
                ("hurt", 24, 2, 10.0),
                ("dead", 26, 9, 10.0),
            ],
        ),
        // Its attack is three of the pack's clips on end: the claws drawn,
        // the slash, the claws put away.
        (
            "mad_ghost",
            UVec2::new(64, 64),
            [
                ("idle", 0, 4, 6.0),
                ("walk", 4, 4, 8.0),
                ("attack", 8, 12, 12.0),
                ("hurt", 20, 2, 10.0),
                ("dead", 22, 9, 10.0),
            ],
        ),
    ]
    .into_iter()
    .flat_map(|(monster, frame, clips)| {
        // Each is drawn four times over: as the pack ships it, and in the
        // three palettes scripts/recolour_monsters.py writes beside it, which
        // are the types `<monster>_rust`, `<monster>_gold` and
        // `<monster>_purple`.
        [
            ("", ""),
            ("_rust", "rust/"),
            ("_gold", "gold/"),
            ("_purple", "purple/"),
        ]
        .into_iter()
        .flat_map(move |(suffix, folder)| {
            clips
                .into_iter()
                .map(move |(clip, first, frames, fps)| SpriteSheetSpec {
                    key: format!("{monster}{suffix}_{clip}"),
                    path: format!("enemies/horror/{folder}{monster}.png"),
                    frame_width: frame.x,
                    frame_height: frame.y,
                    frames,
                    fps,
                    first_cell: UVec2::new(first, 0),
                })
        })
    });
    // The eye beast (`enemies/eye_beast/`) ships one image per frame, looking
    // left: each clip is those frames mirrored and laid in a row, the body's
    // widened by 13 px on the left so the eye sits in the middle. `shot` is
    // the void ball it fires.
    let eye_beast = [
        ("idle", 73, 20, 10.0),
        ("walk", 73, 13, 12.0),
        ("attack", 73, 13, 12.0),
        ("hurt", 73, 4, 12.0),
        ("dead", 73, 15, 12.0),
        ("shot", 60, 3, 10.0),
    ]
    .map(|(clip, frame_width, frames, fps)| SpriteSheetSpec {
        key: format!("eye_beast_{clip}"),
        path: format!("enemies/eye_beast/{clip}.png"),
        frame_width,
        frame_height: 57,
        frames,
        fps,
        first_cell: UVec2::ZERO,
    });
    // The sci-fi pack's robots (`enemies/sci_fi/`, cut by `just sci-fi`): one
    // strip of 48 px frames per clip, and a 16 px shot. They have no hurt or
    // death clips of their own: a hurt one shows its first frames, a dead one
    // its first while the pack's explosion goes off over it.
    let sci_fi = [
        (
            "sci_fi_walker",
            "idle",
            "enemies/sci_fi/walker_idle.png",
            48,
            5,
            8.0,
        ),
        (
            "sci_fi_walker",
            "walk",
            "enemies/sci_fi/walker_walk.png",
            48,
            6,
            10.0,
        ),
        (
            "sci_fi_walker",
            "attack",
            "enemies/sci_fi/walker_shoot.png",
            48,
            6,
            12.0,
        ),
        (
            "sci_fi_walker",
            "hurt",
            "enemies/sci_fi/walker_idle.png",
            48,
            2,
            10.0,
        ),
        (
            "sci_fi_walker",
            "dead",
            "enemies/sci_fi/walker_idle.png",
            48,
            1,
            1.0,
        ),
        (
            "sci_fi_walker",
            "shot",
            "effects/sci_fi/shot_blue.png",
            16,
            4,
            12.0,
        ),
        (
            "sci_fi_orb",
            "idle",
            "enemies/sci_fi/orb_idle.png",
            48,
            5,
            10.0,
        ),
        (
            "sci_fi_orb",
            "walk",
            "enemies/sci_fi/orb_idle.png",
            48,
            5,
            14.0,
        ),
        (
            "sci_fi_orb",
            "hurt",
            "enemies/sci_fi/orb_idle.png",
            48,
            2,
            10.0,
        ),
        (
            "sci_fi_orb",
            "dead",
            "enemies/sci_fi/orb_idle.png",
            48,
            1,
            1.0,
        ),
        (
            "sci_fi_drone",
            "idle",
            "enemies/sci_fi/drone_fly.png",
            48,
            4,
            12.0,
        ),
        (
            "sci_fi_drone",
            "walk",
            "enemies/sci_fi/drone_fly.png",
            48,
            4,
            14.0,
        ),
        (
            "sci_fi_drone",
            "attack",
            "enemies/sci_fi/drone_shoot.png",
            48,
            4,
            10.0,
        ),
        (
            "sci_fi_drone",
            "hurt",
            "enemies/sci_fi/drone_fly.png",
            48,
            2,
            10.0,
        ),
        (
            "sci_fi_drone",
            "dead",
            "enemies/sci_fi/drone_fly.png",
            48,
            1,
            1.0,
        ),
        (
            "sci_fi_drone",
            "shot",
            "effects/sci_fi/ball_blue.png",
            16,
            4,
            12.0,
        ),
    ]
    .map(|(robot, clip, path, size, frames, fps)| SpriteSheetSpec {
        key: format!("{robot}_{clip}"),
        path: path.into(),
        frame_width: size,
        frame_height: size,
        frames,
        fps,
        first_cell: UVec2::ZERO,
    });
    horror.chain(eye_beast).chain(sci_fi).collect()
}
