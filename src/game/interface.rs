use super::*;
use bevy::{camera::Viewport, text::FontSize, window::PrimaryWindow};

pub(super) fn spawn_interface(mut commands: Commands) {
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: px(0),
            right: px(0),
            top: px(0),
            bottom: px(0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.78, 0.86, 1.0, 0.72)),
        Visibility::Hidden,
        RoomClearOverlay,
        Pickable::IGNORE,
    ));

    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: px(0),
            right: px(0),
            top: px(0),
            bottom: px(0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.0)),
        Visibility::Hidden,
        RoomTransitionOverlay,
        Pickable::IGNORE,
    ));

    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: px(0),
            right: px(0),
            top: px(0),
            bottom: px(0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.02, 0.025, 0.05, 0.0)),
        Visibility::Hidden,
        BossIntroOverlay,
        Pickable::IGNORE,
    ));

    commands
        .spawn((
            Node {
                width: percent(100),
                height: percent(100),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            Pickable::IGNORE,
        ))
        .with_children(|parent| {
            let mut label = parent.spawn((
                Text::new("AFTERWARD"),
                TextFont {
                    font_size: FontSize::Px(36.0),
                    ..default()
                },
                TextColor(Color::WHITE),
                ScreenLabel,
            ));
            label.with_children(|parent| {
                for index in 0..2 {
                    parent.spawn((
                        TextSpan::default(),
                        BonusScreenSpan {
                            index,
                            kind: BonusSpanKind::Category,
                        },
                        TextColor(Color::WHITE),
                    ));
                    parent.spawn((
                        TextSpan::default(),
                        BonusScreenSpan {
                            index,
                            kind: BonusSpanKind::Name,
                        },
                        TextColor(Color::WHITE),
                    ));
                    parent.spawn((
                        TextSpan::default(),
                        BonusScreenSpan {
                            index,
                            kind: BonusSpanKind::Description,
                        },
                        TextColor(Color::WHITE),
                    ));
                }
                parent.spawn((
                    TextSpan::default(),
                    BonusScreenSpan {
                        index: 2,
                        kind: BonusSpanKind::Reroll,
                    },
                    TextColor(Color::WHITE),
                ));
            });
        });

    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: px(24),
            top: px(20),
            ..default()
        },
        Text::new(""),
        TextFont {
            font_size: FontSize::Px(20.0),
            ..default()
        },
        TextColor(Color::WHITE),
        HudText,
    ));

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(24),
                top: px(54),
                width: px(240),
                height: px(16),
                ..default()
            },
            BackgroundColor(Color::srgb(0.07, 0.08, 0.12)),
            HealthBarFrame,
            Pickable::IGNORE,
        ))
        .with_children(|parent| {
            parent.spawn((
                Node {
                    width: percent(100),
                    height: percent(100),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.77, 0.18, 0.26)),
                HealthFill,
            ));
        });
}

pub(super) fn update_letterbox(
    windows: Query<&Window, With<PrimaryWindow>>,
    mut cameras: Query<&mut Camera, With<Camera2d>>,
    data: Res<GameData>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let Ok(mut camera) = cameras.single_mut() else {
        return;
    };

    let physical = window.physical_size();
    if physical.x == 0 || physical.y == 0 {
        return;
    }
    let aspect = f32::from(data.gameplay.viewport.aspect_ratio.width)
        / f32::from(data.gameplay.viewport.aspect_ratio.height);
    let available_aspect = physical.x as f32 / physical.y as f32;
    let (width, height) = if available_aspect > aspect {
        (physical.y as f32 * aspect, physical.y as f32)
    } else {
        (physical.x as f32, physical.x as f32 / aspect)
    };
    let size = UVec2::new(width.round() as u32, height.round() as u32);
    let position = UVec2::new((physical.x - size.x) / 2, (physical.y - size.y) / 2);
    camera.viewport = Some(Viewport {
        physical_position: position,
        physical_size: size,
        ..default()
    });
}

pub(super) fn update_hud(
    screen: Res<State<GameScreen>>,
    session: Res<RunSession>,
    mut text_query: Query<&mut Text, With<HudText>>,
    mut fill_query: Query<&mut Node, With<HealthFill>>,
    mut bar_visibility: Query<&mut Visibility, With<HealthBarFrame>>,
) {
    let text_value = if *screen.get() == GameScreen::Playing {
        format!(
            "ROOM {:02}    HP {:03.0}/{:03.0}",
            session.room_id, session.health, session.max_health
        )
    } else {
        String::new()
    };
    for mut text in &mut text_query {
        text.0 = text_value.clone();
    }
    let health_fraction = if session.max_health > 0.0 {
        session.health / session.max_health
    } else {
        0.0
    };
    for mut node in &mut fill_query {
        node.width = percent((health_fraction.clamp(0.0, 1.0) * 100.0).round() as u32);
    }
    for mut visibility in &mut bar_visibility {
        *visibility = if *screen.get() == GameScreen::Playing {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

pub(super) fn update_room_clear_overlay(
    screen: Res<State<GameScreen>>,
    timer: Res<PhaseTimer>,
    mut overlay: Query<(&mut BackgroundColor, &mut Visibility), With<RoomClearOverlay>>,
) {
    for (mut color, mut visibility) in &mut overlay {
        if *screen.get() == GameScreen::RoomClear {
            let opacity = 0.72 * (1.0 - timer.0.fraction());
            color.0 = Color::srgba(0.78, 0.86, 1.0, opacity);
            *visibility = Visibility::Visible;
        } else {
            *visibility = Visibility::Hidden;
        }
    }
}

pub(super) fn update_boss_intro_overlay(
    screen: Res<State<GameScreen>>,
    timer: Res<PhaseTimer>,
    mut overlay: Query<(&mut BackgroundColor, &mut Visibility), With<BossIntroOverlay>>,
) {
    for (mut color, mut visibility) in &mut overlay {
        if *screen.get() == GameScreen::BossIntro {
            let progress = timer.0.fraction();
            let fade_in = (progress / 0.25).clamp(0.0, 1.0);
            let fade_out = ((1.0 - progress) / 0.75).clamp(0.0, 1.0);
            color.0 = Color::srgba(0.02, 0.025, 0.05, 0.72 * fade_in.min(fade_out));
            *visibility = Visibility::Visible;
        } else {
            *visibility = Visibility::Hidden;
        }
    }
}

pub(super) fn update_room_transition_overlay(
    screen: Res<State<GameScreen>>,
    timer: Res<PhaseTimer>,
    mut overlay: Query<(&mut BackgroundColor, &mut Visibility), With<RoomTransitionOverlay>>,
) {
    let opacity = match screen.get() {
        GameScreen::Transitioning => Some(timer.0.fraction()),
        GameScreen::RoomLoading => Some(1.0),
        GameScreen::TransitioningOut => Some(1.0 - timer.0.fraction()),
        _ => None,
    };
    for (mut color, mut visibility) in &mut overlay {
        if let Some(opacity) = opacity {
            color.0 = Color::srgba(0.0, 0.0, 0.0, opacity.clamp(0.0, 1.0));
            *visibility = Visibility::Visible;
        } else {
            *visibility = Visibility::Hidden;
        }
    }
}
