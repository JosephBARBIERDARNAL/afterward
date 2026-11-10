//! Gameplay tuning from `config/gameplay.yaml`: viewport, player, melee, room flow, audio, and controls.

use serde::Deserialize;

use super::{HexColor, ImageSizeConfig, MeleeStrike, RectH};

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
    pub difficulty: DifficultyColors,
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

/// The colour each difficulty's box is edged in on the screen that asks for
/// one. The choice itself has no other data, because nothing reads it yet.
#[derive(Debug, Deserialize)]
pub struct DifficultyColors {
    pub easy: HexColor,
    pub normal: HexColor,
    pub hard: HexColor,
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
    /// How muffled it sounds while a door is crossed, as if heard through the
    /// wall between two rooms.
    pub crossing_music_muffle: f32,
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
    /// The chain's blows landing on an enemy, in the chain's order. A blow
    /// past the end of the list plays the last.
    pub punch_hits: Vec<AudioAsset>,
    /// A blow that reached nobody.
    pub punch_missed: AudioAsset,
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
    pub attack: Vec<String>,
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
    pub damage_invulnerability_s: f32,
    pub crouch_speed_h_s: f32,
    /// Damage multiplier for hits from the front while Cover is held.
    pub cover_damage_taken_multiplier: f32,
    /// Seconds the last death frame holds before the run ends.
    pub death_hold_s: f32,
    pub melee: MeleeConfig,
    pub roll: RollConfig,
    pub wall: WallConfig,
}

/// The player's only weapon: a melee chain. Each press lands the next blow of
/// `steps`; letting `combo_window_s` lapse after one ends drops the chain back
/// to its first step. The steps with no `requires_bonus` are the chain every
/// run throws; those with one are its rival finishers.
#[derive(Debug, Deserialize)]
pub struct MeleeConfig {
    pub combo_window_s: f32,
    /// Knockback a blow gives the boss, the only body the player can shove.
    pub boss_knockback_h_s: f32,
    pub steps: Vec<MeleeComboStep>,
    pub slam: SlamConfig,
    pub charge: ChargeConfig,
    /// The burst drawn where a blow lands.
    pub impact_burst: BurstConfig,
}

/// Attack with Down held in mid-air: the player is driven at the floor, and
/// the landing hurts everything to both sides of them.
#[derive(Debug, Deserialize)]
pub struct SlamConfig {
    pub damage: f32,
    /// Measured from the feet and mirrored around them, like a blow landing
    /// on both sides.
    pub hitbox_h: RectH,
    pub drop_speed_h_s: f32,
}

/// Attack held rather than tapped: the gather, and the blow it throws.
#[derive(Debug, Deserialize)]
pub struct ChargeConfig {
    /// How long Attack is held, after the blow in hand ends, before the
    /// charged blow is ready.
    pub ready_s: f32,
    pub damage: f32,
    pub hitbox_h: RectH,
    pub contact_s: f32,
    pub lunge_h_s: f32,
}

/// The roll Down and Jump together ask for.
#[derive(Debug, Deserialize)]
pub struct RollConfig {
    pub speed_h_s: f32,
    /// The stretch of the roll nothing can hurt the player through.
    pub invulnerable_from_s: f32,
    pub invulnerable_to_s: f32,
}

/// One blow of the chain.
#[derive(Debug, Deserialize)]
pub struct MeleeComboStep {
    /// Which blow it is, and so which clip it plays.
    pub strike: MeleeStrike,
    pub damage: f32,
    /// Where the blow lands, from the player's feet: `x` points the way they
    /// face (negative reaches behind), `y` up.
    pub hitbox_h: RectH,
    /// Whether the blow reaches behind the player as well as in front, which
    /// mirrors `hitbox_h` around their feet instead of pointing it.
    #[serde(default)]
    pub both_sides: bool,
    /// How far into the clip the blow connects.
    pub contact_s: f32,
    /// Forward shove the blow gives the player, so the chain carries them in.
    pub lunge_h_s: f32,
    /// A finisher, thrown only by a run owning this bonus and only in place
    /// of the other finishers: of the ones a run holds, the last listed wins.
    #[serde(default)]
    pub requires_bonus: Option<String>,
}

impl MeleeConfig {
    /// The chain a run actually throws: every step owing nothing to a bonus,
    /// then the one finisher its bonuses bought, if any.
    pub(crate) fn chain<'a>(&'a self, owned: &dyn Fn(&str) -> bool) -> Vec<&'a MeleeComboStep> {
        self.steps
            .iter()
            .filter(|step| step.requires_bonus.is_none())
            .chain(self.steps.iter().rev().find(|step| {
                step.requires_bonus
                    .as_ref()
                    .is_some_and(|bonus| owned(bonus))
            }))
            .collect()
    }
}

/// Clinging to a platform's side and pushing off it.
#[derive(Debug, Deserialize)]
pub struct WallConfig {
    /// How fast the player slides down a wall they are pressed against.
    pub slide_speed_h_s: f32,
    /// Upward part of the push off a wall.
    pub jump_impulse_h_s: f32,
    /// Sideways part, away from the wall.
    pub jump_push_h_s: f32,
    /// Seconds after a wall jump during which movement input is ignored, so
    /// the player actually leaves the wall instead of sticking to it.
    pub control_lock_s: f32,
}

/// A fiery burst: a swelling, fading core and embers flying outward.
#[derive(Debug, Deserialize)]
pub struct BurstConfig {
    pub radius_h: f32,
    pub duration_s: f32,
    pub embers: u8,
    pub ember_speed_h_s: f32,
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
    /// How long the bonus screen holds after a choice, for the droplet to land
    /// on the box that was pressed.
    pub bonus_chosen_hold_s: f32,
    pub boss_intro_duration_s: f32,
    pub room_transition_duration_s: f32,
    pub room_intro: RoomIntroConfig,
    pub exit_side: String,
    /// How fast the camera slides from one room into the next, in H a second.
    pub room_pan_speed_h_s: f32,
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
    /// How long before the camera lands the entrance begins.
    pub early_start_s: f32,
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
