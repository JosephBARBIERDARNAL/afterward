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

use super::PerAim;

/// Width of the player's (and every enemy's) collision box, in H. Its height
/// is 1 H by definition.
pub const PLAYER_COLLISION_WIDTH_H: f32 = 0.6;

#[derive(Debug, Deserialize)]
pub struct GameplayConfig {
    pub viewport: ViewportConfig,
    pub room_size_limits: RoomSizeLimits,
    pub camera: CameraFollowConfig,
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
    pub move_down: Vec<String>,
    pub navigate_up: Vec<String>,
    pub navigate_down: Vec<String>,
    pub jump: Vec<String>,
    pub dash: Vec<String>,
    pub shoot: Vec<String>,
    pub aim_up: Vec<String>,
    pub reload: Vec<String>,
    pub cover: Vec<String>,
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

/// Allowed room dimensions, in multiples of the gameplay viewport.
#[derive(Debug, Deserialize)]
pub struct RoomSizeLimits {
    pub width_viewports: RangeConfig,
    pub height_viewports: RangeConfig,
}

/// The camera follows the player on both axes, clamped to the room. It only
/// moves vertically once the player leaves a band around the screen center.
#[derive(Debug, Deserialize)]
pub struct CameraFollowConfig {
    pub vertical_dead_zone_h: f32,
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
    pub jump_release_velocity_multiplier: f32,
    pub dash_distance_h: f32,
    pub dash_duration_s: f32,
    pub dash_cooldown_s: f32,
    pub damage_invulnerability_s: f32,
    pub dash_invulnerability_s: f32,
    pub walls: WallMovementConfig,
    /// Seconds one-way platforms ignore the player after Down + Jump.
    pub drop_through_s: f32,
    pub crouch_speed_h_s: f32,
    /// Damage multiplier for hits from the front while Cover is held.
    pub cover_damage_taken_multiplier: f32,
    /// Seconds the last death frame holds before the run ends.
    pub death_hold_s: f32,
    pub gun: GunConfig,
}

/// Wall slide, wall jump, and ledge mantling.
#[derive(Debug, Deserialize)]
pub struct WallMovementConfig {
    pub slide_max_fall_speed_h_s: f32,
    pub jump_impulse_h_s: f32,
    pub jump_kick_speed_h_s: f32,
    /// Seconds after a wall jump during which horizontal input is ignored, so
    /// the kick carries the player away from the wall.
    pub jump_control_lock_s: f32,
    /// How far above the top of the body a ledge can still be grabbed.
    pub ledge_grab_reach_h: f32,
    pub ledge_climb_s: f32,
    /// Seconds a falling player pressing into a wall clings to it before sliding.
    pub latch_s: f32,
}

/// The player's only weapon: a gun firing visible bullets from a magazine.
#[derive(Debug, Deserialize)]
pub struct GunConfig {
    pub damage: f32,
    pub fire_interval_s: f32,
    pub magazine_size: u32,
    pub reload_s: f32,
    pub bullet_speed_h_s: f32,
    pub bullet_range_h: f32,
    pub bullet_size_h: SizeH,
    pub diagonal_angle_degrees: f32,
    pub boss_knockback_h_s: f32,
    /// Bullet origin from the body center, facing right.
    pub muzzle_offset_h: PerAim<PointH>,
    pub muzzle_flash: BurstConfig,
    pub impact_burst: BurstConfig,
}

/// A fiery burst: a swelling, fading core and embers flying outward.
#[derive(Debug, Deserialize)]
pub struct BurstConfig {
    pub radius_h: f32,
    pub duration_s: f32,
    pub embers: u8,
    pub ember_speed_h_s: f32,
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct PointH {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct SizeH {
    pub width: f32,
    pub height: f32,
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
    pub room_intro: RoomIntroConfig,
    pub exit_side: String,
    pub bonus_rerolls_per_run: u8,
}

/// Room entrance timeline: the player walks in, holds idle while the enemies
/// materialize one after another, then the fight starts.
#[derive(Debug, Deserialize)]
pub struct RoomIntroConfig {
    pub player_walk_in_s: f32,
    pub player_walk_in_speed_h_s: f32,
    pub player_fade_in_s: f32,
    pub player_idle_hold_s: f32,
    pub enemy_appear_s: f32,
    pub enemy_appear_spread_s: f32,
    pub fight_start_delay_s: f32,
}

impl RoomIntroConfig {
    /// Seconds into the intro at which the first enemy starts to appear.
    pub(crate) fn enemies_start_s(&self) -> f32 {
        self.player_walk_in_s + self.player_idle_hold_s
    }

    pub(crate) fn duration_s(&self) -> f32 {
        self.enemies_start_s()
            + self.enemy_appear_spread_s
            + self.enemy_appear_s
            + self.fight_start_delay_s
    }
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
    pub preferred_distance_h: Option<RangeConfig>,
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
pub struct RangeConfig {
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
    /// Solid blocks (floors, walls, ceilings, pillars): they block bodies and
    /// projectiles from every side.
    pub terrain: Vec<TerrainBlockConfig>,
    /// One-way platforms: bodies land on their top, jump through them from
    /// below, and drop through them on purpose.
    #[serde(default)]
    pub platforms: Vec<TerrainBlockConfig>,
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

    /// Whether feet at (`x_h`, `y_h`) stand on a block top with a body's
    /// worth of room above them, free of solid terrain.
    pub(crate) fn has_footing_at(&self, x_h: f32, y_h: f32) -> bool {
        let stands =
            self.terrain.iter().chain(&self.platforms).any(|block| {
                block.rect_h.covers_x(x_h) && (block.rect_h.top() - y_h).abs() <= 0.01
            });
        stands && self.body_is_clear(x_h, y_h)
    }

    /// Whether a 1 H tall collision box centered on `x_h`, with its feet at
    /// `y_h`, overlaps no solid terrain.
    pub(crate) fn body_is_clear(&self, x_h: f32, y_h: f32) -> bool {
        let body = RectH {
            x: x_h - PLAYER_COLLISION_WIDTH_H / 2.0,
            y: y_h,
            width: PLAYER_COLLISION_WIDTH_H,
            height: 1.0,
        };
        !self
            .terrain
            .iter()
            .any(|block| block.rect_h.overlaps(&body))
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
pub struct TerrainBlockConfig {
    pub surface: PlatformMaterial,
    pub rect_h: RectH,
}

/// An axis-aligned rectangle in H; `x`/`y` is its lower-left corner.
#[derive(Debug, Clone, Copy, Deserialize)]
pub struct RectH {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl RectH {
    pub(crate) fn right(&self) -> f32 {
        self.x + self.width
    }

    pub(crate) fn top(&self) -> f32 {
        self.y + self.height
    }

    fn covers_x(&self, x: f32) -> bool {
        (self.x..=self.right()).contains(&x)
    }

    /// Strict overlap: rectangles that only share an edge do not overlap.
    fn overlaps(&self, other: &Self) -> bool {
        const EPSILON: f32 = 0.001;
        self.x < other.right() - EPSILON
            && other.x < self.right() - EPSILON
            && self.y < other.top() - EPSILON
            && other.y < self.top() - EPSILON
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Eq, Ord, PartialEq, PartialOrd)]
#[serde(rename_all = "snake_case")]
pub enum PlatformMaterial {
    Soil,
    Stone,
    Wood,
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
