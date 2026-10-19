use super::*;
use bevy::window::{MonitorSelection, PrimaryWindow, WindowMode};

pub(super) fn update_screen_text(
    screen: Res<State<GameScreen>>,
    session: Res<RunSession>,
    data: Res<GameData>,
    settings: Res<AudioSettings>,
    controls: Res<ControlBindings>,
    mut labels: Query<&mut Text, With<ScreenLabel>>,
    mut bonus_spans: Query<(&BonusScreenSpan, &mut TextSpan, &mut TextColor)>,
) {
    let message = match screen.get() {
        GameScreen::MainMenu => String::from("AFTERWARD\n\nENTER  Begin run"),
        GameScreen::Paused => {
            String::from("PAUSED\n\nENTER  Continue\nS  Settings\nR  Restart\nM  Main menu")
        }
        GameScreen::RoomClear => String::from("ROOM CLEARED"),
        GameScreen::Bonus => format!(
            "CHOOSE A BONUS\n\n{} / {}  Select    {}  Confirm\n\n",
            controls.binding_label(GameAction::NavigateUp),
            controls.binding_label(GameAction::NavigateDown),
            controls.binding_label(GameAction::Interact),
        ),
        GameScreen::BossIntro => String::from("CHAPTER ONE\n\nTHE WARRIOR"),
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
        GameScreen::Playing => String::new(),
    };
    for mut text in &mut labels {
        text.0 = message.clone();
    }
    for (span, mut text, mut color) in &mut bonus_spans {
        let (content, span_color) = if *screen.get() != GameScreen::Bonus {
            (String::new(), Color::WHITE)
        } else if matches!(span.kind, BonusSpanKind::Reroll) {
            if session.bonus_reroll_animation_remaining_s > 0.0 {
                let dots = (session.bonus_reroll_animation_remaining_s * 12.0) as usize % 4;
                (
                    format!(
                        "@  REROLLING{}  ({} remaining)\n",
                        ".".repeat(dots),
                        session.rerolls
                    ),
                    Color::WHITE,
                )
            } else {
                let marker = if session.selected_option == 2 {
                    ">"
                } else {
                    " "
                };
                if flow::reroll_available(&session, &data) {
                    (
                        format!("{marker} 3. @ REROLL  ({} remaining)\n", session.rerolls),
                        Color::WHITE,
                    )
                } else {
                    ("  3. REROLL UNAVAILABLE\n".to_owned(), Color::WHITE)
                }
            }
        } else if session.bonus_reroll_animation_remaining_s > 0.0 {
            (String::new(), Color::WHITE)
        } else if let Some(bonus_id) = session.offered_bonuses.get(span.index) {
            if let Some(bonus) = data
                .bonuses
                .bonuses
                .iter()
                .find(|bonus| bonus.id == *bonus_id)
            {
                let selected = session.selected_option == span.index;
                match span.kind {
                    BonusSpanKind::Category => {
                        if let Some(category) = data.bonuses.categories.get(&bonus.category) {
                            let marker = if selected { ">" } else { " " };
                            let icon = category.icon.to_string();
                            let mut characters = bonus.category.chars();
                            let category_name = characters
                                .next()
                                .map(|first| {
                                    first.to_uppercase().chain(characters).collect::<String>()
                                })
                                .unwrap_or_default();
                            (
                                format!("{marker} {}. {icon} {category_name}\n", span.index + 1),
                                Color::from(category.color),
                            )
                        } else {
                            (String::new(), Color::WHITE)
                        }
                    }
                    BonusSpanKind::Name => (format!("{}\n", bonus.name), Color::WHITE),
                    BonusSpanKind::Description => {
                        (format!("{}\n\n", bonus.description), Color::WHITE)
                    }
                    BonusSpanKind::Reroll => unreachable!(),
                }
            } else {
                (String::new(), Color::WHITE)
            }
        } else {
            (String::new(), Color::WHITE)
        };
        text.0 = content;
        color.0 = span_color;
    }
}

#[derive(SystemParam)]
pub(super) struct ScreenInputResources<'w> {
    keys: Res<'w, ButtonInput<KeyCode>>,
    screen: Res<'w, State<GameScreen>>,
    next_screen: ResMut<'w, NextState<GameScreen>>,
    data: Res<'w, GameData>,
    session: ResMut<'w, RunSession>,
    settings: ResMut<'w, AudioSettings>,
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
        mut session,
        mut settings,
        mut controls,
        mut sounds,
    } = resources;
    match screen.get() {
        GameScreen::MainMenu if controls.just_pressed(GameAction::Interact, &keys) => {
            reset_run(&mut session, &data);
            next_screen.set(GameScreen::Playing);
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
            reset_run(&mut session, &data);
            next_screen.set(GameScreen::Playing);
            sounds.push(SoundCue::UiConfirm);
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
                settings::save_settings(&settings);
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
                controls::save_bindings(&controls);
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
                controls.restore_defaults();
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

fn reset_run(session: &mut RunSession, data: &GameData) {
    session.room_id = 1;
    session.max_health = data.gameplay.player.max_health;
    session.health = session.max_health;
    session.rerolls = data.gameplay.room_flow.bonus_rerolls_per_run;
    session.owned_bonuses.clear();
    session.offered_bonuses.clear();
    session.previous_offers.clear();
    session.selected_option = 0;
    session.bonus_reroll_animation_remaining_s = 0.0;
    session.room_ward_used = false;
    session.door_open = false;
    session.world_loaded = false;
}
