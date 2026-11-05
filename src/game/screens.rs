use super::*;
use crate::game::interface::ScreenLabel;
use bevy::window::{MonitorSelection, PrimaryWindow, WindowMode};

/// Menu line for the debug room editor, shown in debug builds only.
const EDITOR_MENU_LINE: &str = if cfg!(debug_assertions) {
    "\nD  Debug editor"
} else {
    ""
};

pub(super) fn update_screen_text(
    screen: Res<State<GameScreen>>,
    session: Res<RunSession>,
    data: Res<GameData>,
    settings: Res<AudioSettings>,
    controls: Res<ControlBindings>,
    mut labels: Query<&mut Text, With<ScreenLabel>>,
) {
    let message = match screen.get() {
        GameScreen::MainMenu => format!("AFTERWARD\n\nENTER  Begin run{EDITOR_MENU_LINE}"),
        GameScreen::Paused => format!(
            "PAUSED\n\nENTER  Continue\nS  Settings\nR  Restart\nM  Main menu{EDITOR_MENU_LINE}"
        ),
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
        GameScreen::Settings => format!(
            "SETTINGS\n\n{} MUSIC  {}%\n{} SOUND  {}%\n\nMENU UP/DOWN  Select\nMOVE LEFT/RIGHT  Adjust\nINTERACT  Controls\n{} FULLSCREEN\nPAUSE  Back",
            if settings.selected_slider == 0 {
                ">"
            } else {
                " "
            },
            settings.music_volume_percent,
            if settings.selected_slider == 1 {
                ">"
            } else {
                " "
            },
            settings.sfx_volume_percent,
            controls.binding_label(GameAction::ToggleFullscreen),
        ),
        GameScreen::Controls => {
            let mut message = String::from("CONTROLS\n\n");
            for (index, action) in GameAction::ALL.iter().enumerate() {
                let marker = if controls.selected_action_index == index {
                    "> "
                } else {
                    "  "
                };
                message.push_str(&format!(
                    "{marker}{}  {}\n",
                    action.label(),
                    controls.binding_label(*action)
                ));
            }
            if controls.waiting_for_key {
                message.push_str("\nPRESS A KEY TO BIND  (ESC cancels)");
            } else {
                message.push_str(
                    "\nMENU UP/DOWN  Select   INTERACT  Remap\nR  Restore defaults   PAUSE  Back",
                );
            }
            message
        }
        GameScreen::Dead => String::from("RUN ENDED\n\nENTER  Return to menu"),
        GameScreen::Complete => String::from("CHAPTER ONE CLEARED\n\nENTER  Return to menu"),
        GameScreen::RoomIntro | GameScreen::Playing | GameScreen::Bonus => String::new(),
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
) {
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
        GameScreen::MainMenu | GameScreen::Paused if keys.just_pressed(KeyCode::KeyD) => {
            // From the main menu the editor opens the room runs start in;
            // from the pause menu, the room being played.
            if *screen.get() == GameScreen::MainMenu {
                session.room_id = start_room.0;
            }
            next_screen.set(GameScreen::Editor);
            sounds.push(SoundCue::UiConfirm);
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
        GameScreen::Paused if keys.just_pressed(KeyCode::KeyS) => {
            next_screen.set(GameScreen::Settings);
            sounds.push(SoundCue::UiConfirm);
        }
        GameScreen::Paused if keys.just_pressed(KeyCode::KeyM) => {
            next_screen.set(GameScreen::MainMenu);
            sounds.push(SoundCue::UiCancel);
        }
        GameScreen::Paused if keys.just_pressed(KeyCode::KeyR) => {
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
            if keys.just_pressed(KeyCode::KeyR) {
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
    session.offered_bonuses.clear();
    session.previous_offers.clear();
    session.selected_choice = bonus_screen::BonusChoice::Offer(0);
    session.bonus_reroll_animation_remaining_s = 0.0;
    session.room_ward_used = false;
    session.door_open = false;
    session.world_loaded = false;
    GameScreen::RoomIntro
}
