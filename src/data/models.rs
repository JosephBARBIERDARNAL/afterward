use std::{
    borrow::Borrow,
    collections::BTreeMap,
    fmt::{self, Display, Formatter},
    ops::Deref,
    str::FromStr,
};

use bevy::{image::ImageSampler, prelude::Color};
use serde::{Deserialize, Deserializer, de::Error as DeserializeError};
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

impl GameplayConfig {
    pub(crate) fn viewport_width_h(&self) -> f32 {
        self.viewport.height_h * f32::from(self.viewport.aspect_ratio.width)
            / f32::from(self.viewport.aspect_ratio.height)
    }
}

#[derive(Debug, Deserialize)]
pub struct AudioConfig {
    pub footstep_interval_s: f32,
    pub paused_music_volume_multiplier: f32,
    pub room_clear_music_volume_multiplier: f32,
    pub chapter_one_music_gain: f32,
    pub chapter_one_ambience_gain: f32,
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
    pub interact: Vec<String>,
    pub pause: Vec<String>,
    pub toggle_fullscreen: Vec<String>,
    pub choose_first_bonus: Vec<String>,
    pub choose_second_bonus: Vec<String>,
    pub reroll_bonus: Vec<String>,
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
    pub air_max_speed_h_s: f32,
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
}

#[derive(Debug, Deserialize)]
pub struct SwordConfig {
    pub damage: f32,
    pub range_h: f32,
    pub attack_interval_s: f32,
    pub boss_knockback_h_s: f32,
    pub combo: SwordComboConfig,
}

#[derive(Debug, Deserialize)]
pub struct SwordComboConfig {
    pub damage_multipliers: Vec<f32>,
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
    pub jumping: EnemyJumpingConfig,
    pub types: BTreeMap<EnemyTypeId, EnemyTypeConfig>,
}

#[derive(Debug, Deserialize)]
pub struct EnemyJumpingConfig {
    pub jump_height_h: f32,
    pub air_speed_h_s: f32,
    pub cooldown_s: f32,
    pub level_change_threshold_h: f32,
}

#[derive(Debug, Clone, Eq, Ord, PartialEq, PartialOrd)]
pub struct EnemyTypeId(String);

impl FromStr for EnemyTypeId {
    type Err = &'static str;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let mut bytes = value.bytes();
        if !bytes.next().is_some_and(|first| first.is_ascii_lowercase())
            || !bytes.all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
        {
            return Err("enemy type IDs must use lowercase letters, digits, and underscores");
        }
        Ok(Self(value.to_string()))
    }
}

impl<'de> Deserialize<'de> for EnemyTypeId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer)?
            .parse()
            .map_err(D::Error::custom)
    }
}

impl Borrow<str> for EnemyTypeId {
    fn borrow(&self) -> &str {
        &self.0
    }
}

impl Deref for EnemyTypeId {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for EnemyTypeId {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl PartialEq<&str> for EnemyTypeId {
    fn eq(&self, other: &&str) -> bool {
        &**self == *other
    }
}

#[derive(Debug, Deserialize)]
pub struct EnemyTypeConfig {
    pub health: f32,
    pub movement_speed_h_s: f32,
    pub jump_height_h: Option<f32>,
    pub behavior: String,
    pub procedural_movement: Option<ProceduralMovementConfig>,
    pub preferred_distance_h: Option<DistanceRange>,
    pub attack: Option<EnemyAttackConfig>,
    #[serde(default)]
    pub attacks: Vec<EnemyAttackConfig>,
    pub phase_two: Option<BossPhaseConfig>,
}

#[derive(Debug, Deserialize)]
pub struct ProceduralMovementConfig {
    pub bob_height_h: f32,
    pub cycles_per_h: f32,
    pub lean_degrees: f32,
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
    pub filtering: TextureFiltering,
    pub layers: Vec<BackgroundLayer>,
}

/// How a texture is sampled when drawn larger than its source pixels.
/// `nearest` keeps upscaled pixel art crisp on large displays; `linear`
/// smooths it, which blurs low-resolution art.
#[derive(Debug, Clone, Copy, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TextureFiltering {
    Nearest,
    Linear,
}

impl From<TextureFiltering> for ImageSampler {
    fn from(filtering: TextureFiltering) -> Self {
        match filtering {
            TextureFiltering::Nearest => ImageSampler::nearest(),
            TextureFiltering::Linear => ImageSampler::linear(),
        }
    }
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
    pub color: HexColor,
    pub icon: BonusCategoryIcon,
}

#[derive(Debug, Clone, Copy)]
pub struct HexColor(Color);

impl FromStr for HexColor {
    type Err = &'static str;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let hex = value.strip_prefix('#').unwrap_or(value);
        if hex.len() != 6 {
            return Err("hex colors must contain six digits");
        }
        let rgb = u32::from_str_radix(hex, 16).map_err(|_| "hex colors must contain six digits")?;
        let red = f32::from(((rgb >> 16) & 0xff) as u8) / 255.0;
        let green = f32::from(((rgb >> 8) & 0xff) as u8) / 255.0;
        let blue = f32::from((rgb & 0xff) as u8) / 255.0;
        Ok(Self(Color::srgb(red, green, blue)))
    }
}

impl<'de> Deserialize<'de> for HexColor {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer)?
            .parse()
            .map_err(D::Error::custom)
    }
}

impl From<HexColor> for Color {
    fn from(color: HexColor) -> Self {
        color.0
    }
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BonusCategoryIcon {
    CrossedSwords,
    Shield,
    Wing,
    Light,
    Knot,
}

impl Display for BonusCategoryIcon {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::CrossedSwords => "X",
            Self::Shield => "O",
            Self::Wing => "^",
            Self::Light => "*",
            Self::Knot => "&",
        })
    }
}

#[derive(Debug, Deserialize)]
pub struct BonusConfig {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub effects: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct BonusEffect(f32);

impl TryFrom<&Value> for BonusEffect {
    type Error = &'static str;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        let number = value.as_f64().ok_or("bonus effects must be numeric")? as f32;
        if !number.is_finite() {
            return Err("bonus effects must be finite numbers");
        }
        Ok(Self(number))
    }
}

impl From<BonusEffect> for f32 {
    fn from(effect: BonusEffect) -> Self {
        effect.0
    }
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

impl RoomConfig {
    pub(crate) fn width_h(&self, gameplay: &GameplayConfig) -> f32 {
        f32::from(self.size.width_viewports) * gameplay.viewport_width_h()
    }

    pub(crate) fn surface_y_at(&self, x_h: f32) -> Option<f32> {
        self.platforms
            .iter()
            .filter(|platform| {
                let half_width = platform.size_h.width_h / 2.0;
                (platform.position_h.x_h - half_width..=platform.position_h.x_h + half_width)
                    .contains(&x_h)
            })
            .map(|platform| platform.position_h.y_h + platform.size_h.height_h / 2.0)
            .max_by(f32::total_cmp)
    }
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
    pub surface: PlatformMaterial,
    pub position_h: SpawnPosition,
    pub size_h: PlatformSize,
}

#[derive(Debug, Clone, Copy, Deserialize, Eq, Ord, PartialEq, PartialOrd)]
#[serde(rename_all = "snake_case")]
pub enum PlatformMaterial {
    Soil,
    Stone,
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
    pub type_id: EnemyTypeId,
    pub position_h: SpawnPosition,
    pub stat_overrides: Option<EnemyStatOverrides>,
}

#[derive(Debug, Deserialize)]
pub struct EnemyStatOverrides {
    pub health: Option<f32>,
    pub movement_speed_h_s: Option<f32>,
    pub attack_damage: Option<f32>,
}
