//! Gameplay tuning from `config/gameplay.yaml`: viewport, player, gun, room flow, audio, and controls.

use serde::Deserialize;

use super::{HexColor, ImageSizeConfig, PerAim};

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
    pub prologue: PrologueConfig,
    pub effects: EffectsConfig,
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
    /// How fast the music plays once a room is cleared, as a fraction of its
    /// recorded speed: below 1 it drags, dropping in pitch as well as tempo.
    pub room_clear_music_speed: f32,
    /// How muffled the music sounds while paused: 0 leaves it untouched, 1 cuts
    /// the highs hardest.
    pub paused_music_muffle: f32,
    /// Background music; one is picked at random when a run starts.
    pub music_tracks: Vec<AudioAsset>,
    /// The opening music, playing from launch until a run leaves room 0.
    pub intro_music: AudioAsset,
    /// How long the opening music takes to fade away once it is over.
    pub intro_music_fade_out_s: f32,
    /// The city heard under everything for the length of a run.
    pub ambience: AudioAsset,
    /// How long the city takes to rise to its gain when a run opens.
    pub ambience_fade_in_s: f32,
    pub recorded_effects: RecordedEffects,
}

/// The recording each sound cue plays. Cues missing from here have no sound
/// yet and stay silent.
#[derive(Debug, Deserialize)]
pub struct RecordedEffects {
    pub gun_shot: AudioAsset,
    pub reload: AudioAsset,
    pub door_open: AudioAsset,
    /// A hit on the player, or on an enemy of polymer.
    pub damage_neutral: AudioAsset,
    /// A hit on a metal enemy whose shield is already gone.
    pub damage_metal: AudioAsset,
    /// The hit that breaks a metal enemy's shield.
    pub damage_metal_armor: AudioAsset,
    /// Taking one of the two bonuses on offer.
    pub bonus_chosen: AudioAsset,
    /// Spending a reroll on the bonus screen.
    pub bonus_reroll: AudioAsset,
}

/// A sound file and how loud it plays relative to its volume setting.
#[derive(Debug, Deserialize)]
pub struct AudioAsset {
    /// Path under `src/assets/`.
    pub asset: String,
    /// Multiplies the volume setting of the channel the sound plays on.
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
    pub hurtbox_h: SizeH,
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
    pub crouch_speed_h_s: f32,
    /// Damage multiplier for hits from the front while Cover is held.
    pub cover_damage_taken_multiplier: f32,
    /// Seconds the last death frame holds before the run ends.
    pub death_hold_s: f32,
    pub gun: GunConfig,
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

#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
pub struct SizeH {
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Deserialize)]
pub struct HealthPickupConfig {
    pub healing: f32,
    pub base_drop_chance: f32,
    /// Path under `src/assets/`.
    pub sprite: String,
    pub sprite_size_px: ImageSizeConfig,
    pub size_h: f32,
}

#[derive(Debug, Deserialize)]
pub struct RoomFlowConfig {
    pub clear_animation_s: f32,
    pub bonus_reroll_animation_s: f32,
    pub boss_intro_duration_s: f32,
    pub room_transition_duration_s: f32,
    pub room_intro: RoomIntroConfig,
    pub exit_side: String,
    /// How long the camera takes to slide from one room into the next.
    pub room_pan_s: f32,
    /// How wide the dark gap between two rooms is, in H.
    pub room_gap_h: f32,
    pub exit_door: ExitDoorConfig,
    pub bonus_pickup: BonusPickupConfig,
    pub bonus_rerolls_per_run: u8,
}

/// The words the city is entered on, between the opening room and room 1.
#[derive(Debug, Deserialize)]
pub struct PrologueConfig {
    /// How fast the text is written out.
    pub characters_per_s: f32,
    /// How long the finished text stands before the run moves on.
    pub hold_s: f32,
    /// How long the water takes to rise over the room, and to drain away.
    pub veil_fade_s: f32,
    /// The typing heard while the words are written.
    pub typing_sound: AudioAsset,
    /// The lines, drawn one blank line apart.
    pub lines: Vec<String>,
}

/// What the shaders in `src/game/shaders/` and `src/game/ripple.rs` are tuned
/// with.
#[derive(Debug, Deserialize)]
pub struct EffectsConfig {
    pub city_lights: CityLightsConfig,
    pub glow: GlowConfig,
    pub flood: FloodConfig,
    pub ripple: RippleConfig,
}

/// The droplet that falls where a button is hovered, selected with an arrow,
/// or clicked.
#[derive(Debug, Deserialize)]
pub struct RippleConfig {
    /// How far the rings reach from the spot the droplet lands on, in pixels,
    /// and how long the whole splash takes.
    pub radius_px: f32,
    pub duration_s: f32,
    /// How much of that run the droplet spends falling in, before the impact.
    pub fall: f32,
    /// How many water pixels the splash is across.
    pub pixels: f32,
    /// How thick a ring is, in ripple radii, and how long after one ring the
    /// next is born, as a fraction of the spread.
    pub ring_width: f32,
    pub ring_interval: f32,
    /// How far the ordered dither spreads one tone into the next.
    pub dither: f32,
    /// The two tones, and how much of the card below each one covers.
    pub crest_color: HexColor,
    pub crest_opacity: f32,
    pub foam_color: HexColor,
    pub foam_opacity: f32,
}

/// The breathing lit windows and stars of the background layers.
#[derive(Debug, Deserialize)]
pub struct CityLightsConfig {
    /// How far above the pixels around it a pixel's brightness must stand to
    /// count as a light, and the margin over which that judgement fades in.
    pub light_threshold: f32,
    pub light_softness: f32,
    /// How bright the pixels around a light may be for it to still count as
    /// one: a lit window sits on a dark facade, a dithered sky does not.
    pub surround_ceiling: f32,
    /// The widest a light's brightness swings, and how long one breath takes.
    pub swing: f32,
    pub period_s: f32,
}

/// The flood: the water the city drowned in, and the water the main menu
/// is drawn over.
#[derive(Debug, Deserialize)]
pub struct FloodConfig {
    /// How many water pixels fit in one H.
    pub pixels_per_h: f32,
    /// How wide one cell of the web is, in H.
    pub cell_h: f32,
    /// How many turns a cell's point circles in a second, how fast the whole
    /// field drifts across the screen in H per second, and how far a slow wave
    /// bends the borders, in cells.
    pub churn: f32,
    pub drift_h_s: f32,
    pub wobble: f32,
    /// How thick the lit borders of the web are, and the fainter veins
    /// crossing a cell, in cells.
    pub web_width: f32,
    pub vein_width: f32,
    /// How far the ordered dither spreads one tone into the next.
    pub dither: f32,
    /// The four tones, darkest first: the heart of a cell, its body, the
    /// lighter water towards its border, and the lit web itself.
    pub depth_color: HexColor,
    pub cell_color: HexColor,
    pub crest_color: HexColor,
    pub foam_color: HexColor,
}

/// The pool of light behind anything the player should walk towards.
#[derive(Debug, Deserialize)]
pub struct GlowConfig {
    /// How bright the core is, how far that swings, and how long one pulse takes.
    pub strength: f32,
    pub pulse: f32,
    pub pulse_period_s: f32,
    /// How far the light reaches past what it wraps, in H.
    pub bonus_radius_h: f32,
    pub health_radius_h: f32,
    pub door_radius_h: f32,
    pub health_color: HexColor,
    pub door_color: HexColor,
}

/// The bonus item lying on the room's `bonus` spot once it is cleared.
#[derive(Debug, Deserialize)]
pub struct BonusPickupConfig {
    pub size_h: f32,
    pub bob_height_h: f32,
    pub bob_period_s: f32,
    /// How close the player must come for the item to be picked up.
    pub touch_radius_h: f32,
}

/// The exit door's sprite sheet: frame 0 locked, 1 unlocked, then the
/// opening frames.
#[derive(Debug, Deserialize)]
pub struct ExitDoorConfig {
    /// Path under `src/assets/`.
    pub sheet: String,
    pub frame_size_px: ImageSizeConfig,
    pub frames: usize,
    pub height_h: f32,
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
