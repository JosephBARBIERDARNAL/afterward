//! The material palette: a swatch for every terrain material. Clicking one
//! paints the selected blocks and decorations with it and makes it the
//! material new ones use.

use super::*;
use bevy::ui::RelativeCursorPosition;

const SWATCH_PX: f32 = 26.0;
const SWATCH_GAP_PX: f32 = 4.0;
const COLUMNS: usize = 9;
const SWATCH_BORDER: Color = Color::srgba(1.0, 1.0, 1.0, 0.12);
const CURRENT_BORDER: Color = Color::srgb(1.0, 0.9, 0.2);
const HOVER_BORDER: Color = Color::WHITE;

/// The palette panel; clicks over it never reach the room.
#[derive(Component)]
pub(super) struct PalettePanel;

#[derive(Component)]
pub(super) struct PaletteSwatch(MaterialId);

/// The line naming the hovered (or current) material.
#[derive(Component)]
pub(super) struct PaletteCaption;

pub(super) fn spawn_palette(commands: &mut Commands, data: &GameData, asset_server: &AssetServer) {
    let width = COLUMNS as f32 * (SWATCH_PX + SWATCH_GAP_PX);
    let texture_px = data.terrain.texture_px as f32;
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                right: px(12),
                top: px(12),
                padding: UiRect::all(px(10)),
                flex_direction: FlexDirection::Column,
                row_gap: px(8),
                ..default()
            },
            BackgroundColor(Color::srgba(0.02, 0.025, 0.05, 0.82)),
            RelativeCursorPosition::default(),
            PalettePanel,
            EditorUi,
        ))
        .with_children(|panel| {
            panel.spawn((
                Text::new(""),
                TextFont {
                    font_size: bevy::text::FontSize::Px(15.0),
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    width: px(width),
                    ..default()
                },
                PaletteCaption,
            ));
            panel
                .spawn(Node {
                    width: px(width),
                    flex_direction: FlexDirection::Row,
                    flex_wrap: FlexWrap::Wrap,
                    column_gap: px(SWATCH_GAP_PX),
                    row_gap: px(SWATCH_GAP_PX),
                    ..default()
                })
                .with_children(|grid| {
                    for (id, material) in &data.terrain.materials {
                        let mut swatch = grid.spawn((
                            Node {
                                width: px(SWATCH_PX),
                                height: px(SWATCH_PX),
                                border: UiRect::all(px(2)),
                                ..default()
                            },
                            BorderColor::all(SWATCH_BORDER),
                            ImageNode::new(terrain_art::load_texture(
                                asset_server,
                                &material.texture,
                            )),
                            RelativeCursorPosition::default(),
                            PaletteSwatch(id.clone()),
                        ));
                        if let Some(cap) = &material.top {
                            swatch.with_child((
                                Node {
                                    position_type: PositionType::Absolute,
                                    left: px(0),
                                    top: px(0),
                                    width: percent(100),
                                    height: percent(cap.depth_px / texture_px * 100.0),
                                    ..default()
                                },
                                ImageNode::new(terrain_art::load_texture(
                                    asset_server,
                                    &cap.texture,
                                )),
                            ));
                        }
                    }
                });
        });
}

/// Paints the selection with a clicked swatch's material and remembers it
/// for new blocks; notes whether the cursor is over the palette so the room
/// ignores the click.
pub(super) fn handle_palette(
    mouse: Res<ButtonInput<MouseButton>>,
    mut editor: ResMut<RoomEditor>,
    mut data: ResMut<GameData>,
    panels: Query<&RelativeCursorPosition, With<PalettePanel>>,
    swatches: Query<(&RelativeCursorPosition, &PaletteSwatch)>,
) {
    editor.pointer_over_panel = panels.iter().any(RelativeCursorPosition::cursor_over);
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }
    let Some((_, PaletteSwatch(material))) =
        swatches.iter().find(|(cursor, _)| cursor.cursor_over())
    else {
        return;
    };
    let material = material.clone();
    let selection = editor.selected.clone();
    editor.edit(&mut data, |room| {
        for item in &selection {
            item.set_material(room, &material);
        }
    });
    editor.status = format!("Material: {material}");
    editor.material = Some(material);
}

/// Outlines the current material's swatch and the hovered one, and names
/// the hovered material (or the current one) above the grid.
pub(super) fn highlight_swatches(
    editor: Res<RoomEditor>,
    data: Res<GameData>,
    mut swatches: Query<(&RelativeCursorPosition, &PaletteSwatch, &mut BorderColor)>,
    mut caption: Query<&mut Text, With<PaletteCaption>>,
) {
    let current = data
        .rooms
        .get(&editor.room_id)
        .and_then(|room| editor.primary().and_then(|item| item.material(room)))
        .or(editor.material.as_ref());
    let mut hovered = None;
    for (cursor, PaletteSwatch(material), mut border) in &mut swatches {
        let color = if cursor.cursor_over() {
            hovered = Some(material);
            HOVER_BORDER
        } else if Some(material) == current {
            CURRENT_BORDER
        } else {
            SWATCH_BORDER
        };
        *border = BorderColor::all(color);
    }
    let name = hovered.or(current).map_or("-", |material| material);
    let line = format!("MATERIALS  {name}");
    for mut text in &mut caption {
        if text.0 != line {
            text.0.clone_from(&line);
        }
    }
}
