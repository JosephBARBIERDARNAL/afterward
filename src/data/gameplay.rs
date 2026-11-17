//! Gameplay tuning from `config/gameplay.yaml`: viewport, player, melee, room flow, audio, and controls.

use serde::Deserialize;
use std::collections::BTreeMap;

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
    pub enemy_markers: EnemyMarkersConfig,
    pub fonts: FontsConfig,
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
    /// A hit on the body of a metal enemy.
    pub damage_metal: AudioAsset,
    /// The hit that breaks a layer of an enemy's armor.
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
    /// The player touching down.
    pub land: AudioAsset,
    /// One step of the player running on the ground, on any surface.
    pub footstep: AudioAsset,
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
    pub move_up: Vec<String>,
    pub move_down: Vec<String>,
    pub navigate_up: Vec<String>,
    pub navigate_down: Vec<String>,
    pub jump: Vec<String>,
    pub attack: Vec<String>,
    pub heavy_attack: Vec<String>,
    pub dash: Vec<String>,
    pub interact: Vec<String>,
    pub pause: Vec<String>,
    pub toggle_fullscreen: Vec<String>,
    pub choose_first_bonus: Vec<String>,
    pub choose_second_bonus: Vec<String>,
    pub choose_third_bonus: Vec<String>,
    pub reroll_bonus: Vec<String>,
    pub show_build: Vec<String>,
    /// Presses the button a menu's focus stands on, beside `interact`.
    pub confirm: Vec<String>,
    /// Leaves a menu, beside `pause`.
    pub back: Vec<String>,
    pub gamepad: GamepadControlsConfig,
}

/// A gamepad's buttons, by action id (`jump`, `move_left`...), read beside the
/// keys. Gamepad buttons are not rebound from the controls screen.
#[derive(Debug, Deserialize)]
pub struct GamepadControlsConfig {
    /// How far the left stick leans (0 to 1) before it holds a direction.
    pub stick_threshold: f32,
    pub bindings: BTreeMap<String, Vec<String>>,
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
    /// The steepest face of a turned platform a body still walks, in degrees.
    pub walkable_slope_deg: f32,
    pub jump_impulse_h_s: f32,
    pub jumps_per_takeoff: u8,
    pub jump_release_velocity_multiplier: f32,
    /// How long a Jump that cannot be obeyed yet is remembered.
    pub jump_buffer_s: f32,
    pub damage_invulnerability_s: f32,
    pub crouch_speed_h_s: f32,
    /// Seconds the last death frame holds before the run ends.
    pub death_hold_s: f32,
    pub melee: MeleeConfig,
    pub dash: DashConfig,
    pub wall: WallConfig,
    pub climb: ClimbConfig,
    pub ledge: LedgeConfig,
}

/// Climbing over the edge of a platform caught in mid-air.
#[derive(Debug, Deserialize)]
pub struct LedgeConfig {
    /// How far above the feet a platform's top may stand and still be
    /// climbed onto rather than clung to.
    pub reach_h: f32,
    /// How long the climb takes, from the hand on the edge to standing.
    pub climb_s: f32,
}

/// Climbing a ladder: Up or Down inside its column grabs it, Up and Down
/// climb it, and Jump lets go with a hop.
#[derive(Debug, Deserialize)]
pub struct ClimbConfig {
    pub speed_h_s: f32,
    /// How far from the ladder's middle the player may stand and still grab it.
    pub reach_h: f32,
    /// The upward push of the hop off a ladder.
    pub jump_off_impulse_h_s: f32,
}

/// The player's only weapon: a melee chain. Each press of Attack lands the
/// next blow of `steps`, and Heavy Attack ends the chain on a blow of `heavy`;
/// letting `combo_window_s` lapse after one ends drops the chain back to its
/// first step.
#[derive(Debug, Deserialize)]
pub struct MeleeConfig {
    pub combo_window_s: f32,
    /// How long an Attack that cannot be thrown yet is remembered.
    pub buffer_s: f32,
    /// The shove a blow gives a body with no armor up.
    pub knockback_h_s: f32,
    /// The shove it gives the boss instead.
    pub boss_knockback_h_s: f32,
    pub impact: ImpactConfig,
    pub steps: Vec<MeleeComboStep>,
    /// The heavy blows, by how many of `steps` came before: the ones with no
    /// `requires_bonus` in order, then the rival finishers.
    pub heavy: Vec<MeleeComboStep>,
    pub slam: SlamConfig,
    /// The effect played where a blow lands.
    pub hit_effect: HitEffectConfig,
}

/// What a landed blow does to the moment: the freeze, the shake and the flare
/// of `src/game/impact.rs`.
#[derive(Debug, Deserialize)]
pub struct ImpactConfig {
    /// How long the game holds still on a hit, and on one that kills.
    pub freeze_s: f32,
    pub kill_freeze_s: f32,
    /// How far the camera is thrown by an ordinary blow, and by a heavy one.
    pub shake_h: f32,
    pub heavy_shake_h: f32,
    /// How long the camera takes to settle.
    pub shake_s: f32,
    /// How long the body that was hit flares, and how much brighter.
    pub flash_s: f32,
    pub flash_brightness: f32,
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

/// The dash: a short burst on the ground or in mid-air, the roll clip fitted
/// to its length.
#[derive(Debug, Deserialize)]
pub struct DashConfig {
    pub speed_h_s: f32,
    pub duration_s: f32,
    /// The stretch of the dash nothing can hurt the player through.
    pub invulnerable_from_s: f32,
    pub invulnerable_to_s: f32,
    /// How long after one dash ends the next may begin.
    pub cooldown_s: f32,
    /// Dashes allowed in mid-air before landing.
    pub air_dashes: u8,
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
    /// The speed the blow carries the player forward at until it connects.
    pub lunge_h_s: f32,
    /// A heavy finisher, thrown only by a run owning this bonus and only in
    /// place of the other finishers: of the ones a run holds, the last listed
    /// wins. Only a step of `heavy` may name one.
    #[serde(default)]
    pub requires_bonus: Option<String>,
}

impl MeleeConfig {
    /// Which of `heavy` Heavy Attack throws after `light_blows` of the chain:
    /// the plain heavy blows by that count, the last of them being replaced
    /// by the one finisher the run's bonuses bought, if any. `None` only when
    /// `heavy` holds no plain blow at all, which validation refuses.
    pub(crate) fn heavy_after(
        &self,
        light_blows: usize,
        owned: &dyn Fn(&str) -> bool,
    ) -> Option<usize> {
        let plain = self
            .heavy
            .iter()
            .enumerate()
            .filter(|(_, step)| step.requires_bonus.is_none())
            .map(|(index, _)| index)
            .collect::<Vec<_>>();
        let pick = light_blows.min(plain.len().checked_sub(1)?);
        if pick + 1 == plain.len()
            && let Some(finisher) = self.heavy.iter().rposition(|step| {
                step.requires_bonus
                    .as_ref()
                    .is_some_and(|bonus| owned(bonus))
            })
        {
            return Some(finisher);
        }
        Some(plain[pick])
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
    /// How fast Wall Runner carries the player up a wall.
    pub run_speed_h_s: f32,
}

/// How tall the effect a landed blow plays is drawn, by what the blow was:
/// an ordinary one, a heavy one, and one that kills.
#[derive(Debug, Deserialize)]
pub struct HitEffectConfig {
    pub spark_size_h: f32,
    pub burst_size_h: f32,
    pub kill_size_h: f32,
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
    /// How long the bonus screen holds after a choice, for the box that was
    /// pressed to flare.
    pub bonus_chosen_hold_s: f32,
    pub boss_intro_duration_s: f32,
    pub room_transition_duration_s: f32,
    pub room_intro: RoomIntroConfig,
    pub waves: WavesConfig,
    pub untouched: UntouchedConfig,
    pub exit_side: String,
    /// How fast the camera slides from one room into the next, in H a second.
    pub room_pan_speed_h_s: f32,
    /// How wide the dark gap between two rooms is, in H.
    pub room_gap_h: f32,
    pub exit_door: ExitDoorConfig,
    pub bonus_pickup: BonusPickupConfig,
    pub interact: InteractConfig,
    pub bonus_rerolls_per_run: u8,
    /// The places of a chapter (1 to 10) whose room is never shuffled.
    pub fixed_room_places: Vec<u16>,
}

/// The words the city is entered on, between the opening room and room 1.
#[derive(Debug, Deserialize)]
pub struct PrologueConfig {
    /// How fast the text is written out.
    pub characters_per_s: f32,
    /// How long the skyline takes to rise over the room, and to fade away.
    pub veil_fade_s: f32,
    /// The typing heard while the words are written.
    pub typing_sound: AudioAsset,
    /// The lines, drawn one blank line apart.
    pub lines: Vec<String>,
}

/// What the shaders in `src/game/shaders/`, the skyline behind the menus
/// (`src/game/skyline.rs`) and a touched button (`src/game/touch.rs`) are
/// tuned with.
#[derive(Debug, Deserialize)]
pub struct EffectsConfig {
    pub city_lights: CityLightsConfig,
    pub glow: GlowConfig,
    pub skyline: SkylineConfig,
    pub touch: TouchConfig,
    pub bonus_pick: BonusPickConfig,
    pub hazard: HazardLookConfig,
    pub bolt: BoltLookConfig,
    pub glass: GlassLookConfig,
    pub telegraph: TelegraphConfig,
    pub tells: TellsConfig,
}

/// How a bonus shows it is at work (`src/game/tells/`); the YAML says what
/// each value does.
#[derive(Debug, Deserialize)]
pub struct TellsConfig {
    pub callout_px: f32,
    pub callout_s: f32,
    pub callout_rise_h_s: f32,
    pub callout_repeat_s: f32,
    pub afterimage_s: f32,
    pub afterimage_every_s: f32,
    pub afterimage_alpha: f32,
    pub aura_radius_h: f32,
    pub boosted_number_growth: f32,
    pub boosted_number_max: f32,
    pub boosted_color: HexColor,
    pub healed_color: HexColor,
    pub soaked_color: HexColor,
    /// What the tells of speed, of a heavier blow and of reading an enemy
    /// are tinted.
    pub speed_color: HexColor,
    pub power_color: HexColor,
    pub insight_color: HexColor,
}

/// How an enemy's plain shot is drawn (`shaders/bolt.wgsl`): its two
/// colours, how many times longer and thicker than what hurts the whole
/// streak is, and its grid of art pixels to the H.
#[derive(Debug, Deserialize)]
pub struct BoltLookConfig {
    pub core_color: HexColor,
    pub tail_color: HexColor,
    pub length: f32,
    pub thickness: f32,
    pub pixels_per_h: f32,
    /// How tall the flash at the muzzle and the burst where it lands are.
    pub flash_size_h: f32,
}

/// The warning every enemy wears while it winds up: its sprite warms toward
/// `color`, `brightness` times as bright by the moment it strikes.
#[derive(Debug, Deserialize)]
pub struct TelegraphConfig {
    pub color: HexColor,
    pub brightness: f32,
}

/// How fire is drawn (`shaders/fire.wgsl`): the colour of its warning embers
/// and of a flame's edge, the colour of a flame's core, and how many art
/// pixels it is drawn with to the H.
#[derive(Debug, Deserialize)]
pub struct HazardLookConfig {
    pub warning_color: HexColor,
    pub burn_color: HexColor,
    pub pixels_per_h: f32,
}

/// How the hidden player is drawn (`shaders/glass.wgsl`).
#[derive(Debug, Deserialize)]
pub struct GlassLookConfig {
    /// The colour the clear body is filled with, and how much of it shows.
    pub fill_color: HexColor,
    pub fill_opacity: f32,
    /// The colour of the light edging it, and of the glint.
    pub rim_color: HexColor,
    /// How far a row of it ripples sideways, in art pixels.
    pub ripple_px: f32,
    /// How long before the hiding ends the sprite starts coming back
    /// through the glass.
    pub fade_out_s: f32,
}

/// How the box of a chosen bonus flares while the bonus screen holds.
#[derive(Debug, Deserialize)]
pub struct BonusPickConfig {
    /// How far the ring spreads out from the chosen box's edge, in pixels.
    pub ring_px: f32,
    /// How much the chosen box swells at the height of its flare.
    pub pulse_scale: f32,
}

/// What touching a button shows: corner brackets snapping onto one hovered or
/// reached with an arrow, and a blow's spark where one is pressed.
#[derive(Debug, Deserialize)]
pub struct TouchConfig {
    /// The brackets' colour, how long each arm is and how thick, in pixels.
    pub bracket_color: HexColor,
    pub arm_px: f32,
    pub thickness_px: f32,
    /// How far outside the button's corners they start and where they come
    /// to rest, in pixels, and how long the snap takes.
    pub snap_px: f32,
    pub gap_px: f32,
    pub snap_s: f32,
    /// How long they stay on the button, and the last stretch of that over
    /// which they fade out.
    pub lock_s: f32,
    pub fade_s: f32,
    /// How wide the spark of a press is drawn, in pixels, and the frame of
    /// `effect_hit_spark` it starts on.
    pub strike_px: f32,
    pub strike_first_frame: usize,
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

/// The city the screens standing over no room are drawn over, and the
/// prologue writes its words over.
#[derive(Debug, Deserialize)]
pub struct SkylineConfig {
    /// The background profile it is drawn from.
    pub profile: String,
    /// How fast it pans by, in H per second: each layer slides that times its
    /// own parallax.
    pub drift_h_s: f32,
    /// How opaque the dark veil drawn over it is.
    pub dim: f32,
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
    /// How bright a health drop's light burns, as a share of `strength`.
    pub health_brightness: f32,
    pub bonus_color: HexColor,
    pub health_color: HexColor,
    pub door_color: HexColor,
    /// How much brighter the aura of the thing within reach of Interact burns.
    pub reach_boost: f32,
}

/// The bonus item lying on the room's `bonus` spot once it is cleared.
#[derive(Debug, Deserialize)]
pub struct BonusPickupConfig {
    /// Path under `src/assets/`.
    pub sheet: String,
    pub frame_size_px: ImageSizeConfig,
    pub frames: usize,
    pub fps: f32,
    pub height_h: f32,
    /// How close the player must stand for Interact to take the item.
    pub reach_h: f32,
}

/// What Interact acts on, and the key badge standing over it.
#[derive(Debug, Deserialize)]
pub struct InteractConfig {
    /// How far to either side of the unlocked door, and above or below its
    /// middle, the player can use it from.
    pub door_reach_x_h: f32,
    pub door_reach_y_h: f32,
    /// The prompt naming the key, and what it does, at the bottom of the
    /// screen while something is in reach: its key badge's size, and how far
    /// it stands above the screen's bottom edge, in pixels.
    pub prompt_key_px: f32,
    pub prompt_margin_px: f32,
}

/// The exit door's sprite sheet: frame 0 locked, 1 unlocked, then the
/// opening frames.
#[derive(Debug, Deserialize)]
pub struct ExitDoorConfig {
    /// Path under `src/assets/`.
    pub sheet: String,
    pub frame_size_px: ImageSizeConfig,
    pub frames: usize,
    /// How many of the sheet's pixels make one H: the door is drawn at that
    /// scale, never resized to a height, so its pixels stay whole.
    pub px_per_h: f32,
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

/// What clearing a room without losing any health earns.
#[derive(Debug, Deserialize)]
pub struct UntouchedConfig {
    /// How many times heavier every rarity above common weighs in the draw
    /// of that room's bonuses.
    pub rarer_weight_multiplier: u32,
    /// How much further that room's bonus item glows.
    pub glow_multiplier: f32,
    /// The word standing over the item.
    pub label: String,
}

/// How a room's later waves of enemies arrive.
#[derive(Debug, Deserialize)]
pub struct WavesConfig {
    /// The breath between the last enemy of a wave going down and the next
    /// wave arriving.
    pub delay_s: f32,
    /// How long an arriving enemy takes to rise out of the floor, held where
    /// it stands.
    pub arrive_s: f32,
}

/// The rings at the screen's edge that point at the enemies of the wave out
/// of view, each showing the head of the one it stands for. Sizes are in UI
/// pixels of the 16:9 viewport.
#[derive(Debug, Deserialize)]
pub struct EnemyMarkersConfig {
    /// Whether a player with no saved settings sees them.
    pub on_by_default: bool,
    pub diameter_px: f32,
    /// How far a marker's ring stands in from the edge of the screen.
    pub edge_margin_px: f32,
    pub border_px: f32,
    pub border_color: HexColor,
    pub fill_color: HexColor,
    /// The pointer on the ring's rim, which turns toward the enemy.
    pub pointer_px: f32,
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

/// The font files of the game, under the asset root: `default` writes every
/// text that names no `FontRole`, the others the texts that do.
#[derive(Debug, Deserialize)]
pub struct FontsConfig {
    pub default: String,
    pub regular: String,
    pub italic: String,
    pub bold_italic: String,
    pub display: String,
    pub display_italic: String,
}

impl FontsConfig {
    /// Every file named, the default first.
    pub(crate) fn files(&self) -> [&String; 6] {
        [
            &self.default,
            &self.regular,
            &self.italic,
            &self.bold_italic,
            &self.display,
            &self.display_italic,
        ]
    }
}
