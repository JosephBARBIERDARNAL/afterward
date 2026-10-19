use super::super::SpriteSheetSpec;

pub(super) fn sprites() -> Vec<SpriteSheetSpec> {
    vec![
        SpriteSheetSpec {
            key: "boss_idle",
            path: "enemies/boss1/Right/Png/WarriorRightIdle.png",
            frame_width: 48,
            frame_height: 48,
            frames: 5,
            fps: 4.0,
        },
        SpriteSheetSpec {
            key: "boss_walk",
            path: "enemies/boss1/Right/Png/WarriorRightWalk.png",
            frame_width: 48,
            frame_height: 48,
            frames: 8,
            fps: 8.0,
        },
        SpriteSheetSpec {
            key: "boss_attack",
            path: "enemies/boss1/Right/Png/WarriorRightAttack01.png",
            frame_width: 48,
            frame_height: 48,
            frames: 6,
            fps: 12.0,
        },
        SpriteSheetSpec {
            key: "boss_attack_2",
            path: "enemies/boss1/Right/Png/WarriorRightAttack02.png",
            frame_width: 48,
            frame_height: 48,
            frames: 6,
            fps: 12.0,
        },
        SpriteSheetSpec {
            key: "boss_attack_3",
            path: "enemies/boss1/Right/Png/WarriorRightAttack03.png",
            frame_width: 48,
            frame_height: 48,
            frames: 5,
            fps: 12.0,
        },
        SpriteSheetSpec {
            key: "boss_jump",
            path: "enemies/boss1/Right/Png/WarriorRightJump.png",
            frame_width: 48,
            frame_height: 48,
            frames: 5,
            fps: 8.0,
        },
        SpriteSheetSpec {
            key: "boss_land",
            path: "enemies/boss1/Right/Png/WarriorRightLand.png",
            frame_width: 48,
            frame_height: 48,
            frames: 5,
            fps: 10.0,
        },
        SpriteSheetSpec {
            key: "boss_hurt",
            path: "enemies/boss1/Right/Png/WarriorRightHurt.png",
            frame_width: 48,
            frame_height: 48,
            frames: 4,
            fps: 12.0,
        },
        SpriteSheetSpec {
            key: "boss_dead",
            path: "enemies/boss1/Right/Png/WarriorRightDeath.png",
            frame_width: 48,
            frame_height: 48,
            frames: 5,
            fps: 8.0,
        },
    ]
}
