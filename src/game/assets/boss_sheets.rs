use super::super::{SpriteSheetSpec, UVec2};

pub(super) fn sprites() -> Vec<SpriteSheetSpec> {
    // The Black Knight's clips, cut out of the pack's atlas by
    // `scripts/slice_black_knight.py` into one strip each: its name, its
    // frames, and the pace it plays at. Each is the sheet `black_knight_<clip>`,
    // which is how `set_enemy_animation` finds the clip of an action and how
    // an attack of `enemies/enemy_types.yaml` names the one it plays. An
    // attack's clip is not played at this pace until its strike has landed:
    // the frames before its `strike_frame` are spread over its wind-up. The
    // roll is spread over its own length the same way.
    let black_knight = [
        ("unarmed_idle", 6, 8.0),
        ("draw", 18, 18.0),
        ("idle", 6, 8.0),
        ("walk", 12, 14.0),
        ("jump", 7, 12.0),
        ("fall", 6, 10.0),
        ("land", 6, 14.0),
        ("combo", 21, 16.0),
        ("lunge", 13, 16.0),
        ("guard", 6, 12.0),
        ("counter", 15, 16.0),
        ("guard_miss", 3, 10.0),
        ("roll", 12, 20.0),
        ("buff", 12, 14.0),
        ("hurt", 5, 14.0),
        ("dead", 9, 10.0),
    ]
    .map(|(clip, frames, fps)| SpriteSheetSpec {
        key: format!("black_knight_{clip}"),
        path: format!("enemies/black_knight/{clip}.png"),
        frame_width: 150,
        frame_height: 62,
        frames,
        fps,
        first_cell: UVec2::ZERO,
    });
    let mut sheets = vec![
        SpriteSheetSpec {
            key: "boss_idle".into(),
            path: "enemies/boss1/Idle.png".into(),
            frame_width: 250,
            frame_height: 250,
            frames: 8,
            fps: 8.0,
            first_cell: UVec2::ZERO,
        },
        SpriteSheetSpec {
            key: "boss_walk".into(),
            path: "enemies/boss1/Run.png".into(),
            frame_width: 250,
            frame_height: 250,
            frames: 8,
            fps: 10.0,
            first_cell: UVec2::ZERO,
        },
        SpriteSheetSpec {
            key: "boss_attack".into(),
            path: "enemies/boss1/Attack_1.png".into(),
            frame_width: 250,
            frame_height: 250,
            frames: 8,
            fps: 14.0,
            first_cell: UVec2::ZERO,
        },
        SpriteSheetSpec {
            key: "boss_attack_2".into(),
            path: "enemies/boss1/Attack_2.png".into(),
            frame_width: 250,
            frame_height: 250,
            frames: 8,
            fps: 12.0,
            first_cell: UVec2::ZERO,
        },
        SpriteSheetSpec {
            key: "boss_jump".into(),
            path: "enemies/boss1/Jump.png".into(),
            frame_width: 250,
            frame_height: 250,
            frames: 2,
            fps: 8.0,
            first_cell: UVec2::ZERO,
        },
        SpriteSheetSpec {
            key: "boss_fall".into(),
            path: "enemies/boss1/Fall.png".into(),
            frame_width: 250,
            frame_height: 250,
            frames: 2,
            fps: 8.0,
            first_cell: UVec2::ZERO,
        },
        SpriteSheetSpec {
            key: "boss_hurt".into(),
            path: "enemies/boss1/Hurt.png".into(),
            frame_width: 250,
            frame_height: 250,
            frames: 3,
            fps: 12.0,
            first_cell: UVec2::ZERO,
        },
        SpriteSheetSpec {
            key: "boss_dead".into(),
            path: "enemies/boss1/Dead.png".into(),
            frame_width: 250,
            frame_height: 250,
            frames: 7,
            fps: 9.0,
            first_cell: UVec2::ZERO,
        },
    ];
    sheets.extend(black_knight);
    sheets
}
