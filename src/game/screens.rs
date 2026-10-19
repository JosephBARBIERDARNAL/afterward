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
        GameScreen::Bonus => bonus_screen_header(&controls),
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
            "SETTINGS\n\n{} MUSIC  {}%\n{} SOUND  {}%\n\nMENU UP/DOWN  Select\nMOVE LEFT/RIGHT  Adjust\nINTERACT  Controls\nF  Fullscreen\nPAUSE  Back",
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
        ),
        GameScreen::Controls => control_screen_text(&controls),
        GameScreen::Dead => String::from("RUN ENDED\n\nENTER  Return to menu"),
        GameScreen::Complete => String::from("CHAPTER ONE CLEARED\n\nENTER  Return to menu"),
        GameScreen::Playing => String::new(),
    };
    for mut text in &mut labels {
        text.0 = message.clone();
    }
    for (span, mut text, mut color) in &mut bonus_spans {
        let (content, span_color) = if *screen.get() == GameScreen::Bonus {
            bonus_span_content(span, &session, &data)
        } else {
            (String::new(), Color::WHITE)
        };
        text.0 = content;
        color.0 = span_color;
    }
}

fn bonus_screen_header(controls: &ControlBindings) -> String {
    format!(
        "CHOOSE A BONUS\n\n{} / {}  Select    {}  Confirm\n\n",
        controls.binding_label(GameAction::NavigateUp),
        controls.binding_label(GameAction::NavigateDown),
        controls.binding_label(GameAction::Interact),
    )
}

fn bonus_span_content(
    span: &BonusScreenSpan,
    session: &RunSession,
    data: &GameData,
) -> (String, Color) {
    if matches!(span.kind, BonusSpanKind::Reroll) {
        if session.bonus_reroll_animation_remaining_s > 0.0 {
            let dots = (session.bonus_reroll_animation_remaining_s * 12.0) as usize % 4;
            return (
                format!(
                    "↻  REROLLING{}  ({} remaining)\n",
                    ".".repeat(dots),
                    session.rerolls
                ),
                Color::WHITE,
            );
        }
        let marker = if session.selected_option == 2 {
            ">"
        } else {
            " "
        };
        return if flow::reroll_available(session, data) {
            (
                format!("{marker} 3. ↻ REROLL  ({} remaining)\n", session.rerolls),
                Color::WHITE,
            )
        } else {
            ("  3. REROLL UNAVAILABLE\n".to_owned(), Color::WHITE)
        };
    }
    if session.bonus_reroll_animation_remaining_s > 0.0 {
        return (String::new(), Color::WHITE);
    }

    let Some(bonus_id) = session.offered_bonuses.get(span.index) else {
        return (String::new(), Color::WHITE);
    };
    let Some(bonus) = data
        .bonuses
        .bonuses
        .iter()
        .find(|bonus| bonus.id == *bonus_id)
    else {
        return (String::new(), Color::WHITE);
    };
    let selected = session.selected_option == span.index;
    match span.kind {
        BonusSpanKind::Category => {
            let Some(category) = data.bonuses.categories.get(&bonus.category) else {
                return (String::new(), Color::WHITE);
            };
            let marker = if selected { ">" } else { " " };
            let icon = category_icon(&category.icon);
            (
                format!(
                    "{marker} {}. {icon} {}\n",
                    span.index + 1,
                    title_case(&bonus.category)
                ),
                parse_hex_color(&category.color).unwrap_or(Color::WHITE),
            )
        }
        BonusSpanKind::Name => (format!("{}\n", bonus.name), Color::WHITE),
        BonusSpanKind::Description => (format!("{}\n\n", bonus.description), Color::WHITE),
        BonusSpanKind::Reroll => unreachable!(),
    }
}

fn title_case(value: &str) -> String {
    let mut characters = value.chars();
    let Some(first) = characters.next() else {
        return String::new();
    };
    first.to_uppercase().chain(characters).collect()
}

fn category_icon(icon: &str) -> &'static str {
    match icon {
        "crossed_swords" => "⚔",
        "shield" => "⬡",
        "wing" => "➤",
        "light" => "✦",
        "knot" => "∞",
        _ => "◆",
    }
}

fn parse_hex_color(value: &str) -> Option<Color> {
    let hex = value.strip_prefix('#').unwrap_or(value).as_bytes();
    if hex.len() != 6 {
        return None;
    }
    let channel = |index| {
        Some(
            u8::from_str_radix(std::str::from_utf8(&hex[index..index + 2]).ok()?, 16).ok()? as f32
                / 255.0,
        )
    };
    Some(Color::srgb(channel(0)?, channel(2)?, channel(4)?))
}

fn control_screen_text(controls: &ControlBindings) -> String {
    let mut message = String::from("CONTROLS\n\n");
    for (index, action) in GameAction::ALL.iter().enumerate() {
        let marker = if controls.selected_index() == index {
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
    if controls.is_waiting_for_key() {
        message.push_str("\nPRESS A KEY TO BIND  (ESC cancels)");
    } else {
        message.push_str(
            "\nMENU UP/DOWN  Select   INTERACT  Remap\nR  Restore defaults   PAUSE  Back",
        );
    }
    message
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
            if keys.just_pressed(KeyCode::KeyF) {
                toggle_fullscreen(&data, &mut windows);
                sounds.push(SoundCue::UiConfirm);
            }
        }
        GameScreen::Controls if controls.is_waiting_for_key() => {
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

fn toggle_fullscreen(data: &GameData, windows: &mut Query<&mut Window, With<PrimaryWindow>>) {
    let Ok(mut window) = windows.single_mut() else {
        return;
    };
    if matches!(window.mode, WindowMode::BorderlessFullscreen(_)) {
        window.mode = WindowMode::Windowed;
        window.resizable = data.gameplay.window.resizable_windowed;
    } else {
        window.mode = WindowMode::BorderlessFullscreen(MonitorSelection::Current);
    }
}
