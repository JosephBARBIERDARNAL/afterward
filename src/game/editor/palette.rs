//! The fill palette: a swatch for every surface material, then a color
//! picker (a field to type any hex code into, and a hue strip over a grid of
//! shades mixed from it). Clicking any of them paints the selected platforms
//! and decorations and becomes the fill new ones use. With nothing selected,
//! a color paints the room's background instead: the one the inspector
//! stands on, else Background 1.

use super::*;
use crate::data::{Backdrop, HexColor, MaterialId};
use bevy::ui::RelativeCursorPosition;

const SWATCH_PX: f32 = 26.0;
const SWATCH_GAP_PX: f32 = 4.0;
const COLUMNS: usize = 9;
/// The color picker's swatches, in a denser grid than the materials.
const COLOR_SWATCH_PX: f32 = 18.0;
const HUES: usize = 12;
/// Saturation steps across the shade grid, left (grey) to right (pure).
const SATURATIONS: usize = 12;
/// Brightness of each row of shades, dark to light.
const LIGHTNESSES: [f32; 4] = [0.16, 0.34, 0.54, 0.78];
const SWATCH_BORDER: Color = Color::srgba(1.0, 1.0, 1.0, 0.12);
const CURRENT_BORDER: Color = Color::srgb(1.0, 0.9, 0.2);
const HOVER_BORDER: Color = Color::WHITE;

/// The palette panel; clicks over it never reach the room.
#[derive(Component)]
pub(super) struct PalettePanel;

#[derive(Component)]
pub(super) struct PaletteSwatch(MaterialId);

/// A hue of the color picker, in degrees: clicking it mixes the shade grid
/// from that hue without painting anything.
#[derive(Component)]
pub(super) struct HueSwatch(f32);

/// Every shade swatch, with what the picked hue repaints each frame.
type Shades<'world, 'state> = Query<
    'world,
    'state,
    (
        &'static RelativeCursorPosition,
        &'static ShadeSwatch,
        &'static mut BorderColor,
        &'static mut BackgroundColor,
    ),
    (Without<PaletteSwatch>, Without<HueSwatch>),
>;

/// One shade of the picked hue.
#[derive(Component)]
pub(super) struct ShadeSwatch {
    saturation: f32,
    lightness: f32,
}

impl ShadeSwatch {
    fn color(&self, hue: f32) -> Color {
        Color::hsl(hue, self.saturation, self.lightness)
    }
}

/// The line naming the hovered (or current) fill.
#[derive(Component)]
pub(super) struct PaletteCaption;

/// The field a hex code is typed into: clicking it opens the inspector's
/// Fill row (or background row) for typing, which Enter keeps.
#[derive(Component)]
pub(super) struct HexField;

/// The palette panel and its hex field, told apart by `Has<HexField>`.
type PanelParts<'world, 'state> = Query<
    'world,
    'state,
    (&'static RelativeCursorPosition, Has<HexField>),
    Or<(With<PalettePanel>, With<HexField>)>,
>;

/// The text of the hex field: the current color, or what is being typed.
#[derive(Component)]
pub(super) struct HexFieldText;

impl RoomEditor {
    /// The background the palette paints while nothing is selected: the one
    /// the inspector stands on, else Background 1. `None` while something is
    /// selected, which the palette paints instead.
    fn painted_background(&self) -> Option<Field> {
        self.selected
            .is_empty()
            .then(|| match self.fields().get(self.field) {
                Some(Field::BackgroundOver) => Field::BackgroundOver,
                _ => Field::Background,
            })
    }
}

pub(super) fn spawn_palette(commands: &mut Commands, data: &GameData, asset_server: &AssetServer) {
    let width = COLUMNS as f32 * (SWATCH_PX + SWATCH_GAP_PX);
    let grid = |columns: usize, swatch_px: f32| Node {
        width: px(width),
        max_width: px(columns as f32 * (swatch_px + SWATCH_GAP_PX)),
        flex_direction: FlexDirection::Row,
        flex_wrap: FlexWrap::Wrap,
        column_gap: px(SWATCH_GAP_PX),
        row_gap: px(SWATCH_GAP_PX),
        ..default()
    };
    let cell = |swatch_px: f32| Node {
        width: px(swatch_px),
        height: px(swatch_px),
        border: UiRect::all(px(2)),
        ..default()
    };
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
            panel.spawn(grid(COLUMNS, SWATCH_PX)).with_children(|grid| {
                for (id, material) in &data.surfaces.materials {
                    grid.spawn((
                        cell(SWATCH_PX),
                        BorderColor::all(SWATCH_BORDER),
                        ImageNode::new(room_art::load_texture(asset_server, &material.texture)),
                        RelativeCursorPosition::default(),
                        PaletteSwatch(id.clone()),
                    ));
                }
            });
            panel
                .spawn((
                    Node {
                        width: px(width),
                        padding: UiRect::axes(px(6), px(4)),
                        border: UiRect::all(px(1)),
                        ..default()
                    },
                    BorderColor::all(HOVER_BORDER.with_alpha(0.4)),
                    BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.5)),
                    RelativeCursorPosition::default(),
                    HexField,
                ))
                .with_children(|field| {
                    field.spawn((
                        Text::new(""),
                        TextFont {
                            font_size: bevy::text::FontSize::Px(15.0),
                            ..default()
                        },
                        TextColor(Color::WHITE),
                        HexFieldText,
                    ));
                });
            panel
                .spawn(grid(HUES, COLOR_SWATCH_PX))
                .with_children(|strip| {
                    for index in 0..HUES {
                        let hue = index as f32 / HUES as f32 * 360.0;
                        strip.spawn((
                            cell(COLOR_SWATCH_PX),
                            BorderColor::all(SWATCH_BORDER),
                            BackgroundColor(Color::hsl(hue, 0.75, 0.5)),
                            RelativeCursorPosition::default(),
                            HueSwatch(hue),
                        ));
                    }
                });
            panel
                .spawn(grid(SATURATIONS, COLOR_SWATCH_PX))
                .with_children(|shades| {
                    for lightness in LIGHTNESSES {
                        for column in 0..SATURATIONS {
                            let shade = ShadeSwatch {
                                saturation: column as f32 / (SATURATIONS - 1) as f32,
                                lightness,
                            };
                            shades.spawn((
                                cell(COLOR_SWATCH_PX),
                                BorderColor::all(SWATCH_BORDER),
                                BackgroundColor(shade.color(0.0)),
                                RelativeCursorPosition::default(),
                                shade,
                            ));
                        }
                    }
                });
        });
}

/// Paints the selection with a clicked material or shade and remembers it for
/// new blocks; a clicked hue only remixes the shades. Notes whether the cursor
/// is over the palette so the room ignores the click.
pub(super) fn handle_palette(
    mouse: Res<ButtonInput<MouseButton>>,
    mut editor: ResMut<RoomEditor>,
    mut data: ResMut<GameData>,
    panels: PanelParts,
    materials: Query<(&RelativeCursorPosition, &PaletteSwatch)>,
    hues: Query<(&RelativeCursorPosition, &HueSwatch)>,
    shades: Query<(&RelativeCursorPosition, &ShadeSwatch)>,
) {
    editor.pointer_over_panel = panels
        .iter()
        .any(|(cursor, is_field)| !is_field && cursor.cursor_over());
    if !mouse.just_pressed(MouseButton::Left) || editor.playtest_row.is_some() {
        return;
    }
    if panels
        .iter()
        .any(|(cursor, is_field)| is_field && cursor.cursor_over())
    {
        // Typing goes through the inspector row it changes, so Enter keeps
        // it and Escape drops it there as anywhere.
        let target = editor.painted_background().unwrap_or(Field::Fill);
        if let Some(row) = editor.fields().iter().position(|field| *field == target) {
            editor.field = row;
            editor.text_entry = Some(String::from("#"));
        }
        return;
    }
    if !editor.pointer_over_panel {
        return;
    }
    editor.text_entry = None;
    if let Some((_, HueSwatch(hue))) = hues.iter().find(|(cursor, _)| cursor.cursor_over()) {
        editor.hue = *hue;
        return;
    }
    let picked = materials
        .iter()
        .find(|(cursor, _)| cursor.cursor_over())
        .map(|(_, PaletteSwatch(material))| Fill::Material(material.clone()))
        .or_else(|| {
            let hue = editor.hue;
            shades
                .iter()
                .find(|(cursor, _)| cursor.cursor_over())
                .map(|(_, shade)| Fill::Color(HexColor::from(shade.color(hue))))
        });
    let Some(fill) = picked else {
        return;
    };
    if let Some(field) = editor.painted_background() {
        if let Fill::Color(color) = fill {
            editor.edit(&mut data, |room| {
                field.set_backdrop(room, Some(Backdrop::Color(color)));
            });
        }
        return;
    }
    let selection = editor.selected.clone();
    editor.edit(&mut data, |room| {
        for item in &selection {
            item.set_fill(room, &fill);
        }
    });
    editor.fill = Some(fill);
}

/// Mixes the shade grid from the picked hue, outlines the current fill's
/// swatch and the hovered one, and names the hovered fill (or the current
/// one) above the grids.
pub(super) fn refresh_palette(
    editor: Res<RoomEditor>,
    data: Res<GameData>,
    mut materials: Query<(&RelativeCursorPosition, &PaletteSwatch, &mut BorderColor)>,
    mut hues: Query<
        (&RelativeCursorPosition, &HueSwatch, &mut BorderColor),
        Without<PaletteSwatch>,
    >,
    mut shades: Shades,
    mut caption: Query<&mut Text, With<PaletteCaption>>,
    mut hex_text: Query<&mut Text, (With<HexFieldText>, Without<PaletteCaption>)>,
) {
    let room = data.rooms.get(&editor.room_id);
    let background = editor.painted_background();
    let current = match background {
        Some(field) => room
            .and_then(|room| {
                room.background_profiles
                    .get(usize::from(field == Field::BackgroundOver))
            })
            .and_then(|backdrop| match backdrop {
                Backdrop::Color(color) => Some(Fill::Color(*color)),
                Backdrop::Profile(_) => None,
            }),
        None => room
            .and_then(|room| editor.primary().and_then(|item| item.fill(room)))
            .or(editor.fill.as_ref())
            .cloned(),
    };
    let current = current.as_ref();
    let mut hovered = None;
    let border = |cursor: &RelativeCursorPosition, is_current: bool| {
        if cursor.cursor_over() {
            HOVER_BORDER
        } else if is_current {
            CURRENT_BORDER
        } else {
            SWATCH_BORDER
        }
    };
    for (cursor, PaletteSwatch(material), mut outline) in &mut materials {
        if cursor.cursor_over() {
            hovered = Some(material.to_string());
        }
        let is_current = matches!(current, Some(Fill::Material(picked)) if picked == material);
        *outline = BorderColor::all(border(cursor, is_current));
    }
    for (cursor, HueSwatch(hue), mut outline) in &mut hues {
        let is_current = (*hue - editor.hue).abs() < f32::EPSILON;
        *outline = BorderColor::all(border(cursor, is_current));
    }
    for (cursor, shade, mut outline, mut background) in &mut shades {
        let color = HexColor::from(shade.color(editor.hue));
        if cursor.cursor_over() {
            hovered = Some(color.to_string());
        }
        let is_current = matches!(current, Some(Fill::Color(picked)) if *picked == color);
        *outline = BorderColor::all(border(cursor, is_current));
        background.0 = Color::from(color);
    }
    let name = hovered
        .or_else(|| current.map(ToString::to_string))
        .unwrap_or_else(|| String::from("-"));
    let target = background.map_or("Fill", Field::name).to_uppercase();
    let line = format!("{target}  {name}");
    for mut text in &mut caption {
        if text.0 != line {
            text.0.clone_from(&line);
        }
    }
    let typing = editor
        .text_entry
        .as_ref()
        .filter(|_| editor.fields().get(editor.field) == Some(&background.unwrap_or(Field::Fill)));
    let hex = match (typing, current) {
        (Some(text), _) => format!("HEX  {text}_   Enter keeps, Esc cancels"),
        (None, Some(Fill::Color(color))) => format!("HEX  {color}   click to type"),
        (None, _) => String::from("HEX  click to type a code"),
    };
    for mut text in &mut hex_text {
        if text.0 != hex {
            text.0.clone_from(&hex);
        }
    }
}
