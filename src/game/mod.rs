use std::collections::{BTreeMap, BTreeSet};

use bevy::{
    audio::{AddAudioSource, AudioPlugin},
    ecs::system::SystemParam,
    prelude::*,
};

use crate::data::{
    Aim, EnemyAttackConfig, EnemyDetectionConfig, EnemyMaterial, EnemyTypeId, FootstepSound,
    GameData, PLAYER_COLLISION_WIDTH_H,
};

mod ammo_hud;
mod assets;
mod audio;
mod bonus_pickup;
mod bonus_screen;
mod camera;
mod collision;
mod combat;
mod controls;
#[cfg(debug_assertions)]
mod editor;
mod flow;
#[cfg(debug_assertions)]
mod hitbox_overlay;
mod interface;
mod menus;
mod pause_blur;
mod player;
mod random;
mod room_art;
mod room_intro;
mod screens;
mod session;
mod settings;
mod shaders;
mod sprites;
mod ui_style;
mod visual_walkthrough;
mod world;

pub use visual_walkthrough::VisualWalkthroughPlugin;

#[cfg(test)]
mod tests;

use audio::{DamageSound, SoundCue, SoundQueue};
use camera::{BackgroundParallax, RoomCamera};
use collision::{HALF_BODY_WIDTH, PlatformSurface, Platforms};
use combat::{
    EnemyAttack, EnemyBody, EnemyHealthBar, EnemyHealthBarBackground, EnemyShieldIcon,
    ProceduralMovementState,
};
use controls::{ControlBindings, GameAction};
use flow::PhaseTimer;
use player::{PlayerAction, PlayerBody, Reload};
use random::RandomState;
use session::RunSession;
use settings::{AudioSettings, PreferencesDir};
use shaders::{CityLightsMaterial, Glowing, ShaderAssets};
use sprites::{AnimatedSprite, SpriteSheet, SpriteSheetSpec, SpriteSheets, set_animation};
use world::{ExitDoor, RoomEntity};

const H_WORLD_UNITS: f32 = 48.0;
/// Horizontal speed, in world units per second, above which a body counts as moving.
const MOVING_SPEED_THRESHOLD: f32 = 20.0;

pub struct GamePlugin;

/// Room a new run starts in. Defaults to the empty opening room 0; `--room
/// <id>` overrides it for debugging, and restarts from the pause menu return
/// to the same room.
#[derive(Resource, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct StartRoom(pub u16);

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
        #[cfg(debug_assertions)]
        app.add_plugins(editor::EditorPlugin)
            .init_resource::<hitbox_overlay::HitboxOverlay>()
            .add_systems(
                Update,
                (
                    hitbox_overlay::toggle_hitbox_overlay,
                    hitbox_overlay::draw_hitbox_overlay,
                )
                    .chain(),
            );
        // The playback systems `add_audio_source` registers need the audio
        // output of `AudioPlugin`; headless tests run without it.
        if app.is_plugin_added::<AudioPlugin>() {
            app.add_audio_source::<audio::MusicStream>();
        } else {
            app.init_asset::<audio::MusicStream>();
        }
        app.add_plugins((pause_blur::PauseBlurPlugin, shaders::ShaderPlugin))
            .register_asset_loader(audio::MusicStreamLoader)
            .init_state::<GameScreen>()
            .init_resource::<RunSession>()
            .init_resource::<StartRoom>()
            .init_resource::<AudioSettings>()
            .init_resource::<PreferencesDir>()
            .init_resource::<ControlBindings>()
            .init_resource::<SoundQueue>()
            .init_resource::<audio::SoundBank>()
            .init_resource::<RandomState>()
            .insert_resource(PhaseTimer(Timer::from_seconds(0.5, TimerMode::Once)))
            .add_systems(
                Startup,
                (
                    camera::spawn_camera,
                    interface::spawn_interface,
                    ammo_hud::spawn_ammo_hud,
                    bonus_screen::spawn_bonus_screen,
                    menus::spawn_menu,
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
                    menus::update_menu,
                    camera::update_background_parallax,
                    interface::update_hud,
                    ammo_hud::update_ammo_hud,
                    shaders::attach_glows,
                    shaders::show_menu_water,
                ),
            )
            .add_systems(
                Update,
                (camera::follow_camera, player::update_player)
                    .chain()
                    .before(combat::player_gun)
                    .run_if(
                        in_state(GameScreen::Playing)
                            .or_else(in_state(GameScreen::RoomClear))
                            .or_else(in_state(GameScreen::Bonus)),
                    ),
            )
            .add_systems(
                Update,
                (
                    combat::player_gun,
                    combat::update_player_bullets,
                    combat::update_enemy_ai,
                    combat::update_enemy_platform_physics,
                    combat::update_falls,
                    combat::update_projectiles,
                    combat::update_damage_numbers,
                    combat::update_dead_enemies,
                    combat::update_impact_effects,
                    combat::update_health_bars,
                    combat::update_enemy_shields,
                    combat::update_health_pickups,
                    combat::update_player_death,
                    bonus_pickup::spawn_bonus_pickup,
                    bonus_pickup::update_bonus_pickup,
                    shaders::light_the_unlocked_door,
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
                assets::animate_sprites.run_if(
                    in_state(GameScreen::Playing)
                        .or_else(in_state(GameScreen::RoomIntro))
                        .or_else(in_state(GameScreen::RoomClear))
                        .or_else(in_state(GameScreen::Bonus)),
                ),
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
    /// Debug room editor (debug builds only).
    #[cfg(debug_assertions)]
    Editor,
}

fn move_towards(current: f32, target: f32, max_delta: f32) -> f32 {
    if (target - current).abs() <= max_delta {
        target
    } else {
        current + (target - current).signum() * max_delta
    }
}
