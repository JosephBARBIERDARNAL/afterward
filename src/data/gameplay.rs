//! Gameplay tuning from `config/gameplay.yaml`: viewport, player, gun, room flow, audio, and controls.

use serde::Deserialize;

use super::PerAim;

/// Width of the player's (and every enemy's) collision box, in H. Its height
/// is 1 H by definition.
pub const PLAYER_COLLISION_WIDTH_H: f32 = 0.6;

#[derive(Debug, Deserialize)]
pub struct GameplayConfig {
    pub viewport: ViewportConfig,
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
    pub recorded_effects: RecordedEffects,
}

/// Recorded sound files that replace the synthesized tone of their cue.
#[derive(Debug, Deserialize)]
pub struct RecordedEffects {
    pub gun_shot: RecordedEffect,
    pub reload: RecordedEffect,
    pub door_open: RecordedEffect,
}

#[derive(Debug, Deserialize)]
pub struct RecordedEffect {
    /// Path under `src/assets/`.
    pub asset: String,
    /// Multiplies the sound-effects volume setting.
    pub gain: f32,
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
    pub start_mode: WindowStartMode,
    pub resizable_windowed: bool,
    pub music_volume_percent: u8,
    pub sfx_volume_percent: u8,
}

/// How the game window opens.
#[derive(Debug, Clone, Copy, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum WindowStartMode {
    /// A regular window filling the screen, keeping the menu bar or taskbar.
    Maximized,
    /// Covers the whole screen without a window frame.
    BorderlessFullscreen,
    /// A regular window at the platform's default size.
    Windowed,
}

#[derive(Debug, Deserialize)]
pub struct RangeConfig {
    pub min: f32,
    pub max: f32,
}
