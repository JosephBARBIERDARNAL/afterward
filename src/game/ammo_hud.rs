use super::*;
use crate::game::ui_style::load_pixel_art;
use bevy::text::FontSize;

/// One round of the pip row: a gray image tinted by the round's state,
/// drawn at twice its 3 x 7 px size.
const ROUND_IMAGE: &str = "ui/round.png";

const LOADED_ROUND: Color = Color::srgb(0.98, 0.8, 0.36);
const RELOADING_ROUND: Color = Color::srgba(0.98, 0.8, 0.36, 0.45);
const SPENT_ROUND: Color = Color::srgba(0.32, 0.28, 0.22, 0.75);

/// Backing panel, so the readout stays legible over any background.
const PANEL_BACKGROUND: Color = Color::srgba(0.02, 0.025, 0.05, 0.5);

/// Ammo readout under the health bar: the rounds left over the magazine size, and one
/// pip per round that empties as the player fires and refills while reloading.
#[derive(Component)]
pub(super) struct AmmoHud;

#[derive(Component)]
pub(super) struct AmmoCount;

/// The pip row, with the magazine size its pips were built for and the
/// round image they show.
#[derive(Component)]
pub(super) struct AmmoPips {
    capacity: u32,
    round: Handle<Image>,
}

#[derive(Component)]
pub(super) struct AmmoPip(u32);

pub(super) fn spawn_ammo_hud(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn((
            Node {
                // Left-aligned with the health bar (interface.rs), 6 px below it.
                position_type: PositionType::Absolute,
                left: px(24),
                top: px(50),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::FlexStart,
                row_gap: px(4),
                padding: UiRect::axes(px(8), px(6)),
                border_radius: BorderRadius::all(px(4)),
                ..default()
            },
            BackgroundColor(PANEL_BACKGROUND),
            Visibility::Hidden,
            AmmoHud,
            Pickable::IGNORE,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(""),
                TextFont {
                    font_size: FontSize::Px(16.0),
                    ..default()
                },
                TextColor(Color::WHITE),
                AmmoCount,
            ));
            parent.spawn((
                Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: px(3),
                    ..default()
                },
                AmmoPips {
                    capacity: 0,
                    round: load_pixel_art(&asset_server, ROUND_IMAGE),
                },
            ));
        });
}

#[derive(SystemParam)]
pub(super) struct AmmoHudResources<'w, 's> {
    screen: Res<'w, State<GameScreen>>,
    data: Res<'w, GameData>,
    session: Res<'w, RunSession>,
    players: Query<'w, 's, &'static PlayerBody>,
}

pub(super) fn update_ammo_hud(
    mut commands: Commands,
    resources: AmmoHudResources,
    mut hud: Query<&mut Visibility, With<AmmoHud>>,
    mut count: Query<&mut Text, With<AmmoCount>>,
    mut rows: Query<(Entity, &mut AmmoPips)>,
    mut pips: Query<(&AmmoPip, &mut ImageNode)>,
) {
    let AmmoHudResources {
        screen,
        data,
        session,
        players,
    } = resources;
    let player = players
        .single()
        .ok()
        .filter(|_| screen.get().shows_the_hud());
    for mut visibility in &mut hud {
        *visibility = if player.is_some() {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
    let Some(player) = player else {
        return;
    };
    let capacity = session.magazine_capacity(&data);
    // While reloading, the pips refill one by one with the reload's progress.
    let (filled, color) = match player.reload {
        Some(reload) => (
            ((1.0 - reload.remaining_s / reload.duration_s) * capacity as f32) as u32,
            RELOADING_ROUND,
        ),
        None => (player.ammo, LOADED_ROUND),
    };
    for mut text in &mut count {
        text.0 = if player.reload.is_some() {
            String::from("RELOADING")
        } else {
            format!("{}/{capacity}", player.ammo)
        };
    }
    for (row, mut pip_row) in &mut rows {
        if pip_row.capacity != capacity {
            pip_row.capacity = capacity;
            commands
                .entity(row)
                .despawn_children()
                .with_children(|parent| {
                    for index in 0..capacity {
                        parent.spawn((
                            Node {
                                width: px(6),
                                height: px(14),
                                ..default()
                            },
                            ImageNode::new(pip_row.round.clone()).with_color(SPENT_ROUND),
                            AmmoPip(index),
                        ));
                    }
                });
        }
    }
    for (pip, mut image) in &mut pips {
        image.color = if pip.0 < filled { color } else { SPENT_ROUND };
    }
}
