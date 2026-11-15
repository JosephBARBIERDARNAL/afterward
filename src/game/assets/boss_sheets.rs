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
    [
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
    })
    .into()
}
