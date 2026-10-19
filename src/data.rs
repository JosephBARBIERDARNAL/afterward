use std::{
    collections::BTreeMap,
    error::Error,
    fs,
    io::{Error as IoError, ErrorKind},
    path::Path,
};

use bevy::prelude::Resource;
use serde::{Deserialize, de::DeserializeOwned};
use yaml_serde::Value;

#[derive(Resource, Debug)]
pub struct GameData {
    pub gameplay: GameplayConfig,
    pub enemies: EnemyTypes,
    pub backgrounds: BackgroundProfiles,
    pub bonuses: BonusesConfig,
    pub rooms: BTreeMap<u16, RoomConfig>,
}

impl GameData {
    pub fn load(asset_root: &Path) -> Result<Self, Box<dyn Error + Send + Sync>> {
        let gameplay = read_yaml(&asset_root.join("config/gameplay.yaml"))?;
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
            enemies,
            backgrounds,
            bonuses,
            rooms,
        };
        data.validate(asset_root)?;
        Ok(data)
    }

    fn validate(&self, asset_root: &Path) -> Result<(), Box<dyn Error + Send + Sync>> {
        if self.gameplay.viewport.height_h != 9.0
            || self.gameplay.viewport.aspect_ratio.width != 16
            || self.gameplay.viewport.aspect_ratio.height != 9
        {
            return Err(invalid_data(
                "the gameplay viewport must be fixed at 16:9 and 9 H",
            ));
        }
        let expected_jump_impulse =
            (2.0 * self.gameplay.player.gravity_h_s2 * self.gameplay.player.jump_height_h).sqrt();
        if (expected_jump_impulse - self.gameplay.player.jump_impulse_h_s).abs() > 0.05 {
            return Err(invalid_data(
                "jump height, gravity, and jump impulse must describe the same jump",
            ));
        }
        if self.gameplay.room_flow.clear_animation_s <= 0.0
            || self.gameplay.room_flow.exit_side != "right"
        {
            return Err(invalid_data(
                "Chapter 1 room flow requires a positive clear animation and right-side exits",
            ));
        }
        let audio = &self.gameplay.audio;
        if audio.footstep_interval_s <= 0.0
            || !(0.0..=1.0).contains(&audio.paused_music_volume_multiplier)
            || !(0.0..=1.0).contains(&audio.chapter_one_music_gain)
        {
            return Err(invalid_data("audio tuning values are out of range"));
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
            &controls.gun,
            &controls.interact,
            &controls.pause,
        ];
        if default_bindings.iter().any(|binding| binding.is_empty()) {
            return Err(invalid_data("each game action needs a default key binding"));
        }

        if self.rooms.len() != 10 {
            return Err(invalid_data(
                "the Chapter 1 data must contain rooms 1 through 10",
            ));
        }

        if self.bonuses.bonuses.len() != 20 {
            return Err(invalid_data(
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
            return Err(invalid_data("bonus IDs must be unique"));
        }
        if self
            .bonuses
            .bonuses
            .iter()
            .any(|bonus| !self.bonuses.categories.contains_key(&bonus.category))
        {
            return Err(invalid_data("a bonus references an unknown category"));
        }

        for (room_id, room) in &self.rooms {
            let room_width_h = f32::from(room.size.width_viewports) * 16.0;
            if room.room_id != *room_id || room.chapter_id != 1 {
                return Err(invalid_data(format!(
                    "room {room_id} has an inconsistent ID"
                )));
            }
            if !(2..=4).contains(&room.size.width_viewports) || room.size.height_h != 9.0 {
                return Err(invalid_data(format!(
                    "room {room_id} has invalid dimensions"
                )));
            }
            if room.next_room_id != room.room_id + 1 {
                return Err(invalid_data(format!(
                    "room {room_id} has an invalid next_room_id"
                )));
            }
            if !self
                .backgrounds
                .profiles
                .contains_key(&room.background_profile)
            {
                return Err(invalid_data(format!(
                    "room {room_id} references an unknown background profile"
                )));
            }
            if !(0.0..=room_width_h).contains(&room.player_spawn.x_h)
                || !(0.0..=room_width_h).contains(&room.exit.x_h)
            {
                return Err(invalid_data(format!(
                    "room {room_id} has an out-of-bounds spawn or exit"
                )));
            }

            if room_id < &10 {
                let unique_types = room
                    .enemy_spawns
                    .iter()
                    .map(|spawn| spawn.type_id.as_str())
                    .collect::<std::collections::BTreeSet<_>>();
                if !(5..=15).contains(&room.enemy_spawns.len()) || unique_types.len() < 2 {
                    return Err(invalid_data(format!(
                        "room {room_id} must have 5–15 spawns across at least two enemy types"
                    )));
                }
            } else if !room
                .enemy_spawns
                .iter()
                .any(|spawn| spawn.type_id == "warrior")
            {
                return Err(invalid_data("room 10 must include the Warrior boss"));
            }

            for spawn in &room.enemy_spawns {
                if !self.enemies.types.contains_key(&spawn.type_id) {
                    return Err(invalid_data(format!(
                        "room {room_id} references unknown enemy type {}",
                        spawn.type_id
                    )));
                }
                if !(0.0..=room_width_h).contains(&spawn.position_h.x_h) {
                    return Err(invalid_data(format!(
                        "room {room_id} has an out-of-bounds enemy spawn"
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
                    return Err(invalid_data(format!(
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
                    return Err(invalid_data(format!("missing background asset: {asset}")));
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

fn invalid_data(message: impl Into<String>) -> Box<dyn Error + Send + Sync> {
    Box::new(IoError::new(ErrorKind::InvalidData, message.into()))
}

#[derive(Debug, Deserialize)]
pub struct GameplayConfig {
    pub viewport: ViewportConfig,
    pub player: PlayerConfig,
    pub health_pickup: HealthPickupConfig,
    pub room_flow: RoomFlowConfig,
    pub window: WindowConfig,
    pub audio: AudioConfig,
    pub controls: ControlsConfig,
}

#[derive(Debug, Deserialize)]
pub struct AudioConfig {
    pub footstep_interval_s: f32,
    pub paused_music_volume_multiplier: f32,
    pub chapter_one_music_gain: f32,
}

#[derive(Debug, Deserialize)]
pub struct ControlsConfig {
    pub move_left: Vec<String>,
    pub move_right: Vec<String>,
    pub navigate_up: Vec<String>,
    pub navigate_down: Vec<String>,
    pub jump: Vec<String>,
    pub dash: Vec<String>,
    pub sword: Vec<String>,
    pub gun: Vec<String>,
    pub interact: Vec<String>,
    pub pause: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct ViewportConfig {
    pub aspect_ratio: AspectRatio,
    pub height_h: f32,
}

#[derive(Debug, Deserialize)]
pub struct AspectRatio {
    pub width: u16,
    pub height: u16,
}

#[derive(Debug, Deserialize)]
pub struct PlayerConfig {
    pub max_health: f32,
    pub max_speed_h_s: f32,
    pub acceleration_h_s2: f32,
    pub deceleration_h_s2: f32,
    pub jump_height_h: f32,
    pub gravity_h_s2: f32,
    pub jump_impulse_h_s: f32,
    pub jumps_per_takeoff: u8,
    pub dash_distance_h: f32,
    pub dash_duration_s: f32,
    pub dash_cooldown_s: f32,
    pub damage_invulnerability_s: f32,
    pub dash_invulnerability_s: f32,
    pub sword: SwordConfig,
    pub gun: GunConfig,
    pub boss_knockback: BossKnockbackConfig,
}

#[derive(Debug, Deserialize)]
pub struct SwordConfig {
    pub damage: f32,
    pub range_h: f32,
    pub attack_interval_s: f32,
}

#[derive(Debug, Deserialize)]
pub struct GunConfig {
    pub damage: f32,
    pub fire_interval_s: f32,
    pub projectile_speed_h_s: f32,
}

#[derive(Debug, Deserialize)]
pub struct BossKnockbackConfig {
    pub sword_h_s: f32,
    pub bullet_h_s: f32,
}

#[derive(Debug, Deserialize)]
pub struct HealthPickupConfig {
    pub healing: f32,
    pub base_drop_chance: f32,
}

#[derive(Debug, Deserialize)]
pub struct RoomFlowConfig {
    pub clear_animation_s: f32,
    pub exit_side: String,
    pub bonus_rerolls_per_run: u8,
}

#[derive(Debug, Deserialize)]
pub struct WindowConfig {
    pub start_borderless_fullscreen: bool,
    pub resizable_windowed: bool,
    pub music_volume_percent: u8,
    pub sfx_volume_percent: u8,
}

#[derive(Debug, Deserialize)]
pub struct EnemyTypes {
    pub detection_range_h: f32,
    pub retain_target_after_detection: bool,
    pub damage_triggers_combat: bool,
    pub types: BTreeMap<String, EnemyTypeConfig>,
}

#[derive(Debug, Deserialize)]
pub struct EnemyTypeConfig {
    pub health: f32,
    pub movement_speed_h_s: f32,
    pub behavior: String,
    pub preferred_distance_h: Option<DistanceRange>,
    pub attack: Option<EnemyAttackConfig>,
    #[serde(default)]
    pub attacks: Vec<EnemyAttackConfig>,
    pub phase_two: Option<BossPhaseConfig>,
}

#[derive(Debug, Deserialize)]
pub struct DistanceRange {
    pub min: f32,
    pub max: f32,
}

#[derive(Debug, Deserialize)]
pub struct EnemyAttackConfig {
    pub kind: String,
    pub damage: Option<f32>,
    pub damage_per_hit: Option<f32>,
    pub hits: Option<u8>,
    pub reach_h: Option<f32>,
    pub projectile_speed_h_s: Option<f32>,
    pub landing_radius_h: Option<f32>,
    pub cooldown_s: f32,
    pub windup_s: f32,
}

#[derive(Debug, Deserialize)]
pub struct BossPhaseConfig {
    pub health_fraction_at_or_below: f32,
    pub cooldown_multiplier: f32,
}

#[derive(Debug, Deserialize)]
pub struct BackgroundProfiles {
    pub profiles: BTreeMap<String, BackgroundProfile>,
}

#[derive(Debug, Deserialize)]
pub struct BackgroundProfile {
    pub backdrop: String,
    pub source_size_px: ImageSizeConfig,
    pub tile_h: f32,
    #[serde(default)]
    pub repeat_horizontal: bool,
    pub layers: Vec<BackgroundLayer>,
}

#[derive(Debug, Deserialize)]
pub struct ImageSizeConfig {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Deserialize)]
pub struct BackgroundLayer {
    pub asset: String,
    pub parallax: f32,
}

#[derive(Debug, Deserialize)]
pub struct BonusesConfig {
    pub categories: BTreeMap<String, BonusCategoryConfig>,
    pub bonuses: Vec<BonusConfig>,
}

#[derive(Debug, Deserialize)]
pub struct BonusCategoryConfig {
    pub color: String,
    pub icon: String,
}

#[derive(Debug, Deserialize)]
pub struct BonusConfig {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub effects: BTreeMap<String, Value>,
}

#[derive(Debug, Deserialize)]
pub struct RoomConfig {
    pub room_id: u16,
    pub chapter_id: u8,
    pub size: RoomSizeConfig,
    pub background_profile: String,
    pub player_spawn: SpawnPosition,
    pub camera: CameraConfig,
    pub platforms: Vec<PlatformConfig>,
    #[serde(default)]
    pub decorations: Vec<DecorationConfig>,
    pub enemy_spawns: Vec<EnemySpawnConfig>,
    pub exit: SpawnPosition,
    pub next_room_id: u16,
}

#[derive(Debug, Deserialize)]
pub struct RoomSizeConfig {
    pub width_viewports: u8,
    pub height_h: f32,
}

#[derive(Debug, Deserialize)]
pub struct SpawnPosition {
    #[serde(alias = "x")]
    pub x_h: f32,
    #[serde(alias = "y")]
    pub y_h: f32,
}

#[derive(Debug, Deserialize)]
pub struct CameraConfig {
    pub follow_player_x: bool,
    pub fixed_y_h: f32,
}

#[derive(Debug, Deserialize)]
pub struct PlatformConfig {
    pub position_h: SpawnPosition,
    pub size_h: PlatformSize,
}

#[derive(Debug, Deserialize)]
pub struct PlatformSize {
    #[serde(alias = "width")]
    pub width_h: f32,
    #[serde(alias = "height")]
    pub height_h: f32,
}

#[derive(Debug, Deserialize)]
pub struct DecorationConfig {
    pub asset: String,
    pub position_h: SpawnPosition,
    pub visual_layer: String,
}

#[derive(Debug, Deserialize)]
pub struct EnemySpawnConfig {
    pub type_id: String,
    pub position_h: SpawnPosition,
    pub stat_overrides: Option<EnemyStatOverrides>,
}

#[derive(Debug, Deserialize)]
pub struct EnemyStatOverrides {
    pub health: Option<f32>,
    pub movement_speed_h_s: Option<f32>,
    pub attack_damage: Option<f32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chapter_one_yaml_loads_and_matches_the_room_graph() {
        let asset_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/assets");
        let data = GameData::load(&asset_root).expect("Chapter 1 YAML should be valid");

        assert_eq!(data.rooms.len(), 10);
        assert_eq!(data.bonuses.bonuses.len(), 20);
        assert_eq!(data.rooms[&10].enemy_spawns[0].type_id, "warrior");
    }
}
