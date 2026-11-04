use super::*;
use bevy::text::FontSize;

const LOADED_ROUND: Color = Color::srgb(0.98, 0.8, 0.36);
const RELOADING_ROUND: Color = Color::srgba(0.98, 0.8, 0.36, 0.45);
const SPENT_ROUND: Color = Color::srgba(0.32, 0.28, 0.22, 0.75);

/// Bottom-right ammo readout: the rounds left over the magazine size, and one
/// pip per round that empties as the player fires and refills while reloading.
#[derive(Component)]
pub(super) struct AmmoHud;

#[derive(Component)]
pub(super) struct AmmoCount;

/// The pip row, with the magazine size its pips were built for.
#[derive(Component)]
pub(super) struct AmmoPips {
    capacity: u32,
}

#[derive(Component)]
pub(super) struct AmmoPip(u32);

pub(super) fn spawn_ammo_hud(mut commands: Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                right: px(24),
                bottom: px(20),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::FlexEnd,
                row_gap: px(4),
                ..default()
            },
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
                AmmoPips { capacity: 0 },
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
    mut pips: Query<(&AmmoPip, &mut BackgroundColor)>,
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
        .filter(|_| matches!(screen.get(), GameScreen::RoomIntro | GameScreen::Playing));
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
            format!("{} / {capacity}", player.ammo)
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
                                width: px(5),
                                height: px(14),
                                border_radius: BorderRadius::all(px(1)),
                                ..default()
                            },
                            BackgroundColor(SPENT_ROUND),
                            AmmoPip(index),
                        ));
                    }
                });
        }
    }
    for (pip, mut background) in &mut pips {
        background.0 = if pip.0 < filled { color } else { SPENT_ROUND };
    }
}
