use std::{
    collections::{BTreeMap, BTreeSet},
    time::{SystemTime, UNIX_EPOCH},
};

use bevy::{ecs::system::SystemParam, prelude::*};

use crate::data::{
    Aim, EnemyAttackConfig, EnemyTypeId, GameData, PLAYER_COLLISION_WIDTH_H, PlatformMaterial,
};

mod ammo_hud;
mod assets;
mod audio;
mod bonus_screen;
mod camera;
mod combat;
mod controls;
mod flow;
mod interface;
mod pause_blur;
mod player;
mod player_model;
mod room_intro;
mod screens;
mod settings;
mod terrain;
mod visual_walkthrough;
mod world;

pub use visual_walkthrough::VisualWalkthroughPlugin;

#[cfg(test)]
mod tests;

use audio::{SoundCue, SoundQueue};
use controls::{ControlBindings, GameAction};
use settings::AudioSettings;
use terrain::{PlatformSurface, Terrain};

const H_WORLD_UNITS: f32 = 48.0;
/// Horizontal speed, in world units per second, above which a body counts as moving.
const MOVING_SPEED_THRESHOLD: f32 = 20.0;

pub struct GamePlugin;

/// Room a new run starts in. Defaults to room 1; `--room <id>` overrides it
/// for debugging, and restarts from the pause menu return to the same room.
#[derive(Resource, Clone, Copy, Debug, PartialEq, Eq)]
pub struct StartRoom(pub u16);

impl Default for StartRoom {
    fn default() -> Self {
        Self(1)
    }
}

impl std::str::FromStr for StartRoom {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        value
            .parse()
            .map(Self)
            .map_err(|_| format!("--room expects a room ID such as 3, got {value:?}"))
    }
}

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(pause_blur::PauseBlurPlugin)
            .init_state::<GameScreen>()
            .init_resource::<RunSession>()
            .init_resource::<StartRoom>()
            .init_resource::<AudioSettings>()
            .init_resource::<ControlBindings>()
            .init_resource::<SoundQueue>()
            .init_resource::<audio::SoundBank>()
            .insert_resource(RandomState::default())
            .insert_resource(PhaseTimer(Timer::from_seconds(0.5, TimerMode::Once)))
            .add_systems(
                Startup,
                (
                    camera::spawn_camera,
                    interface::spawn_interface,
                    ammo_hud::spawn_ammo_hud,
                    bonus_screen::spawn_bonus_screen,
                    assets::load_sprites,
                    player_model::setup_player_model.after(assets::load_sprites),
                    settings::initialize_settings,
                    controls::initialize_control_bindings,
                    audio::initialize_sound_bank,
                ),
            )
            .add_systems(
                OnEnter(GameScreen::Playing),
                (world::spawn_room, audio::start_chapter_music),
            )
            .add_systems(
                OnEnter(GameScreen::RoomClear),
                combat::clear_room_combat_effects,
            )
            .add_systems(
                OnEnter(GameScreen::RoomIntro),
                (
                    world::spawn_room,
                    room_intro::start_room_intro,
                    audio::start_chapter_music,
                ),
            )
            .add_systems(
                OnEnter(GameScreen::BossIntro),
                (flow::start_boss_intro, audio::queue_boss_intro),
            )
            .add_systems(
                OnEnter(GameScreen::RoomLoading),
                (world::spawn_room, flow::start_room_transition_fade_out).chain(),
            )
            .add_systems(
                OnEnter(GameScreen::MainMenu),
                (world::cleanup_room, audio::stop_chapter_music),
            )
            .add_systems(
                OnEnter(GameScreen::Dead),
                (world::cleanup_room, audio::stop_chapter_music),
            )
            .add_systems(
                OnEnter(GameScreen::Complete),
                (audio::stop_chapter_music, audio::queue_victory_cue),
            )
            .add_systems(
                Update,
                (
                    interface::update_letterbox,
                    screens::update_screen_text,
                    screens::handle_screen_input,
                    interface::update_room_clear_overlay,
                    interface::update_boss_intro_overlay,
                    interface::update_room_transition_overlay,
                    interface::update_menu_modal,
                    bonus_screen::update_bonus_screen,
                    camera::update_background_parallax,
                    interface::update_hud,
                    ammo_hud::update_ammo_hud,
                ),
            )
            .add_systems(
                Update,
                (
                    camera::follow_camera,
                    player::update_player,
                    combat::player_gun,
                    combat::update_player_bullets,
                    combat::update_enemy_ai,
                    combat::update_enemy_platform_physics,
                    combat::update_projectiles,
                    combat::update_damage_numbers,
                    combat::update_dead_enemies,
                    combat::update_impact_effects,
                    combat::update_health_bars,
                    combat::update_health_pickups,
                    combat::update_player_death,
                    flow::handle_door_touch,
                    flow::room_clear_check,
                )
                    .chain()
                    .run_if(in_state(GameScreen::Playing)),
            )
            .add_systems(
                Update,
                flow::update_room_clear.run_if(in_state(GameScreen::RoomClear)),
            )
            .add_systems(
                Update,
                player_model::animate_player_model
                    .after(combat::player_gun)
                    .after(room_intro::pose_room_intro)
                    .run_if(in_state(GameScreen::Playing).or_else(in_state(GameScreen::RoomIntro))),
            )
            .add_systems(
                Update,
                flow::handle_bonus_input.run_if(in_state(GameScreen::Bonus)),
            )
            .add_systems(
                Update,
                (
                    room_intro::update_room_intro.run_if(in_state(GameScreen::RoomIntro)),
                    room_intro::pose_room_intro.run_if(
                        in_state(GameScreen::RoomLoading)
                            .or_else(in_state(GameScreen::TransitioningOut))
                            .or_else(in_state(GameScreen::RoomIntro)),
                    ),
                )
                    .chain(),
            )
            .add_systems(
                Update,
                flow::update_boss_intro.run_if(in_state(GameScreen::BossIntro)),
            )
            .add_systems(
                Update,
                flow::update_room_transition.run_if(in_state(GameScreen::Transitioning)),
            )
            .add_systems(
                Update,
                flow::animate_exit_door.run_if(in_state(GameScreen::Transitioning)),
            )
            .add_systems(
                Update,
                flow::update_room_transition_fade_out
                    .run_if(in_state(GameScreen::TransitioningOut)),
            )
            .add_systems(
                Update,
                assets::animate_sprites
                    .run_if(in_state(GameScreen::Playing).or_else(in_state(GameScreen::RoomIntro))),
            )
            .add_systems(
                PostUpdate,
                (audio::update_audio_volume, audio::play_sound_queue),
            );
    }
}

#[derive(States, Debug, Clone, Copy, Default, Eq, PartialEq, Hash)]
enum GameScreen {
    #[default]
    MainMenu,
    /// Room entrance: the player walks in, then the enemies appear.
    RoomIntro,
    Playing,
    Paused,
    RoomClear,
    Bonus,
    BossIntro,
    Transitioning,
    RoomLoading,
    TransitioningOut,
    Settings,
    Controls,
    Dead,
    Complete,
}

#[derive(Resource, Debug)]
struct RunSession {
    room_id: u16,
    health: f32,
    max_health: f32,
    rerolls: u8,
    owned_bonuses: Vec<String>,
    offered_bonuses: Vec<String>,
    previous_offers: BTreeSet<String>,
    selected_choice: bonus_screen::BonusChoice,
    bonus_reroll_animation_remaining_s: f32,
    door_open: bool,
    room_ward_used: bool,
    world_loaded: bool,
}

impl RunSession {
    fn has_bonus(&self, id: &str) -> bool {
        self.owned_bonuses.iter().any(|owned| owned == id)
    }

    /// An owned bonus's multiplier effect, or 1 without the bonus.
    fn bonus_multiplier(&self, data: &GameData, id: &str, effect: &str) -> f32 {
        if self.has_bonus(id) {
            data.bonus_effect(id, effect)
        } else {
            1.0
        }
    }

    /// Rounds a full magazine holds, with Extended Magazine.
    fn magazine_capacity(&self, data: &GameData) -> u32 {
        let extra = if self.has_bonus("extended_magazine") {
            data.bonus_effect("extended_magazine", "extra_magazine_rounds") as u32
        } else {
            0
        };
        data.gameplay.player.gun.magazine_size + extra
    }
}

impl Default for RunSession {
    fn default() -> Self {
        Self {
            room_id: 1,
            health: 100.0,
            max_health: 100.0,
            rerolls: 3,
            owned_bonuses: Vec::new(),
            offered_bonuses: Vec::new(),
            previous_offers: BTreeSet::new(),
            selected_choice: bonus_screen::BonusChoice::Offer(0),
            bonus_reroll_animation_remaining_s: 0.0,
            door_open: false,
            room_ward_used: false,
            world_loaded: false,
        }
    }
}

#[derive(Resource, Default)]
struct SpriteSheets(BTreeMap<String, SpriteSheet>);

#[derive(Clone)]
struct SpriteSheet {
    image: Handle<Image>,
    layout: Handle<TextureAtlasLayout>,
    frame_size: Vec2,
    frames: usize,
    fps: f32,
}

#[derive(Component)]
struct AnimatedSprite {
    sheet: String,
    frame: usize,
    frames: usize,
    timer: Timer,
    looping: bool,
}

#[derive(Component)]
struct ScreenLabel;

#[derive(Component)]
struct HudText;

#[derive(Component)]
struct HealthFill;

#[derive(Component)]
struct HealthBarFrame;

#[derive(Component)]
struct RoomClearOverlay;

#[derive(Component)]
struct BossIntroOverlay;

#[derive(Component)]
struct RoomTransitionOverlay;

#[derive(Component)]
struct RoomEntity;

#[derive(Component)]
struct PlayerBody {
    velocity: Vec2,
    facing: f32,
    grounded: bool,
    jumps_used: u8,
    /// Rising from a jump whose button is still held; releasing it cuts the rise.
    jump_held: bool,
    dash_timer: f32,
    dash_cooldown: f32,
    dash_direction: f32,
    /// Seconds until the gun can fire again.
    fire_cooldown: f32,
    /// Rounds left in the magazine.
    ammo: u32,
    reload: Option<Reload>,
    /// Shot variant the next shot plays; consecutive shots alternate.
    shot_variant: usize,
    /// Direction of the last shot, held by the aim pose.
    aim: Aim,
    /// Seconds the aim pose keeps holding after a shot.
    aim_hold_timer: f32,
    /// Health Vampiric Rounds restored from the current magazine.
    magazine_healing: f32,
    crouching: bool,
    covering: bool,
    /// Sprinter bonus: running faster since a grounded dash.
    sprinting: bool,
    /// Seconds spent against the current wall while falling.
    wall_contact_s: f32,
    /// Seconds spent standing idle, for the hand fidgets.
    idle_s: f32,
    /// The next fidget uses the right hand.
    fidget_right: bool,
    /// Seconds since the player died, while the death animation plays.
    dying_s: Option<f32>,
    invulnerability_timer: f32,
    dash_invulnerability_timer: f32,
    dash_shot_timer: f32,
    rhythm_timer: f32,
    rhythm_stacks: u8,
    chain_reaction_timer: f32,
    sated_fury_timer: f32,
    /// One-shot animation playing over the movement clips.
    action: Option<PlayerAction>,
    /// Seconds left on `action`; it only grows when an action (re)starts.
    action_animation_timer: f32,
    /// `action` (re)started since the animation last showed it.
    action_started: bool,
    footstep_timer: f32,
    /// Side (-1 left, 1 right) of the wall an airborne player is touching.
    wall_side: Option<f32>,
    wall_jump_lock_timer: f32,
    drop_through_timer: f32,
    /// Standing on a one-way platform, which Down + Jump drops through.
    on_one_way: bool,
    mantle: Option<Mantle>,
}

impl Default for PlayerBody {
    fn default() -> Self {
        Self {
            velocity: Vec2::ZERO,
            facing: 1.0,
            grounded: true,
            jumps_used: 0,
            jump_held: false,
            dash_timer: 0.0,
            dash_cooldown: 0.0,
            dash_direction: 1.0,
            fire_cooldown: 0.0,
            ammo: 0,
            reload: None,
            shot_variant: 0,
            aim: Aim::Straight,
            aim_hold_timer: 0.0,
            magazine_healing: 0.0,
            crouching: false,
            covering: false,
            sprinting: false,
            wall_contact_s: 0.0,
            idle_s: 0.0,
            fidget_right: false,
            dying_s: None,
            invulnerability_timer: 0.0,
            dash_invulnerability_timer: 0.0,
            dash_shot_timer: 0.0,
            rhythm_timer: 0.0,
            rhythm_stacks: 0,
            chain_reaction_timer: 0.0,
            sated_fury_timer: 0.0,
            action: None,
            action_animation_timer: 0.0,
            action_started: false,
            footstep_timer: 0.0,
            wall_side: None,
            wall_jump_lock_timer: 0.0,
            drop_through_timer: 0.0,
            on_one_way: false,
            mantle: None,
        }
    }
}

impl PlayerBody {
    /// Starts a one-shot animation lasting `duration_s`.
    fn start_action(&mut self, action: PlayerAction, duration_s: f32) {
        self.action = Some(action);
        self.action_animation_timer = duration_s;
        self.action_started = true;
    }
}

/// A one-shot player animation played over the movement clips.
#[derive(Clone, Copy, Debug, PartialEq)]
enum PlayerAction {
    /// Firing; a straight shot while running plays the run-shooting cycle.
    Shot {
        aim: Aim,
        variant: usize,
        running: bool,
    },
    Hit,
    Land,
    Turn {
        running: bool,
    },
    WallJump,
}

/// A magazine refill under way.
#[derive(Clone, Copy, Debug)]
struct Reload {
    remaining_s: f32,
    duration_s: f32,
}

/// An automatic climb onto a ledge, between two body-center positions.
#[derive(Clone, Copy)]
struct Mantle {
    from: Vec2,
    to: Vec2,
    elapsed_s: f32,
}

#[derive(Component)]
struct EnemyBody {
    type_id: EnemyTypeId,
    health: f32,
    max_health: f32,
    movement_speed_h_s: f32,
    home_y: f32,
    feet_offset: f32,
    visible_top_offset: f32,
    vertical_velocity: f32,
    grounded: bool,
    jump_height_h: f32,
    jump_cooldown: f32,
    jump: Option<EnemyJump>,
    drop_through_timer: f32,
    behavior: String,
    procedural_movement: Option<ProceduralMovementState>,
    preferred_distance_h: Option<(f32, f32)>,
    attacks: Vec<EnemyAttack>,
    engaged: bool,
    attack_cooldown: f32,
    windup_remaining: f32,
    combo_hit_pending: bool,
    active_attack: usize,
    next_attack: usize,
    hurt_timer: f32,
    knockback_velocity: f32,
    phase_two: Option<(f32, f32)>,
    phase_two_announced: bool,
    marked_timer: f32,
    movement_sound_timer: f32,
}

#[derive(Clone, Copy)]
struct EnemyJump {
    velocity_x: f32,
    slam: bool,
    landed: bool,
}

struct ProceduralMovementState {
    phase_radians: f32,
    bob_height_h: f32,
    cycles_per_h: f32,
    lean_radians: f32,
}

#[derive(Clone)]
struct EnemyAttack {
    kind: String,
    damage: f32,
    hits: u8,
    reach_h: f32,
    projectile_speed_h_s: f32,
    landing_radius_h: f32,
    cooldown_s: f32,
    windup_s: f32,
}

impl From<&EnemyAttackConfig> for EnemyAttack {
    fn from(config: &EnemyAttackConfig) -> Self {
        Self {
            kind: config.kind.clone(),
            damage: config.damage.or(config.damage_per_hit).unwrap_or(0.0),
            hits: config.hits.unwrap_or(1),
            reach_h: config.reach_h.or(config.landing_radius_h).unwrap_or(1.2),
            projectile_speed_h_s: config.projectile_speed_h_s.unwrap_or(0.0),
            landing_radius_h: config.landing_radius_h.unwrap_or(0.0),
            cooldown_s: config.cooldown_s,
            windup_s: config.windup_s,
        }
    }
}

#[derive(Component)]
struct DeadEnemy {
    timer: Timer,
}

#[derive(Component)]
struct CombatProjectile {
    velocity_x: f32,
    damage: f32,
    life_s: f32,
    remaining_range: f32,
    half_size: Vec2,
}

/// A piece of a fiery burst (its hot core or a flying ember): it drifts,
/// swells by `growth`, cools from yellow to red, and fades out.
#[derive(Component)]
struct ImpactEffect {
    timer: Timer,
    velocity: Vec2,
    growth: f32,
    color: Color,
}

/// A bullet fired by the player; it stops at the first enemy or solid block.
#[derive(Component)]
struct PlayerBullet {
    velocity: Vec2,
    damage: f32,
    remaining_range: f32,
    half_size: Vec2,
    /// The last round of a magazine, which plays the heavier hit sound.
    last_round: bool,
    /// Where the shooter's body is: the first step traces from here, so an
    /// enemy pressed against the player is hit point-blank.
    fired_from: Option<Vec2>,
}

#[derive(Component)]
struct DamageNumber {
    timer: Timer,
    color: Color,
}

#[derive(Component)]
struct EnemyHealthBar {
    enemy: Entity,
    max_health: f32,
}

#[derive(Component)]
struct EnemyHealthBarBackground {
    enemy: Entity,
}

#[derive(Component)]
struct HealthPickup {
    healing: f32,
}

#[derive(Component)]
struct ExitDoor;

#[derive(Resource)]
struct RandomState(u64);

impl Default for RandomState {
    fn default() -> Self {
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(1, |duration| duration.as_nanos() as u64);
        Self(seed.max(1))
    }
}

impl RandomState {
    fn next_u64(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }

    fn index(&mut self, length: usize) -> usize {
        (self.next_u64() as usize) % length
    }

    /// A uniform value in `0.0..1.0`.
    fn fraction(&mut self) -> f32 {
        (self.next_u64() >> 40) as f32 / (1u64 << 24) as f32
    }

    fn chance(&mut self, probability: f32) -> bool {
        (self.next_u64() as f64 / u64::MAX as f64) < f64::from(probability)
    }
}

#[derive(Resource)]
struct PhaseTimer(Timer);

#[derive(Component)]
struct BackgroundParallax {
    base_x: f32,
    factor: f32,
}

/// The room the camera is confined to, in H.
#[derive(Component)]
struct RoomCamera {
    size_h: Vec2,
}

impl RoomCamera {
    /// `center_h` moved just enough to keep the viewport inside the room.
    fn clamp_center_h(&self, center_h: Vec2, gameplay: &crate::data::GameplayConfig) -> Vec2 {
        let half_view_h = Vec2::new(gameplay.viewport_width_h(), gameplay.viewport.height_h) / 2.0;
        center_h.clamp(half_view_h, (self.size_h - half_view_h).max(half_view_h))
    }
}

#[derive(Debug)]
struct SpriteSheetSpec {
    key: String,
    path: String,
    frame_width: u32,
    frame_height: u32,
    frames: usize,
    fps: f32,
    /// Grid cell (column, row) of the clip's first frame within its sheet.
    first_cell: UVec2,
}

fn move_towards(current: f32, target: f32, max_delta: f32) -> f32 {
    if (target - current).abs() <= max_delta {
        target
    } else {
        current + (target - current).signum() * max_delta
    }
}

fn set_animation(
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
    fn idle_sheet(&self) -> &'static str {
        match &**self {
            "swordsman" => "enemy_swordsman_idle",
            "infantryman" => "enemy_infantry_idle",
            "destroyer" => "enemy_destroyer_idle",
            "gorgon_1" => "enemy_gorgon_1_idle",
            "gorgon_2" => "enemy_gorgon_2_idle",
            "gorgon_3" => "enemy_gorgon_3_idle",
            "evil_wizard" => "boss_idle",
            _ => "enemy_swordsman_idle",
        }
    }
}
