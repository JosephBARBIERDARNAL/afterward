use super::super::{SpriteSheetSpec, UVec2};

/// Loose effect sheets: pixel art that is drawn beside a body rather than
/// being one, so it belongs to no character's clip list. They are cut from
/// the same packs as the player's sheets (`player/LICENSE.txt`).
pub(super) fn sprites() -> Vec<SpriteSheetSpec> {
    [
        ("effect_portal", "effects/portal.png", 128, 17, 24.0),
        (
            "effect_health_regeneration",
            "effects/health_regeneration.png",
            64,
            6,
            12.0,
        ),
        (
            "effect_light_charge",
            "effects/light_charge.png",
            64,
            4,
            14.0,
        ),
    ]
    .into_iter()
    .map(|(key, path, size, frames, fps)| SpriteSheetSpec {
        key: key.into(),
        path: path.into(),
        frame_width: size,
        frame_height: size,
        frames,
        fps,
        first_cell: UVec2::ZERO,
    })
    .collect()
}
