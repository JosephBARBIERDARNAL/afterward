use std::{
    collections::{BTreeMap, BTreeSet},
    time::{SystemTime, UNIX_EPOCH},
};

use bevy::{ecs::system::SystemParam, prelude::*};

use crate::data::{EnemyAttackConfig, EnemyTypeId, GameData, PlatformMaterial};

mod assets;
mod audio;
mod camera;
mod combat;
mod controls;
mod flow;
mod interface;
mod pause_blur;
mod player;
mod screens;
mod settings;
mod visual_walkthrough;
mod world;

pub use visual_walkthrough::VisualWalkthroughPlugin;

#[cfg(test)]
mod tests;

use audio::{SoundCue, SoundQueue};
use controls::{ControlBindings, GameAction};
use settings::AudioSettings;

const H_WORLD_UNITS: f32 = 48.0;
const PLAYER_COLLISION_WIDTH_H: f32 = 0.6;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(pause_blur::PauseBlurPlugin)
            .init_state::<GameScreen>()
            .init_resource::<RunSession>()
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
                    assets::load_sprites,
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
                OnEnter(GameScreen::BossIntro),
                (
                    world::spawn_room,
                    flow::start_boss_intro,
                    audio::queue_boss_intro,
                ),
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
                    camera::update_background_parallax,
                    interface::update_hud,
                ),
            )
            .add_systems(
                Update,
                (
                    camera::follow_camera,
                    player::update_player,
                    combat::player_attacks,
                    combat::update_enemy_ai,
                    combat::update_enemy_platform_physics,
                    combat::update_projectiles,
                    combat::update_damage_numbers,
                    combat::update_dead_enemies,
                    combat::update_health_bars,
                    combat::update_health_pickups,
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
                flow::handle_bonus_input.run_if(in_state(GameScreen::Bonus)),
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
                assets::animate_sprites.run_if(in_state(GameScreen::Playing)),
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
    selected_option: usize,
    bonus_reroll_animation_remaining_s: f32,
    door_open: bool,
    room_ward_used: bool,
    world_loaded: bool,
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
            selected_option: 0,
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
struct BonusScreenSpan {
    index: usize,
    kind: BonusSpanKind,
}

#[derive(Clone, Copy)]
enum BonusSpanKind {
    Category,
    Name,
    Description,
    Reroll,
}

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
    weapon_mode: PlayerWeapon,
    grounded: bool,
    jumps_used: u8,
    dash_timer: f32,
    dash_cooldown: f32,
    dash_direction: f32,
    sword_cooldown: f32,
    gun_cooldown: f32,
    invulnerability_timer: f32,
    dash_invulnerability_timer: f32,
    dash_strike_timer: f32,
    stationary_timer: f32,
    rhythm_timer: f32,
    rhythm_stacks: u8,
    chain_reaction_timer: f32,
    sated_fury_timer: f32,
    action_animation_timer: f32,
    footstep_timer: f32,
}

#[derive(Clone, Copy)]
enum PlayerWeapon {
    Sword,
    Gun,
}

#[derive(Component)]
struct EnemyBody {
    type_id: EnemyTypeId,
    health: f32,
    max_health: f32,
    movement_speed_h_s: f32,
    home_y: f32,
    visual_height: f32,
    visible_top_offset: f32,
    vertical_velocity: f32,
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
    knockback_h_s: f32,
    life_s: f32,
    remaining_range: f32,
    friendly: bool,
    half_size: Vec2,
}

#[derive(Component)]
struct DamageNumber {
    timer: Timer,
    received: bool,
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

    fn chance(&mut self, probability: f32) -> bool {
        (self.next_u64() as f64 / u64::MAX as f64) < f64::from(probability)
    }
}

#[derive(Resource)]
struct PhaseTimer(Timer);

#[derive(Component)]
struct PlatformSurface {
    left: f32,
    right: f32,
    bottom: f32,
    top: f32,
    material: PlatformMaterial,
}

impl PlatformSurface {
    fn intersects_projectile(&self, start: Vec2, end: Vec2, half_size: Vec2) -> bool {
        let projectile_left = start.x.min(end.x) - half_size.x;
        let projectile_right = start.x.max(end.x) + half_size.x;
        let projectile_bottom = start.y.min(end.y) - half_size.y;
        let projectile_top = start.y.max(end.y) + half_size.y;

        projectile_left <= self.right
            && projectile_right >= self.left
            && projectile_bottom <= self.top
            && projectile_top >= self.bottom
    }
}

#[derive(Component)]
struct BackgroundParallax {
    base_x: f32,
    factor: f32,
}

#[derive(Component)]
struct RoomCamera {
    width_h: f32,
    fixed_y_h: f32,
    follows_player: bool,
}

#[derive(Debug)]
struct SpriteSheetSpec {
    key: &'static str,
    path: &'static str,
    frame_width: u32,
    frame_height: u32,
    frames: usize,
    fps: f32,
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
            "warrior" => "boss_idle",
            _ => "enemy_swordsman_idle",
        }
    }
}
