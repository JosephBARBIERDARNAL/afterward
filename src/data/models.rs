use std::collections::BTreeMap;

use serde::Deserialize;
use yaml_serde::Value;

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
    pub room_clear_music_volume_multiplier: f32,
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
    pub bonus_reroll_animation_s: f32,
    pub boss_intro_duration_s: f32,
    pub room_transition_duration_s: f32,
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
