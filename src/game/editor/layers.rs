//! The background layers card, shown under the inspector while nothing is
//! selected: every layer of the room's background profiles, sky first then
//! from the furthest to the nearest, each with a thumbnail and a box ticked
//! while the room draws it. Clicking a row hides the layer or shows it again
//! (`hidden_background_layers` of the room file); every layer is drawn until
//! one is hidden. The `-` and `+` beside a layer fade it out or in by a tenth
//! (`layer_opacity` of the room file).

use super::*;
use crate::data::{Backdrop, BackgroundLayerRef, ProfileLayer};
use bevy::image::ImageLoaderSettings;
use ui::{ACCENT, EditorButton, MUTED, SMALL_PX, TEXT, button, card, font, heading};

const THUMB_PX: Vec2 = Vec2::new(56.0, 31.5);

/// The card; `built_for` is the room and backgrounds its rows were laid out
/// for, so they are laid out again when either changes.
#[derive(Component, Default)]
pub(super) struct LayersCard {
    built_for: Option<(u16, Vec<Backdrop>)>,
}

/// The tick in a layer's box, shown while the room draws the layer.
#[derive(Component)]
pub(super) struct LayerTick(BackgroundLayerRef);

/// A layer's thumbnail, faded while the room hides the layer.
#[derive(Component)]
pub(super) struct LayerThumb(BackgroundLayerRef);

/// How opaque the room draws a layer, written between its `-` and `+`.
#[derive(Component)]
pub(super) struct LayerOpacity(BackgroundLayerRef);

pub(super) fn spawn_layers_card(commands: &mut Commands, left: Entity) {
    commands.spawn((card(10.0), LayersCard::default(), ChildOf(left)));
}

pub(super) fn update_layers_card(
    mut commands: Commands,
    editor: Res<RoomEditor>,
    data: Res<GameData>,
    asset_server: Res<AssetServer>,
    mut cards: Query<(Entity, &mut LayersCard, &mut Node)>,
    mut ticks: Query<(&LayerTick, &mut Visibility)>,
    mut thumbs: Query<(&LayerThumb, &mut ImageNode)>,
) {
    let Some(room) = data.rooms.get(&editor.room_id) else {
        return;
    };
    for (entity, mut card, mut node) in &mut cards {
        let display = if editor.selected.is_empty() {
            Display::Flex
        } else {
            Display::None
        };
        if node.display != display {
            node.display = display;
        }
        let wanted = (editor.room_id, room.background_profiles.clone());
        if card.built_for.as_ref() == Some(&wanted) {
            continue;
        }
        card.built_for = Some(wanted);
        commands.entity(entity).despawn_children();
        commands.entity(entity).with_children(|card| {
            card.spawn(Node {
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                ..default()
            })
            .with_children(|header| {
                header.spawn(heading("Background layers"));
                button(header, "Show all", "", EditorAction::ShowAllLayers);
            });
            for backdrop in &room.background_profiles {
                match backdrop {
                    Backdrop::Profile(name) => {
                        let Some(profile) = data.backgrounds.profiles.get(name) else {
                            continue;
                        };
                        card.spawn((
                            Text::new(name.as_str()),
                            font(SMALL_PX + 1.0),
                            TextColor(TEXT),
                            Node {
                                margin: UiRect::top(px(4)),
                                ..default()
                            },
                        ));
                        let filtering = profile.filtering;
                        let sky = (ProfileLayer::Sky, profile.backdrop.as_str(), None);
                        let layers = profile.layers.iter().enumerate().map(|(index, layer)| {
                            (
                                ProfileLayer::Scenery(index),
                                layer.asset.as_str(),
                                Some(layer.parallax),
                            )
                        });
                        for (layer, asset, parallax) in [sky].into_iter().chain(layers) {
                            let image = asset_server
                                .load_builder()
                                .with_settings(move |settings: &mut ImageLoaderSettings| {
                                    settings.sampler = filtering.into();
                                })
                                .load(asset.to_string());
                            let reference = BackgroundLayerRef {
                                profile: name.clone(),
                                layer,
                            };
                            let (title, detail) = match (layer, parallax) {
                                (ProfileLayer::Scenery(index), Some(parallax)) => (
                                    format!("Layer {}", index + 1),
                                    format!("parallax {parallax}"),
                                ),
                                _ => (String::from("Sky"), String::from("behind everything")),
                            };
                            spawn_layer_row(card, reference, image, title, detail);
                        }
                    }
                    Backdrop::Color(color) => {
                        card.spawn(Node {
                            column_gap: px(8),
                            align_items: AlignItems::Center,
                            margin: UiRect::top(px(4)),
                            ..default()
                        })
                        .with_children(|line| {
                            line.spawn((
                                Node {
                                    width: px(THUMB_PX.x),
                                    height: px(THUMB_PX.y),
                                    border_radius: BorderRadius::all(px(3)),
                                    ..default()
                                },
                                BackgroundColor(Color::from(*color)),
                            ));
                            line.spawn((
                                Text::new(format!("Plain sky {color}")),
                                font(SMALL_PX + 1.0),
                                TextColor(TEXT),
                            ));
                        });
                    }
                }
            }
        });
    }
    for (LayerTick(layer), mut visibility) in &mut ticks {
        visibility.set_if_neq(if room.shows_layer(&layer.profile, layer.layer) {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });
    }
    for (LayerThumb(layer), mut image) in &mut thumbs {
        let alpha = if room.shows_layer(&layer.profile, layer.layer) {
            room.layer_opacity(&layer.profile, layer.layer).max(0.15)
        } else {
            0.15
        };
        if image.color.alpha() != alpha {
            image.color = Color::WHITE.with_alpha(alpha);
        }
    }
}

/// Writes how opaque the room draws each layer between its `-` and `+`.
pub(super) fn show_layer_opacity(
    editor: Res<RoomEditor>,
    data: Res<GameData>,
    mut opacities: Query<(&LayerOpacity, &mut Text)>,
) {
    let Some(room) = data.rooms.get(&editor.room_id) else {
        return;
    };
    for (LayerOpacity(layer), mut text) in &mut opacities {
        let percent = format!(
            "{:.0}%",
            room.layer_opacity(&layer.profile, layer.layer) * 100.0
        );
        if text.0 != percent {
            text.0 = percent;
        }
    }
}

/// One layer: its tick box, its thumbnail, its name and how it scrolls.
fn spawn_layer_row(
    card: &mut ChildSpawnerCommands,
    layer: BackgroundLayerRef,
    image: Handle<Image>,
    title: String,
    detail: String,
) {
    card.spawn((
        Node {
            padding: UiRect::axes(px(6), px(3)),
            border_radius: BorderRadius::all(px(5)),
            column_gap: px(10),
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(Color::NONE),
        bevy::ui::RelativeCursorPosition::default(),
        EditorButton(EditorAction::ToggleLayer(layer.clone())),
    ))
    .with_children(|row| {
        row.spawn((
            Node {
                width: px(16),
                height: px(16),
                border: UiRect::all(px(1)),
                border_radius: BorderRadius::all(px(4)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_shrink: 0.0,
                ..default()
            },
            BorderColor::all(MUTED),
        ))
        .with_children(|tick_box| {
            tick_box.spawn((
                Node {
                    width: px(8),
                    height: px(8),
                    border_radius: BorderRadius::all(px(2)),
                    ..default()
                },
                BackgroundColor(ACCENT),
                LayerTick(layer.clone()),
            ));
        });
        row.spawn((
            Node {
                width: px(THUMB_PX.x),
                height: px(THUMB_PX.y),
                border_radius: BorderRadius::all(px(3)),
                flex_shrink: 0.0,
                ..default()
            },
            BackgroundColor(Color::srgb(0.12, 0.13, 0.17)),
            ImageNode::new(image),
            LayerThumb(layer.clone()),
        ));
        row.spawn(Node {
            flex_direction: FlexDirection::Column,
            ..default()
        })
        .with_children(|words| {
            words.spawn((Text::new(title), font(SMALL_PX + 1.0), TextColor(TEXT)));
            words.spawn((Text::new(detail), font(SMALL_PX), TextColor(MUTED)));
        });
        // Its opacity, faded out or in by a tenth.
        row.spawn(Node {
            margin: UiRect::left(Val::Auto),
            column_gap: px(4),
            align_items: AlignItems::Center,
            ..default()
        })
        .with_children(|fade| {
            button(fade, "-", "", EditorAction::FadeLayer(layer.clone(), -1));
            fade.spawn((
                Text::new(""),
                font(SMALL_PX),
                TextColor(TEXT),
                Node {
                    min_width: px(34),
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                TextLayout::justify(Justify::Center),
                LayerOpacity(layer.clone()),
            ));
            button(fade, "+", "", EditorAction::FadeLayer(layer, 1));
        });
    });
}
