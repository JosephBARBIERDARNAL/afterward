use super::*;
use crate::game::ui_style::load_pixel_art;
use bevy::{camera::Viewport, text::FontSize, window::PrimaryWindow};

/// Blur behind the bonus screen and the menus, so their cards stand out.
const MODAL_BLUR_RADIUS_PX: f32 = 18.0;

/// The health bar's frame, nine-sliced around the fill: its outer
/// `HEALTH_FRAME_BORDER_PX` image pixels are the frame, drawn
/// `HEALTH_FRAME_SCALE` times larger.
const HEALTH_FRAME_IMAGE: &str = "ui/health_frame.png";
const HEALTH_FRAME_BORDER_PX: f32 = 3.0;
const HEALTH_FRAME_SCALE: f32 = 2.0;

pub(super) fn spawn_interface(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: px(0),
            right: px(0),
            top: px(0),
            bottom: px(0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.015, 0.02, 0.04, 0.0)),
        Visibility::Hidden,
        MenuBackdrop,
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
            parent.spawn((
                Text::new(""),
                TextFont {
                    font_size: FontSize::Px(32.0),
                    ..default()
                },
                TextLayout::justify(Justify::Center),
                TextColor(Color::WHITE),
                ScreenLabel,
            ));
        });

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(24),
                top: px(20),
                width: px(320),
                height: px(24),
                ..default()
            },
            ImageNode::new(load_pixel_art(&asset_server, HEALTH_FRAME_IMAGE)).with_mode(
                NodeImageMode::Sliced(TextureSlicer {
                    border: BorderRect::all(HEALTH_FRAME_BORDER_PX),
                    max_corner_scale: HEALTH_FRAME_SCALE,
                    ..default()
                }),
            ),
            HealthBarFrame,
            Pickable::IGNORE,
        ))
        .with_children(|parent| {
            // The trough inside the frame. UI images draw inside their node's
            // padding, so the fill gets its own inset node instead.
            let inset = px(HEALTH_FRAME_BORDER_PX * HEALTH_FRAME_SCALE);
            parent
                .spawn(Node {
                    position_type: PositionType::Absolute,
                    left: inset,
                    right: inset,
                    top: inset,
                    bottom: inset,
                    ..default()
                })
                .with_child((
                    Node {
                        width: percent(100),
                        height: percent(100),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.77, 0.18, 0.26)),
                    HealthFill,
                ));
            // The health readout, centered over the whole bar.
            parent.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    width: percent(100),
                    height: percent(100),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                Text::new(""),
                TextFont {
                    font_size: FontSize::Px(14.0),
                    ..default()
                },
                TextLayout::justify(Justify::Center),
                TextColor(Color::WHITE),
                TextShadow {
                    offset: Vec2::ONE,
                    color: Color::srgba(0.0, 0.0, 0.0, 0.8),
                },
                HealthText,
            ));
        });
}

#[derive(Component)]
pub(super) struct MenuBackdrop;

pub(super) fn update_menu_modal(
    screen: Res<State<GameScreen>>,
    mut backdrop: Query<(&mut BackgroundColor, &mut Visibility), With<MenuBackdrop>>,
    mut camera_blur: Query<&mut pause_blur::PauseBlur, With<Camera2d>>,
) {
    let is_modal = matches!(
        screen.get(),
        GameScreen::MainMenu
            | GameScreen::Paused
            | GameScreen::Settings
            | GameScreen::Controls
            | GameScreen::Dead
            | GameScreen::Complete
    );
    let is_blurred = is_modal || *screen.get() == GameScreen::Bonus;

    for (mut color, mut visibility) in &mut backdrop {
        if is_blurred {
            color.0 = Color::srgba(0.015, 0.02, 0.04, 0.36);
            *visibility = Visibility::Visible;
        } else {
            *visibility = Visibility::Hidden;
        }
    }
    for mut blur in &mut camera_blur {
        blur.radius_pixels = if is_blurred {
            MODAL_BLUR_RADIUS_PX
        } else {
            0.0
        };
    }
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
    mut text_query: Query<&mut Text, With<HealthText>>,
    mut fill_query: Query<&mut Node, With<HealthFill>>,
    mut bar_visibility: Query<&mut Visibility, With<HealthBarFrame>>,
) {
    for mut text in &mut text_query {
        text.0 = format!("{:.0}/{:.0}", session.health, session.max_health);
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
        *visibility = if matches!(screen.get(), GameScreen::RoomIntro | GameScreen::Playing) {
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

#[derive(Component)]
pub(crate) struct ScreenLabel;

#[derive(Component)]
pub(crate) struct HealthText;

#[derive(Component)]
pub(crate) struct HealthFill;

#[derive(Component)]
pub(crate) struct HealthBarFrame;

#[derive(Component)]
pub(crate) struct RoomClearOverlay;

#[derive(Component)]
pub(crate) struct BossIntroOverlay;

#[derive(Component)]
pub(crate) struct RoomTransitionOverlay;
