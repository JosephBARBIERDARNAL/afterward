//! Sprite-sheet animation shared by the player, enemies, and the boss.

use super::*;

#[derive(Resource, Default)]
pub(crate) struct SpriteSheets(pub(crate) BTreeMap<String, SpriteSheet>);

#[derive(Clone)]
pub(crate) struct SpriteSheet {
    pub(crate) image: Handle<Image>,
    pub(crate) layout: Handle<TextureAtlasLayout>,
    pub(crate) frame_size: Vec2,
    pub(crate) frames: usize,
    pub(crate) fps: f32,
}

#[derive(Component)]
pub(crate) struct AnimatedSprite {
    pub(crate) sheet: String,
    pub(crate) frame: usize,
    pub(crate) frames: usize,
    pub(crate) timer: Timer,
    pub(crate) looping: bool,
}

#[derive(Debug)]
pub(crate) struct SpriteSheetSpec {
    pub(crate) key: String,
    pub(crate) path: String,
    pub(crate) frame_width: u32,
    pub(crate) frame_height: u32,
    pub(crate) frames: usize,
    pub(crate) fps: f32,
    /// Grid cell (column, row) of the clip's first frame within its sheet.
    pub(crate) first_cell: UVec2,
}

pub(crate) fn set_animation(
    sprite: &mut Sprite,
    animation: &mut AnimatedSprite,
    sheets: &SpriteSheets,
    key: &str,
    looping: bool,
) {
    if animation.sheet == key {
        return;
    }
    let Some(sheet) = sheets.0.get(key) else {
        return;
    };
    sprite.image = sheet.image.clone();
    sprite.texture_atlas = Some(TextureAtlas {
        layout: sheet.layout.clone(),
        index: 0,
    });
    animation.sheet = key.to_string();
    animation.frame = 0;
    animation.frames = sheet.frames;
    animation.looping = looping;
    animation.timer = Timer::from_seconds(1.0 / sheet.fps, TimerMode::Repeating);
}

impl EnemyTypeId {
    pub(crate) fn idle_sheet(&self) -> &'static str {
        match &**self {
            "blade_unit" => "enemy_swordsman_idle",
            "rifle_unit" => "enemy_infantry_idle",
            "heavy_unit" => "enemy_destroyer_idle",
            "dark_blade_unit" => "enemy_dark_swordsman_idle",
            "dark_rifle_unit" => "enemy_dark_infantry_idle",
            "dark_heavy_unit" => "enemy_dark_destroyer_idle",
            "stalker_unit" => "enemy_gorgon_1_idle",
            "lancer_unit" => "enemy_gorgon_2_idle",
            "breaker_unit" => "enemy_gorgon_3_idle",
            "scout_unit" => "enemy_dark_ninja_idle",
            "emitter_unit" => "boss_idle",
            _ => "enemy_swordsman_idle",
        }
    }
}
