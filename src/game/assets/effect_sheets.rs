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
    .map(|(key, path, size, frames, fps)| {
        (
            key.to_owned(),
            path.to_owned(),
            UVec2::splat(size),
            frames,
            fps,
        )
    })
    // Marked Prey's reticle, drawn white in Typst (`art/ui/mark.typ`) for the
    // game to tint.
    .chain([(
        "tell_mark".to_owned(),
        "ui/mark.png".to_owned(),
        UVec2::splat(13),
        1,
        1.0,
    )])
    // What the bonuses show themselves with, cut out of the VFX packs by
    // `scripts/cut_vfx.py` (`just vfx`): name, frame size, frames, pace.
    .chain(
        [
            ("arcane_charge", UVec2::splat(48), 18, 15.0),
            ("shield_break", UVec2::splat(48), 14, 24.0),
            ("shield_hit", UVec2::splat(44), 12, 24.0),
            ("slash", UVec2::new(64, 32), 8, 30.0),
            ("life_orb", UVec2::splat(16), 18, 15.0),
            ("skull", UVec2::splat(28), 10, 15.0),
            ("alert", UVec2::splat(28), 10, 20.0),
            ("unaware", UVec2::new(48, 64), 18, 10.0),
            ("meteor_impact", UVec2::new(68, 62), 21, 24.0),
            ("jump_dust", UVec2::new(44, 24), 14, 28.0),
            ("charge_flare", UVec2::splat(56), 7, 18.0),
            ("shock_sparks", UVec2::new(44, 50), 18, 15.0),
            ("pierce", UVec2::new(48, 32), 4, 18.0),
            ("glint", UVec2::splat(16), 12, 18.0),
            ("heal_burst", UVec2::new(44, 76), 18, 24.0),
            ("whirlpool", UVec2::new(72, 48), 20, 28.0),
            ("air_burst", UVec2::new(120, 52), 10, 28.0),
        ]
        .map(|(name, size, frames, fps)| {
            (
                format!("vfx_{name}"),
                format!("effects/vfx/{name}.png"),
                size,
                frames,
                fps,
            )
        }),
    )
    .map(|(key, path, size, frames, fps)| SpriteSheetSpec {
        key,
        path,
        frame_width: size.x,
        frame_height: size.y,
        frames,
        fps,
        first_cell: UVec2::ZERO,
    })
    .collect()
}
