use std::collections::{BTreeMap, BTreeSet};

use bevy::{
    audio::{AddAudioSource, AudioPlugin},
    ecs::system::SystemParam,
    prelude::*,
};

use crate::data::{
    DifficultyColors, EnemyAttackConfig, EnemyDetectionConfig, EnemyMaterial, EnemyTypeId,
    FootstepSound, GameData, HexColor, PLAYER_COLLISION_WIDTH_H,
};

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
mod impact;
mod interact;
mod interface;
mod menus;
mod pause_blur;
mod player;
mod prologue;
mod random;
mod ripple;
mod room_art;
mod room_intro;
mod room_pan;
mod room_shot;
mod screens;
mod session;
mod settings;
mod shaders;
mod sprites;
mod ui_style;
mod visual_walkthrough;
mod world;

pub use room_shot::RoomShotPlugin;
pub use visual_walkthrough::{VisualWalkthroughPlugin, WalkthroughScope};

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
use interact::{InReach, Interactable};
use menus::{MenuAction, MenuFocus, VolumeChannel};
use player::{BonusAura, PlayerAction, PlayerBody, Strike};
use prologue::Prologue;
use random::RandomState;
use room_pan::RoomPan;
use session::{Difficulty, RunSession};
use settings::{AudioSettings, PreferencesDir, SettingsReturn};
use shaders::{CityLightsMaterial, Glowing, ShaderAssets};
use sprites::{AnimatedSprite, SpriteSheet, SpriteSheetSpec, SpriteSheets, set_animation};
use world::{EntryDoor, ExitDoor, RoomEntity};

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
            .add_plugins(ripple::RipplePlugin)
            .register_asset_loader(audio::MusicStreamLoader)
            .init_state::<GameScreen>()
            .init_resource::<RunSession>()
            .init_resource::<RoomPan>()
            .init_resource::<Prologue>()
            .init_resource::<StartRoom>()
            .init_resource::<AudioSettings>()
            .init_resource::<PreferencesDir>()
            .init_resource::<SettingsReturn>()
            .init_resource::<MenuFocus>()
            .init_resource::<InReach>()
            .add_message::<menus::MenuPressed>()
            .init_resource::<ControlBindings>()
            .init_resource::<SoundQueue>()
            .init_resource::<audio::SoundBank>()
            .init_resource::<RandomState>()
            .insert_resource(PhaseTimer(Timer::from_seconds(0.5, TimerMode::Once)))
            .add_systems(
                Startup,
                (
                    ui_style::use_the_game_font,
                    camera::spawn_camera,
                    interface::spawn_interface,
                    bonus_screen::spawn_bonus_screen,
                    menus::spawn_menu,
                    prologue::spawn_prologue_text,
                    assets::load_sprites,
                    settings::initialize_settings,
                    controls::initialize_control_bindings,
                    (audio::initialize_sound_bank, audio::start_intro_music).chain(),
                ),
            )
            .add_systems(
                OnEnter(GameScreen::Playing),
                (
                    world::spawn_room,
                    audio::start_chapter_music,
                    audio::start_run_ambience,
                ),
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
                    audio::start_run_ambience,
                ),
            )
            .add_systems(
                OnEnter(GameScreen::BossIntro),
                (flow::start_boss_intro, audio::queue_boss_intro),
            )
            .add_systems(
                OnEnter(GameScreen::RoomLoading),
                (
                    room_pan::send_off_the_departing_room,
                    world::spawn_room,
                    room_pan::start_room_pan,
                )
                    .chain(),
            )
            .add_systems(
                OnEnter(GameScreen::MainMenu),
                (
                    world::cleanup_room,
                    audio::stop_run_audio,
                    audio::start_intro_music,
                ),
            )
            .add_systems(
                OnEnter(GameScreen::Transitioning),
                audio::fade_out_intro_music,
            )
            .add_systems(OnEnter(GameScreen::Prologue), prologue::start_prologue)
            .add_systems(
                Update,
                (
                    prologue::update_prologue.run_if(in_state(GameScreen::Prologue)),
                    prologue::drain_the_veil.run_if(not(in_state(GameScreen::Prologue))),
                    prologue::show_prologue_text,
                ),
            )
            .add_systems(
                OnEnter(GameScreen::Dead),
                (
                    world::cleanup_room,
                    audio::stop_run_audio,
                    audio::start_intro_music,
                ),
            )
            .add_systems(
                OnEnter(GameScreen::Complete),
                (audio::stop_run_audio, audio::queue_victory_cue),
            )
            .add_systems(
                Update,
                (
                    interface::update_letterbox,
                    screens::update_screen_text,
                    (
                        menus::navigate_menus,
                        screens::handle_screen_input,
                        menus::update_menu,
                        (menus::highlight_menu_buttons, menus::update_volume_bars),
                    )
                        .chain(),
                    interface::update_room_clear_overlay,
                    interface::update_boss_intro_overlay,
                    interface::update_room_transition_overlay,
                    interface::update_menu_modal,
                    bonus_screen::update_bonus_screen,
                    camera::update_background_parallax,
                    interface::update_hud,
                    shaders::attach_glows,
                    interact::show_the_interact_badge,
                    shaders::show_the_flood,
                ),
            )
            .add_systems(
                Update,
                (camera::follow_camera, player::update_player)
                    .chain()
                    .before(combat::plant_decoy)
                    .run_if(
                        in_state(GameScreen::Playing)
                            .or_else(in_state(GameScreen::RoomClear))
                            .or_else(in_state(GameScreen::Bonus)),
                    ),
            )
            .add_systems(
                Update,
                (
                    // Before the melee and the AI: it takes the spot the roll
                    // `update_player` just set off from, and the AI then marks
                    // the double it left standing there.
                    combat::plant_decoy,
                    combat::player_melee,
                    combat::update_enemy_ai,
                    combat::update_enemy_platform_physics,
                    combat::update_falls,
                    (combat::update_projectiles, combat::update_hazards).chain(),
                    // A tuple holds twenty systems at most: what a fight
                    // leaves on screen is one of them.
                    (
                        combat::update_damage_numbers,
                        combat::update_dead_enemies,
                        combat::update_impact_effects,
                        combat::update_health_bars,
                        combat::update_enemy_shields,
                        combat::update_health_pickups,
                        combat::update_healing_over_time,
                    )
                        .chain(),
                    combat::update_player_death,
                    (combat::send_in_the_next_wave, combat::raise_the_arriving).chain(),
                    bonus_pickup::spawn_bonus_pickup,
                    interact::mark_what_is_in_reach,
                    bonus_pickup::update_bonus_pickup,
                    (
                        bonus_pickup::promise_the_next_bonus,
                        shaders::light_the_unlocked_door,
                    )
                        .chain(),
                    flow::handle_door_touch,
                    flow::room_clear_check,
                )
                    .chain()
                    .run_if(in_state(GameScreen::Playing)),
            )
            // On every screen, so a freeze a blow began always ends, whatever
            // the screen has turned to meanwhile.
            .add_systems(
                Update,
                impact::feel_the_impacts.before(camera::follow_camera),
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
                    (room_intro::pose_room_intro, room_intro::open_the_entry_door).run_if(
                        in_state(GameScreen::RoomLoading)
                            .or_else(in_state(GameScreen::Panning))
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
                (flow::animate_exit_door, room_pan::step_into_the_door)
                    .run_if(in_state(GameScreen::Transitioning)),
            )
            .add_systems(
                Update,
                room_pan::update_room_pan.run_if(in_state(GameScreen::Panning)),
            )
            .add_systems(
                Update,
                flow::update_room_transition_fade_out
                    .run_if(in_state(GameScreen::TransitioningOut)),
            )
            .add_systems(
                Update,
                (assets::animate_sprites, combat::update_hit_effects).run_if(
                    in_state(GameScreen::Playing)
                        .or_else(in_state(GameScreen::RoomIntro))
                        .or_else(in_state(GameScreen::Transitioning))
                        .or_else(in_state(GameScreen::Panning))
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
    /// Choosing how hard the run should be, on the way out of the main menu.
    Difficulty,
    /// Room entrance: the player walks in, then the enemies appear.
    RoomIntro,
    Playing,
    Paused,
    RoomClear,
    Bonus,
    BossIntro,
    Transitioning,
    /// The words the city is entered on, written over the water.
    Prologue,
    RoomLoading,
    /// The camera sliding out of one room and into the next.
    Panning,
    TransitioningOut,
    Settings,
    Controls,
    Dead,
    Complete,
    /// Debug room editor (debug builds only).
    #[cfg(debug_assertions)]
    Editor,
}

impl GameScreen {
    /// The screens the player stands in the room on: playing it, walking into
    /// it, or crossing the door out of it. Crossing a door is still being in
    /// the run, so the HUD stays up rather than blinking off and back on.
    pub(crate) fn shows_the_hud(self) -> bool {
        matches!(
            self,
            Self::RoomIntro
                | Self::Playing
                | Self::Transitioning
                | Self::RoomLoading
                | Self::Panning
        )
    }
}

fn move_towards(current: f32, target: f32, max_delta: f32) -> f32 {
    if (target - current).abs() <= max_delta {
        target
    } else {
        current + (target - current).signum() * max_delta
    }
}
