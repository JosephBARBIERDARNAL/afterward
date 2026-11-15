use super::*;
use crate::game::{
    controls::TypedLetters, interface::ScreenLabel, menus::MenuPressed, prologue::Prologue,
};
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
        GameScreen::BossIntro => String::from("CHAPTER ONE\n\nTHE BLACK KNIGHT"),
        // A room led into by a door says nothing: the camera carries the
        // player there in the open. The door out of the chapter is the one
        // worth announcing.
        GameScreen::Transitioning | GameScreen::TransitioningOut => data
            .rooms
            .get(&session.room_id)
            .filter(|room| data.rooms.contains_key(&room.next_room_id))
            .map_or_else(|| String::from("CHAPTER ONE COMPLETE"), |_| String::new()),
        // Menus are drawn as cards by `menus::update_menu`.
        GameScreen::MainMenu
        | GameScreen::Difficulty
        | GameScreen::Paused
        | GameScreen::ConfirmLeave
        | GameScreen::Settings
        | GameScreen::Controls
        | GameScreen::Dead
        | GameScreen::Complete
        | GameScreen::RoomIntro
        | GameScreen::Playing
        | GameScreen::RoomLoading
        | GameScreen::Panning
        | GameScreen::Prologue
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
    random: ResMut<'w, RandomState>,
    settings: ResMut<'w, PlayerSettings>,
    preferences: Res<'w, PreferencesDir>,
    controls: ResMut<'w, ControlBindings>,
    settings_return: ResMut<'w, SettingsReturn>,
    prologue: ResMut<'w, Prologue>,
    pending_leave: ResMut<'w, PendingLeave>,
    sounds: ResMut<'w, SoundQueue>,
}

/// Carries out what the player asked for: a button they pressed, with the
/// mouse or the confirm key, or a letter shortcut typed straight at the
/// screen. Both arrive here as a [`MenuAction`], so the two never drift apart.
pub(super) fn handle_screen_input(
    resources: ScreenInputResources,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    mut keyboard: MessageReader<KeyboardInput>,
    mut presses: MessageReader<MenuPressed>,
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
        mut random,
        mut settings,
        preferences,
        mut controls,
        mut settings_return,
        mut prologue,
        mut pending_leave,
        mut sounds,
    } = resources;
    let screen = *screen.get();
    let pausing = controls.just_pressed(GameAction::Pause, &keys);

    // The controls screen, while it listens, belongs to the key being bound.
    if screen == GameScreen::Controls && controls.rebinding.is_some() {
        if keys.just_pressed(KeyCode::Escape) {
            controls.rebinding = None;
            sounds.push(SoundCue::UiCancel);
        } else if controls.capture_key(&keys) {
            controls::save_bindings(&controls, &preferences);
            sounds.push(SoundCue::UiConfirm);
        }
        return;
    }

    // Pausing and resuming belong to no button.
    match screen {
        GameScreen::Playing if pausing => {
            next_screen.set(GameScreen::Paused);
            sounds.push(SoundCue::UiCancel);
            return;
        }
        GameScreen::Paused if pausing => {
            next_screen.set(GameScreen::Playing);
            sounds.push(SoundCue::UiConfirm);
            return;
        }
        _ => {}
    }

    // The letters and keys that press a button from anywhere on its screen,
    // without walking to it first.
    let letter = |letter| typed.contains(letter);
    let shortcut = match screen {
        GameScreen::MainMenu | GameScreen::Paused if letter('s') => Some(MenuAction::OpenSettings),
        GameScreen::MainMenu | GameScreen::Paused if letter('q') => Some(MenuAction::Quit),
        #[cfg(debug_assertions)]
        GameScreen::MainMenu | GameScreen::Paused if letter('d') => Some(MenuAction::OpenEditor),
        GameScreen::Paused if letter('m') => Some(MenuAction::LeaveRun),
        GameScreen::Paused if letter('r') => Some(MenuAction::Restart),
        // The letter that asked confirms it.
        GameScreen::ConfirmLeave
            if letter('r') && pending_leave.0 == Some(MenuAction::Restart)
                || letter('m') && pending_leave.0 == Some(MenuAction::LeaveRun) =>
        {
            Some(MenuAction::ConfirmLeave)
        }
        GameScreen::Settings if letter('c') => Some(MenuAction::OpenControls),
        GameScreen::Settings if controls.just_pressed(GameAction::ToggleFullscreen, &keys) => {
            Some(MenuAction::ToggleFullscreen)
        }
        GameScreen::Controls if letter('r') => Some(MenuAction::RestoreDefaultBindings),
        GameScreen::Settings
        | GameScreen::Controls
        | GameScreen::Difficulty
        | GameScreen::ConfirmLeave
            if pausing =>
        {
            Some(MenuAction::Back)
        }
        GameScreen::Dead | GameScreen::Complete if keys.just_pressed(KeyCode::Escape) => {
            Some(MenuAction::Back)
        }
        _ => None,
    };

    for press in presses
        .read()
        .copied()
        .chain(shortcut.map(|action| MenuPressed {
            action,
            along_bar: None,
        }))
    {
        // Confirming carries out what was asked for on the pause menu.
        let action = match press.action {
            MenuAction::ConfirmLeave => match pending_leave.0.take() {
                Some(action) => action,
                None => continue,
            },
            action => action,
        };
        match action {
            // Restarting or leaving gives the run up, so the pause menu asks
            // first.
            MenuAction::Restart | MenuAction::LeaveRun if screen == GameScreen::Paused => {
                pending_leave.0 = Some(action);
                next_screen.set(GameScreen::ConfirmLeave);
                sounds.push(SoundCue::UiConfirm);
            }
            MenuAction::BeginRun => {
                // The run begins on the difficulty screen, not in a room.
                next_screen.set(GameScreen::Difficulty);
                sounds.push(SoundCue::UiConfirm);
            }
            MenuAction::ChooseDifficulty(level) => {
                session.difficulty = level;
                next_screen.set(reset_run(&mut session, &data, *start_room, &mut random));
                sounds.push(SoundCue::UiConfirm);
            }
            MenuAction::EnterTheCity => {
                // The first press finishes writing the words; the next one
                // walks into what they are about.
                if prologue.confirm(&data.gameplay.prologue.lines) {
                    next_screen.set(GameScreen::RoomLoading);
                    sounds.push(SoundCue::UiConfirm);
                }
            }
            MenuAction::Continue => {
                next_screen.set(GameScreen::Playing);
                sounds.push(SoundCue::UiConfirm);
            }
            MenuAction::Restart => {
                // The room and the run's music are taken down on the way out
                // of `ConfirmLeave` (see `GamePlugin`).
                next_screen.set(reset_run(&mut session, &data, *start_room, &mut random));
                sounds.push(SoundCue::RunRestarted);
            }
            MenuAction::OpenSettings => {
                // Settings goes back to whichever menu opened it.
                *settings_return = SettingsReturn(screen);
                next_screen.set(GameScreen::Settings);
                sounds.push(SoundCue::UiConfirm);
            }
            MenuAction::OpenControls => {
                next_screen.set(GameScreen::Controls);
                sounds.push(SoundCue::UiConfirm);
            }
            #[cfg(debug_assertions)]
            MenuAction::OpenEditor => {
                // From the main menu the editor opens the room runs start in;
                // from the pause menu, the room being played.
                if screen == GameScreen::MainMenu {
                    session.room_id = start_room.0;
                }
                next_screen.set(GameScreen::Editor);
                sounds.push(SoundCue::UiConfirm);
            }
            #[cfg(not(debug_assertions))]
            MenuAction::OpenEditor => {}
            MenuAction::LeaveRun => {
                next_screen.set(GameScreen::MainMenu);
                sounds.push(SoundCue::UiCancel);
            }
            MenuAction::Quit => {
                exit.write(AppExit::Success);
            }
            // Turned into what it confirms above.
            MenuAction::ConfirmLeave => {}
            MenuAction::Volume(channel) => {
                // Only a click that landed inside the bar moves it; one on the
                // row's name just walks the focus there.
                if let Some(along_bar) = press.along_bar {
                    settings.set_volume(channel, along_bar);
                    settings::save_settings(&settings, &preferences);
                    sounds.push(SoundCue::UiMove);
                }
            }
            MenuAction::ToggleEnemyMarkers => {
                settings.enemy_markers = !settings.enemy_markers;
                settings::save_settings(&settings, &preferences);
                sounds.push(SoundCue::UiConfirm);
            }
            MenuAction::ToggleFullscreen => {
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
            MenuAction::Rebind(action) => {
                controls.rebinding = Some(action);
                sounds.push(SoundCue::UiConfirm);
            }
            MenuAction::RestoreDefaultBindings => {
                controls.restore_defaults(&preferences);
                sounds.push(SoundCue::UiConfirm);
            }
            MenuAction::Back => {
                next_screen.set(match screen {
                    GameScreen::Settings => settings_return.0,
                    GameScreen::Controls => GameScreen::Settings,
                    GameScreen::ConfirmLeave => GameScreen::Paused,
                    _ => GameScreen::MainMenu,
                });
                sounds.push(SoundCue::UiCancel);
            }
        }
    }
}

/// Resets the run to `start_room` and returns the screen that opens it: the
/// room entrance, like arriving there through a door. Everything the run held
/// is dropped — only what the player chose before it (the difficulty, and the
/// editor's playtest) carries over — so nothing of the run given up, a Last
/// Stand spent or a trickle of healing included, follows the player in.
pub(super) fn reset_run(
    session: &mut RunSession,
    data: &GameData,
    start_room: StartRoom,
    random: &mut RandomState,
) -> GameScreen {
    *session = RunSession {
        room_id: start_room.0,
        playtest: session.playtest,
        difficulty: session.difficulty,
        max_health: data.gameplay.player.max_health,
        rerolls: data.gameplay.room_flow.bonus_rerolls_per_run,
        ..default()
    };
    session.grant_starting_bonuses(data, random);
    GameScreen::RoomIntro
}
