use super::super::{SpriteSheetSpec, UVec2};

pub(super) fn sprites() -> Vec<SpriteSheetSpec> {
    let mut sheets = vec![
        // The shield above a metal enemy's health bar: frame 0 holds while it
        // is up, frames 1-5 are the break (art/ui/enemy_shield.typ).
        SpriteSheetSpec {
            key: "enemy_shield".into(),
            path: "ui/enemy_shield.png".into(),
            frame_width: 21,
            frame_height: 16,
            frames: 6,
            fps: 30.0,
            first_cell: UVec2::ZERO,
        },
        SpriteSheetSpec {
            key: "enemy_swordsman_idle".into(),
            path: "enemies/type1/Swordsman/Idle.png".into(),
            frame_width: 128,
            frame_height: 128,
            frames: 5,
            fps: 5.0,
            first_cell: UVec2::ZERO,
        },
        SpriteSheetSpec {
            key: "enemy_swordsman_attack".into(),
            path: "enemies/type1/Swordsman/Attack_1.png".into(),
            frame_width: 128,
            frame_height: 128,
            frames: 4,
            fps: 12.0,
            first_cell: UVec2::ZERO,
        },
        SpriteSheetSpec {
            key: "enemy_swordsman_hurt".into(),
            path: "enemies/type1/Swordsman/Hurt.png".into(),
            frame_width: 128,
            frame_height: 128,
            frames: 3,
            fps: 12.0,
            first_cell: UVec2::ZERO,
        },
        SpriteSheetSpec {
            key: "enemy_swordsman_dead".into(),
            path: "enemies/type1/Swordsman/Dead.png".into(),
            frame_width: 128,
            frame_height: 128,
            frames: 4,
            fps: 8.0,
            first_cell: UVec2::ZERO,
        },
        SpriteSheetSpec {
            key: "enemy_infantry_idle".into(),
            path: "enemies/type1/Infantryman/Idle.png".into(),
            frame_width: 128,
            frame_height: 128,
            frames: 6,
            fps: 5.0,
            first_cell: UVec2::ZERO,
        },
        SpriteSheetSpec {
            key: "enemy_infantry_walk".into(),
            path: "enemies/type1/Infantryman/Walk.png".into(),
            frame_width: 128,
            frame_height: 128,
            frames: 6,
            fps: 8.0,
            first_cell: UVec2::ZERO,
        },
        SpriteSheetSpec {
            key: "enemy_infantry_attack".into(),
            path: "enemies/type1/Infantryman/Shot_1.png".into(),
            frame_width: 128,
            frame_height: 128,
            frames: 11,
            fps: 12.0,
            first_cell: UVec2::ZERO,
        },
        SpriteSheetSpec {
            key: "enemy_infantry_hurt".into(),
            path: "enemies/type1/Infantryman/Hurt.png".into(),
            frame_width: 128,
            frame_height: 128,
            frames: 4,
            fps: 12.0,
            first_cell: UVec2::ZERO,
        },
        SpriteSheetSpec {
            key: "enemy_infantry_dead".into(),
            path: "enemies/type1/Infantryman/Dead.png".into(),
            frame_width: 128,
            frame_height: 128,
            frames: 5,
            fps: 8.0,
            first_cell: UVec2::ZERO,
        },
        SpriteSheetSpec {
            key: "enemy_destroyer_idle".into(),
            path: "enemies/type1/Destroyer/Idle.png".into(),
            frame_width: 128,
            frame_height: 128,
            frames: 5,
            fps: 5.0,
            first_cell: UVec2::ZERO,
        },
        SpriteSheetSpec {
            key: "enemy_destroyer_walk".into(),
            path: "enemies/type1/Destroyer/Walk.png".into(),
            frame_width: 128,
            frame_height: 128,
            frames: 8,
            fps: 7.0,
            first_cell: UVec2::ZERO,
        },
        SpriteSheetSpec {
            key: "enemy_destroyer_attack".into(),
            path: "enemies/type1/Destroyer/Shot_1.png".into(),
            frame_width: 128,
            frame_height: 128,
            frames: 8,
            fps: 10.0,
            first_cell: UVec2::ZERO,
        },
        SpriteSheetSpec {
            key: "enemy_destroyer_hurt".into(),
            path: "enemies/type1/Destroyer/Hurt.png".into(),
            frame_width: 128,
            frame_height: 128,
            frames: 3,
            fps: 12.0,
            first_cell: UVec2::ZERO,
        },
        SpriteSheetSpec {
            key: "enemy_destroyer_dead".into(),
            path: "enemies/type1/Destroyer/Dead.png".into(),
            frame_width: 128,
            frame_height: 128,
            frames: 7,
            fps: 8.0,
            first_cell: UVec2::ZERO,
        },
    ];
    // The dark robots play the ordinary robots' clips, recoloured frame for
    // frame by scripts/darken_robots.py into `enemies/type1_dark/`, so each of
    // their sheets is its ordinary one under a `dark_` key.
    let dark_robots = sheets
        .iter()
        .filter(|sheet| sheet.path.starts_with("enemies/type1/"))
        .map(|sheet| SpriteSheetSpec {
            key: sheet.key.replacen("enemy_", "enemy_dark_", 1),
            path: sheet.path.replacen("type1", "type1_dark", 1),
            ..*sheet
        })
        .collect::<Vec<_>>();
    sheets.extend(dark_robots);
    sheets
}
