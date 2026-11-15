use super::super::{SpriteSheetSpec, UVec2};

/// Loose effect sheets: pixel art that is drawn beside a body rather than
/// being one, so it belongs to no character's clip list. They are cut from
/// the same packs as the player's sheets (`player/LICENSE.txt`), but for the
/// three a landed blow plays, which come from the pack of
/// `effects/LICENSE.txt`.
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
        ("effect_hit_spark", "effects/hit_spark.png", 32, 6, 30.0),
        // The sci-fi pack's explosion (`packs/sci_fi/`): how its robots die.
        (
            "effect_sci_fi_explosion",
            "effects/sci_fi/explosion.png",
            48,
            9,
            18.0,
        ),
        ("effect_hit_burst", "effects/hit_burst.png", 32, 5, 26.0),
        ("effect_kill_ring", "effects/kill_ring.png", 32, 6, 24.0),
        (
            "effect_light_charge",
            "effects/light_charge.png",
            64,
            4,
            14.0,
        ),
    ]
    .into_iter()
    .map(|(key, path, size, frames, fps)| (key, path, UVec2::splat(size), frames, fps))
    // Drawn white in Typst (`art/effects/`, `art/ui/mark.typ`), for the
    // game to tint in the colour of the bonus each one tells.
    .chain([
        (
            "effect_tell_ring",
            "effects/tell_ring.png",
            UVec2::splat(32),
            6,
            22.0,
        ),
        (
            "effect_air_ring",
            "effects/air_ring.png",
            UVec2::new(32, 12),
            6,
            24.0,
        ),
        ("tell_mark", "ui/mark.png", UVec2::splat(13), 1, 1.0),
        ("tell_alert", "ui/alert.png", UVec2::new(5, 11), 1, 1.0),
        ("tell_unaware", "ui/unaware.png", UVec2::new(7, 11), 1, 1.0),
        // The sci-fi pack's puff of dust, kicked up by a jump High Jump lifts.
        (
            "effect_dust",
            "effects/sci_fi/dust.png",
            UVec2::splat(16),
            5,
            20.0,
        ),
    ])
    .map(|(key, path, size, frames, fps)| SpriteSheetSpec {
        key: key.into(),
        path: path.into(),
        frame_width: size.x,
        frame_height: size.y,
        frames,
        fps,
        first_cell: UVec2::ZERO,
    })
    .collect()
}
