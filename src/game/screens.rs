use super::*;
use crate::game::{controls::TypedLetters, interface::ScreenLabel};
use bevy::window::{MonitorSelection, PrimaryWindow, WindowMode};
use bevy::{app::AppExit, input::keyboard::KeyboardInput};

pub(super) fn update_screen_text(
    screen: Res<State<GameScreen>>,
    session: Res<RunSession>,
    data: Res<GameData>,
    mut labels: Query<&mut Text, With<ScreenLabel>>,
) {
    let message = match screen.get() {
        GameScreen::RoomClear => String::from("ROOM CLEARED"),
        GameScreen::BossIntro => String::from("CHAPTER ONE\n\nTHE EVIL WIZARD"),
        GameScreen::Transitioning => data
            .rooms
            .get(&session.room_id)
            .filter(|room| data.rooms.contains_key(&room.next_room_id))
            .map_or_else(
                || String::from("CHAPTER ONE COMPLETE"),
                |room| format!("ENTERING ROOM {:02}", room.next_room_id),
            ),
        GameScreen::RoomLoading | GameScreen::TransitioningOut => data
            .rooms
            .get(&session.room_id)
            .filter(|room| data.rooms.contains_key(&room.next_room_id))
            .map_or_else(
                || String::from("CHAPTER ONE COMPLETE"),
                |_| format!("ROOM {:02}", session.room_id),
            ),
        // Menus are drawn as cards by `menus::update_menu`.
        GameScreen::MainMenu
        | GameScreen::Paused
        | GameScreen::Settings
        | GameScreen::Controls
        | GameScreen::Dead
        | GameScreen::Complete
        | GameScreen::RoomIntro
        | GameScreen::Playing
        | GameScreen::Bonus => String::new(),
        #[cfg(debug_assertions)]
        GameScreen::Editor => String::new(),
    };
    for mut text in &mut labels {
        text.0 = message.clone();
    }
}

#[derive(SystemParam)]
pub(super) struct ScreenInputResources<'w> {
    keys: Res<'w, ButtonInput<KeyCode>>,
    screen: Res<'w, State<GameScreen>>,
    next_screen: ResMut<'w, NextState<GameScreen>>,
    data: Res<'w, GameData>,
    start_room: Res<'w, StartRoom>,
    session: ResMut<'w, RunSession>,
    settings: ResMut<'w, AudioSettings>,
    preferences: Res<'w, PreferencesDir>,
    controls: ResMut<'w, ControlBindings>,
    sounds: ResMut<'w, SoundQueue>,
}

pub(super) fn handle_screen_input(
    mut commands: Commands,
    resources: ScreenInputResources,
    room_entities: Query<Entity, With<RoomEntity>>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    mut keyboard: MessageReader<KeyboardInput>,
    mut exit: MessageWriter<AppExit>,
) {
    let typed: TypedLetters = keyboard.read().collect();
    let ScreenInputResources {
        keys,
        screen,
        mut next_screen,
        data,
        start_room,
        mut session,
        mut settings,
        preferences,
        mut controls,
        mut sounds,
    } = resources;
    match screen.get() {
        GameScreen::MainMenu if controls.just_pressed(GameAction::Interact, &keys) => {
            next_screen.set(reset_run(&mut session, &data, *start_room));
            sounds.push(SoundCue::UiConfirm);
        }
        #[cfg(debug_assertions)]
        GameScreen::MainMenu | GameScreen::Paused if typed.contains('d') => {
            // From the main menu the editor opens the room runs start in;
            // from the pause menu, the room being played.
            if *screen.get() == GameScreen::MainMenu {
                session.room_id = start_room.0;
            }
            next_screen.set(GameScreen::Editor);
            sounds.push(SoundCue::UiConfirm);
        }
        GameScreen::MainMenu | GameScreen::Paused if typed.contains('q') => {
            exit.write(AppExit::Success);
        }
        GameScreen::Playing if controls.just_pressed(GameAction::Pause, &keys) => {
            next_screen.set(GameScreen::Paused);
            sounds.push(SoundCue::UiCancel);
        }
        GameScreen::Paused
            if keys.just_pressed(KeyCode::Enter)
                || controls.just_pressed(GameAction::Pause, &keys) =>
        {
            next_screen.set(GameScreen::Playing);
            sounds.push(SoundCue::UiConfirm);
        }
        GameScreen::Paused if typed.contains('s') => {
            next_screen.set(GameScreen::Settings);
            sounds.push(SoundCue::UiConfirm);
        }
        GameScreen::Paused if typed.contains('m') => {
            next_screen.set(GameScreen::MainMenu);
            sounds.push(SoundCue::UiCancel);
        }
        GameScreen::Paused if typed.contains('r') => {
            for entity in &room_entities {
                commands.entity(entity).despawn();
            }
            next_screen.set(reset_run(&mut session, &data, *start_room));
            sounds.push(SoundCue::RunRestarted);
        }
        GameScreen::Settings if controls.just_pressed(GameAction::Pause, &keys) => {
            next_screen.set(GameScreen::Paused);
            sounds.push(SoundCue::UiCancel);
        }
        GameScreen::Settings => {
            if controls.just_pressed(GameAction::NavigateUp, &keys)
                || controls.just_pressed(GameAction::NavigateDown, &keys)
            {
                settings.selected_slider = 1 - settings.selected_slider;
                sounds.push(SoundCue::UiMove);
            }
            if controls.just_pressed(GameAction::MoveLeft, &keys)
                || controls.just_pressed(GameAction::MoveRight, &keys)
            {
                let increase = controls.just_pressed(GameAction::MoveRight, &keys);
                let volume = if settings.selected_slider == 0 {
                    &mut settings.music_volume_percent
                } else {
                    &mut settings.sfx_volume_percent
                };
                *volume = if increase {
                    volume.saturating_add(5).min(100)
                } else {
                    volume.saturating_sub(5)
                };
                settings::save_settings(&settings, &preferences);
                sounds.push(SoundCue::UiMove);
            }
            if controls.just_pressed(GameAction::Interact, &keys) {
                next_screen.set(GameScreen::Controls);
                sounds.push(SoundCue::UiConfirm);
            }
            if controls.just_pressed(GameAction::ToggleFullscreen, &keys) {
                if let Ok(mut window) = windows.single_mut() {
                    if matches!(window.mode, WindowMode::BorderlessFullscreen(_)) {
                        window.mode = WindowMode::Windowed;
                        window.resizable = data.gameplay.window.resizable_windowed;
                    } else {
                        window.mode = WindowMode::BorderlessFullscreen(MonitorSelection::Current);
                    }
                }
                sounds.push(SoundCue::UiConfirm);
            }
        }
        GameScreen::Controls if controls.waiting_for_key => {
            if keys.just_pressed(KeyCode::Escape) {
                controls.cancel_rebinding();
                sounds.push(SoundCue::UiCancel);
            } else if controls.capture_key(&keys) {
                controls::save_bindings(&controls, &preferences);
                sounds.push(SoundCue::UiConfirm);
            }
        }
        GameScreen::Controls if controls.just_pressed(GameAction::Pause, &keys) => {
            next_screen.set(GameScreen::Settings);
            sounds.push(SoundCue::UiCancel);
        }
        GameScreen::Controls => {
            if controls.just_pressed(GameAction::NavigateUp, &keys) {
                controls.move_selection(-1);
                sounds.push(SoundCue::UiMove);
            } else if controls.just_pressed(GameAction::NavigateDown, &keys) {
                controls.move_selection(1);
                sounds.push(SoundCue::UiMove);
            }
            if controls.just_pressed(GameAction::Interact, &keys) {
                controls.begin_rebinding();
                sounds.push(SoundCue::UiConfirm);
            }
            if typed.contains('r') {
                controls.restore_defaults(&preferences);
                sounds.push(SoundCue::UiConfirm);
            }
        }
        GameScreen::Dead | GameScreen::Complete
            if keys.just_pressed(KeyCode::Enter)
                || keys.just_pressed(KeyCode::Escape)
                || controls.just_pressed(GameAction::Interact, &keys) =>
        {
            next_screen.set(GameScreen::MainMenu);
            sounds.push(SoundCue::UiConfirm);
        }
        _ => {}
    }
}

/// Resets the run to `start_room` and returns the screen that opens it: the
/// room entrance, like arriving there through a door.
pub(super) fn reset_run(
    session: &mut RunSession,
    data: &GameData,
    start_room: StartRoom,
) -> GameScreen {
    session.room_id = start_room.0;
    session.max_health = data.gameplay.player.max_health;
    session.health = session.max_health;
    session.rerolls = data.gameplay.room_flow.bonus_rerolls_per_run;
    session.owned_bonuses.clear();
    session.bonus_category = None;
    session.offered_bonuses.clear();
    session.previous_offers.clear();
    session.selected_choice = bonus_screen::BonusChoice::Offer(0);
    session.bonus_reroll_animation_remaining_s = 0.0;
    session.room_ward_used = false;
    session.room_cleared = false;
    session.door_open = false;
    session.world_loaded = false;
    GameScreen::RoomIntro
}
