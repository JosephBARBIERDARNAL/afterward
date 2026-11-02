use super::super::SpriteSheetSpec;

pub(super) fn sprites() -> Vec<SpriteSheetSpec> {
    vec![SpriteSheetSpec {
        key: "effect_gun_hit",
        path: "explosion/gun1/spritesheet.png",
        frame_width: 48,
        frame_height: 48,
        frames: 7,
        fps: 20.0,
    }]
}
