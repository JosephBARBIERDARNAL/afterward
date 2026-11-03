mod models;
mod player_animations;
pub use models::*;
pub use player_animations::*;
use std::{
    collections::BTreeMap,
    error::Error,
    fmt::{self, Display, Formatter},
    fs,
    path::Path,
};

use bevy::prelude::Resource;
use serde::de::DeserializeOwned;

#[derive(Resource, Debug)]
pub struct GameData {
    pub gameplay: GameplayConfig,
    pub player_animations: PlayerAnimations,
    pub player_model: PlayerModelConfig,
    pub enemies: EnemyTypes,
    pub backgrounds: BackgroundProfiles,
    pub bonuses: BonusesConfig,
    pub rooms: BTreeMap<u16, RoomConfig>,
}

impl GameData {
    pub fn load(asset_root: &Path) -> Result<Self, Box<dyn Error + Send + Sync>> {
        let gameplay = read_yaml(&asset_root.join("config/gameplay.yaml"))?;
        let player_animations = read_yaml(&asset_root.join("player/animations.yaml"))?;
        let player_model = read_yaml(&asset_root.join("player/model.yaml"))?;
        let enemies = read_yaml(&asset_root.join("enemies/enemy_types.yaml"))?;
        let backgrounds = read_yaml(&asset_root.join("background/background_profiles.yaml"))?;
        let bonuses = read_yaml(&asset_root.join("config/bonuses.yaml"))?;
        let mut rooms = BTreeMap::new();

        for room_id in 1..=10 {
            let path = asset_root.join(format!("rooms/room_{room_id:03}.yaml"));
            let room: RoomConfig = read_yaml(&path)?;
            rooms.insert(room.room_id, room);
        }

        let data = Self {
            gameplay,
            player_animations,
            player_model,
            enemies,
            backgrounds,
            bonuses,
            rooms,
        };
        data.validate(asset_root)?;
        Ok(data)
    }

    pub(crate) fn bonus_effect(&self, bonus_id: &str, effect_id: &str) -> f32 {
        self.bonuses
            .bonuses
            .iter()
            .find(|bonus| bonus.id == bonus_id)
            .and_then(|bonus| bonus.effects.get(effect_id))
            .and_then(|value| BonusEffect::try_from(value).ok())
            .map(f32::from)
            .unwrap_or_default()
    }

    fn validate(&self, asset_root: &Path) -> Result<(), InvalidGameData> {
        if self.gameplay.viewport.height_h != 9.0
            || self.gameplay.viewport.aspect_ratio.width != 16
            || self.gameplay.viewport.aspect_ratio.height != 9
        {
            return Err(InvalidGameData::from(
                "the gameplay viewport must be fixed at 16:9 and 9 H",
            ));
        }
        let player = &self.gameplay.player;
        if !player.max_speed_h_s.is_finite()
            || player.max_speed_h_s <= 0.0
            || !player.air_max_speed_h_s.is_finite()
            || player.air_max_speed_h_s <= 0.0
            || !player.sword.range_h.is_finite()
            || player.sword.range_h <= 0.0
        {
            return Err(InvalidGameData::from(
                "player movement speeds and sword range must be finite and positive",
            ));
        }
        let sword = &player.sword;
        if !(sword.attack_interval_s.is_finite() && sword.attack_interval_s > 0.0)
            || sword.combo.damage_multipliers.is_empty()
            || sword
                .combo
                .damage_multipliers
                .iter()
                .any(|multiplier| !multiplier.is_finite() || *multiplier <= 0.0)
        {
            return Err(InvalidGameData::from(
                "the sword needs a positive attack interval and at least one positive \
                 combo damage multiplier",
            ));
        }
        let animations = &self.player_animations;
        if animations.body_height_px <= 0.0
            || !(animations.body_height_h.is_finite() && animations.body_height_h > 0.0)
            || !(animations.body_height_px..=animations.frame_size_px.height as f32)
                .contains(&animations.feet_row_px)
            || !(animations.run_speed_fraction > 0.0 && animations.run_speed_fraction <= 1.0)
        {
            return Err(InvalidGameData::from(
                "player sprite body heights, feet row, and run speed fraction are out of range",
            ));
        }
        if animations.sword_combo.len() != sword.combo.damage_multipliers.len() {
            return Err(InvalidGameData::from(
                "the player needs exactly one sword-combo animation per combo damage multiplier",
            ));
        }
        let model = &self.player_model;
        if model.enabled {
            if !asset_root.join(&model.scene).is_file() {
                return Err(InvalidGameData::from(format!(
                    "missing player model: {}",
                    model.scene
                )));
            }
            if model.render_size_px == 0
                || [model.model_height, model.view_height, model.blend_s]
                    .iter()
                    .any(|value| !value.is_finite() || *value <= 0.0)
                || !(0.0..model.view_height).contains(&model.feet_margin)
            {
                return Err(InvalidGameData::from(
                    "player model sizes and blend time must be positive, \
                     with the feet margin inside the view",
                ));
            }
            if let Some((key, _)) = animations.keyed().into_iter().find(|(key, _)| {
                key.strip_prefix("player_")
                    .is_none_or(|name| !model.clips.contains_key(name))
            }) {
                return Err(InvalidGameData::from(format!(
                    "the player model maps no animation to {key}"
                )));
            }
        }
        for (key, clip) in animations.keyed() {
            if clip.frames == 0 || !clip.fps.is_finite() || clip.fps <= 0.0 {
                return Err(InvalidGameData::from(format!(
                    "{key} needs at least one frame and a positive frame rate"
                )));
            }
            if !asset_root.join(&clip.sheet).is_file() {
                return Err(InvalidGameData::from(format!(
                    "missing player sprite sheet: {}",
                    clip.sheet
                )));
            }
        }
        let expected_jump_impulse =
            (2.0 * self.gameplay.player.gravity_h_s2 * self.gameplay.player.jump_height_h).sqrt();
        if (expected_jump_impulse - self.gameplay.player.jump_impulse_h_s).abs() > 0.05 {
            return Err(InvalidGameData::from(
                "jump height, gravity, and jump impulse must describe the same jump",
            ));
        }
        if self.gameplay.room_flow.clear_animation_s <= 0.0
            || self.gameplay.room_flow.bonus_reroll_animation_s <= 0.0
            || self.gameplay.room_flow.boss_intro_duration_s <= 0.0
            || self.gameplay.room_flow.room_transition_duration_s <= 0.0
            || self.gameplay.room_flow.exit_side != "right"
        {
            return Err(InvalidGameData::from(
                "Chapter 1 room flow requires a positive clear animation and right-side exits",
            ));
        }
        let audio = &self.gameplay.audio;
        if audio.footstep_interval_s <= 0.0
            || !(0.0..=1.0).contains(&audio.paused_music_volume_multiplier)
            || !(0.0..=1.0).contains(&audio.room_clear_music_volume_multiplier)
            || !(0.0..=1.0).contains(&audio.chapter_one_music_gain)
            || !(0.0..=1.0).contains(&audio.chapter_one_ambience_gain)
        {
            return Err(InvalidGameData::from(
                "audio tuning values are out of range",
            ));
        }
        let controls = &self.gameplay.controls;
        let default_bindings = [
            &controls.move_left,
            &controls.move_right,
            &controls.navigate_up,
            &controls.navigate_down,
            &controls.jump,
            &controls.dash,
            &controls.sword,
            &controls.interact,
            &controls.pause,
            &controls.toggle_fullscreen,
        ];
        if default_bindings.iter().any(|binding| binding.is_empty()) {
            return Err(InvalidGameData::from(
                "each game action needs a default key binding",
            ));
        }

        let jumping = &self.enemies.jumping;
        if !jumping.air_speed_h_s.is_finite()
            || jumping.air_speed_h_s <= 0.0
            || !jumping.cooldown_s.is_finite()
            || jumping.cooldown_s < 0.0
            || !jumping.level_change_threshold_h.is_finite()
            || jumping.level_change_threshold_h <= 0.0
        {
            return Err(InvalidGameData::from(
                "enemy jumping tuning values must be finite and positive",
            ));
        }
        for enemy in self.enemies.types.values() {
            let jump_height_h = enemy.jump_height_h.unwrap_or(jumping.jump_height_h);
            if !jump_height_h.is_finite() || jump_height_h <= 0.0 {
                return Err(InvalidGameData::from(
                    "enemy jump heights must be finite and positive",
                ));
            }
            if enemy.procedural_movement.as_ref().is_some_and(|movement| {
                !movement.bob_height_h.is_finite()
                    || movement.bob_height_h < 0.0
                    || !movement.cycles_per_h.is_finite()
                    || movement.cycles_per_h <= 0.0
                    || !movement.lean_degrees.is_finite()
            }) {
                return Err(InvalidGameData::from(
                    "procedural enemy movement values must be finite and nonnegative",
                ));
            }
        }

        if self.rooms.len() != 10 {
            return Err(InvalidGameData::from(
                "the Chapter 1 data must contain rooms 1 through 10",
            ));
        }

        if self.bonuses.bonuses.len() != 20 {
            return Err(InvalidGameData::from(
                "the bonus pool must contain exactly 20 entries",
            ));
        }

        let mut bonus_ids = self
            .bonuses
            .bonuses
            .iter()
            .map(|bonus| bonus.id.as_str())
            .collect::<Vec<_>>();
        bonus_ids.sort_unstable();
        bonus_ids.dedup();
        if bonus_ids.len() != self.bonuses.bonuses.len() {
            return Err(InvalidGameData::from("bonus IDs must be unique"));
        }
        if self
            .bonuses
            .bonuses
            .iter()
            .any(|bonus| !self.bonuses.categories.contains_key(&bonus.category))
        {
            return Err(InvalidGameData::from(
                "a bonus references an unknown category",
            ));
        }

        for (room_id, room) in &self.rooms {
            let room_width_h = room.width_h(&self.gameplay);
            if room.room_id != *room_id || room.chapter_id != 1 {
                return Err(InvalidGameData::from(format!(
                    "room {room_id} has an inconsistent ID"
                )));
            }
            if !(2..=4).contains(&room.size.width_viewports)
                || room.size.height_h != self.gameplay.viewport.height_h
            {
                return Err(InvalidGameData::from(format!(
                    "room {room_id} has invalid dimensions"
                )));
            }
            if room.next_room_id != room.room_id + 1 {
                return Err(InvalidGameData::from(format!(
                    "room {room_id} has an invalid next_room_id"
                )));
            }
            if !self
                .backgrounds
                .profiles
                .contains_key(&room.background_profile)
            {
                return Err(InvalidGameData::from(format!(
                    "room {room_id} references an unknown background profile"
                )));
            }
            if !(0.0..=room_width_h).contains(&room.player_spawn.x_h)
                || !(0.0..=room_width_h).contains(&room.exit.x_h)
            {
                return Err(InvalidGameData::from(format!(
                    "room {room_id} has an out-of-bounds spawn or exit"
                )));
            }
            if room
                .surface_y_at(room.player_spawn.x_h)
                .is_none_or(|surface_y| (surface_y - room.player_spawn.y_h).abs() > 0.01)
            {
                return Err(InvalidGameData::from(format!(
                    "room {room_id} player spawn must use the platform top as its feet baseline"
                )));
            }

            if room_id < &10 {
                let unique_types = room
                    .enemy_spawns
                    .iter()
                    .map(|spawn| &*spawn.type_id)
                    .collect::<std::collections::BTreeSet<_>>();
                if !(5..=15).contains(&room.enemy_spawns.len()) || unique_types.len() < 2 {
                    return Err(InvalidGameData::from(format!(
                        "room {room_id} must have 5–15 spawns across at least two enemy types"
                    )));
                }
            } else if !room
                .enemy_spawns
                .iter()
                .any(|spawn| spawn.type_id == "evil_wizard")
            {
                return Err(InvalidGameData::from(
                    "room 10 must include the Evil Wizard boss",
                ));
            }

            for spawn in &room.enemy_spawns {
                if !self.enemies.types.contains_key(&spawn.type_id) {
                    return Err(InvalidGameData::from(format!(
                        "room {room_id} references unknown enemy type {}",
                        spawn.type_id
                    )));
                }
                if !(0.0..=room_width_h).contains(&spawn.position_h.x_h) {
                    return Err(InvalidGameData::from(format!(
                        "room {room_id} has an out-of-bounds enemy spawn"
                    )));
                }
                if room
                    .surface_y_at(spawn.position_h.x_h)
                    .is_none_or(|surface_y| (surface_y - spawn.position_h.y_h).abs() > 0.01)
                {
                    return Err(InvalidGameData::from(format!(
                        "room {room_id} enemy spawn must use the platform top as its feet baseline"
                    )));
                }
            }

            for platform in &room.platforms {
                let half_width = platform.size_h.width_h / 2.0;
                if platform.size_h.width_h <= 0.0
                    || platform.size_h.height_h <= 0.0
                    || platform.position_h.x_h - half_width < 0.0
                    || platform.position_h.x_h + half_width > room_width_h
                {
                    return Err(InvalidGameData::from(format!(
                        "room {room_id} has an invalid platform"
                    )));
                }
            }
        }

        for profile in self.backgrounds.profiles.values() {
            let mut assets = vec![profile.backdrop.as_str()];
            assets.extend(profile.layers.iter().map(|layer| layer.asset.as_str()));
            for asset in assets {
                if !asset_root.join(asset).is_file() {
                    return Err(InvalidGameData::from(format!(
                        "missing background asset: {asset}"
                    )));
                }
            }
        }

        Ok(())
    }
}

fn read_yaml<T: DeserializeOwned>(path: &Path) -> Result<T, Box<dyn Error + Send + Sync>> {
    let source = fs::read_to_string(path)?;
    let value = yaml_serde::from_str(&source)?;
    Ok(value)
}

#[derive(Debug)]
struct InvalidGameData(String);

impl From<&str> for InvalidGameData {
    fn from(message: &str) -> Self {
        Self(message.to_string())
    }
}

impl From<String> for InvalidGameData {
    fn from(message: String) -> Self {
        Self(message)
    }
}

impl Display for InvalidGameData {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl Error for InvalidGameData {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chapter_one_yaml_loads_and_matches_the_room_graph() {
        let asset_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/assets");
        let data = GameData::load(&asset_root).expect("Chapter 1 YAML should be valid");

        assert_eq!(data.rooms.len(), 10);
        assert_eq!(data.bonuses.bonuses.len(), 20);
        assert_eq!(data.rooms[&10].enemy_spawns[0].type_id, "evil_wizard");
        for room in data.rooms.values() {
            assert_eq!(room.platforms[0].surface, PlatformMaterial::Soil);
            assert!(
                room.platforms[1..]
                    .iter()
                    .all(|platform| platform.surface == PlatformMaterial::Stone)
            );
        }
        assert_eq!(
            data.gameplay.player.max_speed_h_s,
            data.gameplay.player.air_max_speed_h_s
        );
        let combo = &data.gameplay.player.sword.combo;
        assert!(
            combo
                .damage_multipliers
                .windows(2)
                .all(|pair| pair[1] > pair[0]),
            "longer sword combos must deal more damage per hit"
        );
    }

    #[test]
    fn bonus_category_icons_use_ascii_glyphs_available_in_the_game_font() {
        for icon in [
            BonusCategoryIcon::CrossedSwords,
            BonusCategoryIcon::Shield,
            BonusCategoryIcon::Wing,
            BonusCategoryIcon::Light,
            BonusCategoryIcon::Knot,
        ] {
            assert!(icon.to_string().is_ascii());
        }
    }
}
