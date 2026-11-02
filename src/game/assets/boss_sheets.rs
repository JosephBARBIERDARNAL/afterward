use super::super::SpriteSheetSpec;

pub(super) fn sprites() -> Vec<SpriteSheetSpec> {
    vec![
        SpriteSheetSpec {
            key: "boss_idle",
            path: "enemies/boss1/Idle.png",
            frame_width: 250,
            frame_height: 250,
            frames: 8,
            fps: 8.0,
        },
        SpriteSheetSpec {
            key: "boss_walk",
            path: "enemies/boss1/Run.png",
            frame_width: 250,
            frame_height: 250,
            frames: 8,
            fps: 10.0,
        },
        SpriteSheetSpec {
            key: "boss_attack",
            path: "enemies/boss1/Attack_1.png",
            frame_width: 250,
            frame_height: 250,
            frames: 8,
            fps: 14.0,
        },
        SpriteSheetSpec {
            key: "boss_attack_2",
            path: "enemies/boss1/Attack_2.png",
            frame_width: 250,
            frame_height: 250,
            frames: 8,
            fps: 12.0,
        },
        SpriteSheetSpec {
            key: "boss_jump",
            path: "enemies/boss1/Jump.png",
            frame_width: 250,
            frame_height: 250,
            frames: 2,
            fps: 8.0,
        },
        SpriteSheetSpec {
            key: "boss_fall",
            path: "enemies/boss1/Fall.png",
            frame_width: 250,
            frame_height: 250,
            frames: 2,
            fps: 8.0,
        },
        SpriteSheetSpec {
            key: "boss_hurt",
            path: "enemies/boss1/Hurt.png",
            frame_width: 250,
            frame_height: 250,
            frames: 3,
            fps: 12.0,
        },
        SpriteSheetSpec {
            key: "boss_dead",
            path: "enemies/boss1/Dead.png",
            frame_width: 250,
            frame_height: 250,
            frames: 7,
            fps: 9.0,
        },
    ]
}
