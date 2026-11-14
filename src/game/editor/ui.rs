//! The editor's panels and their look: a toolbar along the top, the inspector
//! and the background layers down the left, the palette on the right, a status
//! bar along the bottom, and the help and playtest cards over the room.
//! Every clickable thing is an `EditorButton` carrying the `EditorAction` a
//! shortcut would send.

use super::*;
use actions::{NewItem, PLAYTEST_ROWS};
use bevy::ui::{ComputedNode, RelativeCursorPosition};

pub(super) const PANEL_BG: Color = Color::srgba(0.055, 0.062, 0.09, 0.97);
pub(super) const PANEL_BORDER: Color = Color::srgba(1.0, 1.0, 1.0, 0.09);
pub(super) const TEXT: Color = Color::srgb(0.93, 0.94, 0.97);
pub(super) const MUTED: Color = Color::srgb(0.56, 0.6, 0.68);
pub(super) const ACCENT: Color = Color::srgb(1.0, 0.82, 0.32);
pub(super) const GOOD: Color = Color::srgb(0.45, 0.88, 0.56);
pub(super) const DANGER: Color = Color::srgb(1.0, 0.45, 0.45);
const BUTTON_BG: Color = Color::srgba(1.0, 1.0, 1.0, 0.045);
const BUTTON_HOVER: Color = Color::srgba(1.0, 1.0, 1.0, 0.13);
const BUTTON_LIT: Color = Color::srgba(1.0, 0.82, 0.32, 0.2);
const BUTTON_LIT_HOVER: Color = Color::srgba(1.0, 0.82, 0.32, 0.32);
pub(super) const FONT_PX: f32 = 14.0;
pub(super) const SMALL_PX: f32 = 12.0;
/// How many rows the inspector holds: the most any selection shows.
pub(super) const INSPECTOR_ROWS: usize = 16;
const LEFT_COLUMN_PX: f32 = 400.0;

/// A panel: clicks over it never reach the room.
#[derive(Component)]
pub(super) struct EditorBlocker;

/// Something to click: it sends `action` as the matching shortcut would.
#[derive(Component)]
pub(super) struct EditorButton(pub(super) EditorAction);

/// A button's words, dimmed when the button cannot be pressed.
#[derive(Component)]
pub(super) struct ButtonText;

/// A part of a panel that `panels.rs` keeps up to date.
#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub(super) enum Part {
    /// The toolbar's room name and chapter.
    RoomTitle,
    /// The toolbar's saved / unsaved badge.
    SaveBadge,
    /// The inspector's dot, in the color the selection is outlined in.
    Chip,
    Title,
    Problem,
    Hint,
    /// An inspector row, by its place in the selection's fields; then its
    /// name, its value, its < and > buttons and its clear button.
    Row(usize),
    RowName(usize),
    RowValue(usize),
    RowStepper(usize),
    RowClear(usize),
    StatusHint,
    StatusInfo,
    PlaytestTitle,
    PlaytestValue(usize),
}

/// The shortcuts card, shown while H is held or pinned from the toolbar.
#[derive(Component)]
pub(super) struct HelpCard;

/// The playtest settings, a card over a dimmed room.
#[derive(Component)]
pub(super) struct PlaytestCard;
pub(super) fn font(size: f32) -> TextFont {
    TextFont {
        font_size: bevy::text::FontSize::Px(size),
        ..default()
    }
}

/// A rounded card with the panels' background and border.
pub(super) fn card(padding: f32) -> impl Bundle {
    (
        Node {
            padding: UiRect::all(px(padding)),
            border: UiRect::all(px(1)),
            border_radius: BorderRadius::all(px(8)),
            flex_direction: FlexDirection::Column,
            row_gap: px(6),
            ..default()
        },
        BackgroundColor(PANEL_BG),
        BorderColor::all(PANEL_BORDER),
        RelativeCursorPosition::default(),
        EditorBlocker,
    )
}

/// A small heading over a card's contents.
pub(super) fn heading(text: &str) -> impl Bundle {
    (
        Text::new(text.to_uppercase()),
        font(SMALL_PX),
        TextColor(MUTED),
    )
}

/// A button showing `label`, then its shortcut dimmed when it has one.
pub(super) fn button(
    parent: &mut ChildSpawnerCommands,
    label: &str,
    key: &str,
    action: EditorAction,
) {
    parent
        .spawn((
            Node {
                padding: UiRect::axes(px(7), px(5)),
                border_radius: BorderRadius::all(px(5)),
                column_gap: px(6),
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(BUTTON_BG),
            RelativeCursorPosition::default(),
            EditorButton(action),
        ))
        .with_children(|button| {
            button.spawn((Text::new(label), font(FONT_PX), TextColor(TEXT), ButtonText));
            if !key.is_empty() {
                button.spawn((Text::new(key), font(SMALL_PX), TextColor(MUTED)));
            }
        });
}

fn separator(parent: &mut ChildSpawnerCommands) {
    parent.spawn((
        Node {
            width: px(1),
            height: px(20),
            margin: UiRect::horizontal(px(4)),
            ..default()
        },
        BackgroundColor(PANEL_BORDER),
    ));
}

pub(super) fn spawn_editor_ui(
    commands: &mut Commands,
    data: &GameData,
    asset_server: &AssetServer,
) {
    let root = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: percent(100),
                height: percent(100),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            Pickable::IGNORE,
            EditorUi,
        ))
        .id();
    commands
        .spawn((
            Node {
                width: percent(100),
                padding: UiRect::axes(px(10), px(6)),
                column_gap: px(4),
                row_gap: px(4),
                align_items: AlignItems::Center,
                flex_wrap: FlexWrap::Wrap,
                border: UiRect::bottom(px(1)),
                ..default()
            },
            BackgroundColor(PANEL_BG),
            BorderColor::all(PANEL_BORDER),
            RelativeCursorPosition::default(),
            EditorBlocker,
            ChildOf(root),
        ))
        .with_children(|bar| {
            button(bar, "<", "", EditorAction::Room(-1));
            bar.spawn((
                Text::new(""),
                font(FONT_PX),
                TextColor(TEXT),
                Node {
                    min_width: px(110),
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                TextLayout::justify(Justify::Center),
                Part::RoomTitle,
            ));
            button(bar, ">", "", EditorAction::Room(1));
            separator(bar);
            button(bar, "+ Platform", "P", EditorAction::Add(NewItem::Platform));
            button(
                bar,
                "+ Decoration",
                "D",
                EditorAction::Add(NewItem::Decoration),
            );
            button(bar, "+ Enemy", "E", EditorAction::Add(NewItem::Enemy));
            button(bar, "+ Vent", "G", EditorAction::Add(NewItem::Vent));
            button(bar, "+ Prop", "O", EditorAction::Add(NewItem::Prop));
            separator(bar);
            button(bar, "Undo", "", EditorAction::Undo);
            button(bar, "Redo", "", EditorAction::Redo);
            button(bar, "Delete", "", EditorAction::Delete);
            separator(bar);
            button(bar, "Frame", "F", EditorAction::ToggleFrame);
            button(bar, "Sight", "V", EditorAction::ToggleSight);
            button(bar, "Fit", "Home", EditorAction::FitRoom);
            button(bar, "Help", "H", EditorAction::ToggleHelp);
            bar.spawn(Node {
                flex_grow: 1.0,
                ..default()
            });
            bar.spawn((
                Text::new(""),
                font(SMALL_PX),
                TextColor(GOOD),
                Node {
                    margin: UiRect::right(px(6)),
                    ..default()
                },
                Part::SaveBadge,
            ));
            button(bar, "Save", &format!("{COMMAND_KEY}+S"), EditorAction::Save);
            button(
                bar,
                "Playtest",
                &format!("{COMMAND_KEY}+R"),
                EditorAction::OpenPlaytest,
            );
        });

    // The room shows between the left column and the palette.
    let middle = commands
        .spawn((
            Node {
                flex_grow: 1.0,
                padding: UiRect::all(px(10)),
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::FlexStart,
                overflow: Overflow::clip(),
                ..default()
            },
            Pickable::IGNORE,
            ChildOf(root),
        ))
        .id();
    let left = commands
        .spawn((
            Node {
                width: px(LEFT_COLUMN_PX),
                flex_direction: FlexDirection::Column,
                row_gap: px(10),
                ..default()
            },
            Pickable::IGNORE,
            ChildOf(middle),
        ))
        .id();
    spawn_inspector(commands, left);
    layers::spawn_layers_card(commands, left);
    palette::spawn_palette(commands, data, asset_server, middle);

    commands
        .spawn((
            Node {
                width: percent(100),
                padding: UiRect::axes(px(12), px(5)),
                column_gap: px(16),
                justify_content: JustifyContent::SpaceBetween,
                border: UiRect::top(px(1)),
                ..default()
            },
            BackgroundColor(PANEL_BG),
            BorderColor::all(PANEL_BORDER),
            RelativeCursorPosition::default(),
            EditorBlocker,
            ChildOf(root),
        ))
        .with_children(|bar| {
            bar.spawn((
                Text::new(""),
                font(SMALL_PX),
                TextColor(MUTED),
                Part::StatusHint,
            ));
            bar.spawn((
                Text::new(""),
                font(SMALL_PX),
                TextColor(MUTED),
                Part::StatusInfo,
            ));
        });

    spawn_help(commands);
    spawn_playtest(commands);
}

fn spawn_inspector(commands: &mut Commands, left: Entity) {
    commands
        .spawn((card(10.0), ChildOf(left)))
        .with_children(|card| {
            card.spawn(Node {
                column_gap: px(8),
                align_items: AlignItems::Center,
                ..default()
            })
            .with_children(|header| {
                header.spawn((
                    Node {
                        width: px(10),
                        height: px(10),
                        border_radius: BorderRadius::all(px(3)),
                        ..default()
                    },
                    BackgroundColor(ACCENT),
                    Part::Chip,
                ));
                header.spawn((
                    Text::new(""),
                    font(FONT_PX + 1.0),
                    TextColor(TEXT),
                    Part::Title,
                ));
            });
            card.spawn((
                Text::new(""),
                font(SMALL_PX),
                TextColor(DANGER),
                Node {
                    display: Display::None,
                    ..default()
                },
                Part::Problem,
            ));
            card.spawn(Node {
                flex_direction: FlexDirection::Column,
                row_gap: px(1),
                ..default()
            })
            .with_children(|rows| {
                for index in 0..INSPECTOR_ROWS {
                    spawn_row(rows, index);
                }
            });
            card.spawn((Text::new(""), font(SMALL_PX), TextColor(MUTED), Part::Hint));
        });
}

/// An inspector row: its name, its value (click to focus it, or to type into
/// it), then clear, < and > buttons where they apply.
fn spawn_row(rows: &mut ChildSpawnerCommands, index: usize) {
    let small_button = |label: &'static str, action: EditorAction| {
        (
            Node {
                width: px(22),
                height: px(20),
                border_radius: BorderRadius::all(px(4)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(BUTTON_BG),
            RelativeCursorPosition::default(),
            EditorButton(action),
            children![(Text::new(label), font(FONT_PX), TextColor(TEXT), ButtonText)],
        )
    };
    rows.spawn((
        Node {
            padding: UiRect::new(px(8), px(4), px(3), px(3)),
            border_radius: BorderRadius::all(px(5)),
            column_gap: px(4),
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(BUTTON_BG),
        RelativeCursorPosition::default(),
        EditorButton(EditorAction::EditField(index)),
        Part::Row(index),
    ))
    .with_children(|row| {
        row.spawn((
            Text::new(""),
            font(FONT_PX - 1.0),
            TextColor(MUTED),
            Node {
                width: px(140),
                flex_shrink: 0.0,
                ..default()
            },
            Part::RowName(index),
        ));
        row.spawn((
            Text::new(""),
            font(FONT_PX),
            TextColor(TEXT),
            Node {
                flex_grow: 1.0,
                flex_shrink: 1.0,
                ..default()
            },
            Part::RowValue(index),
        ));
        row.spawn((
            small_button("x", EditorAction::ClearFieldAt(index)),
            Part::RowClear(index),
        ));
        row.spawn((
            small_button("<", EditorAction::StepFieldAt(index, -1.0)),
            Part::RowStepper(index),
        ));
        row.spawn((
            small_button(">", EditorAction::StepFieldAt(index, 1.0)),
            Part::RowStepper(index),
        ));
    });
}

/// The shortcuts, grouped, in a card over the middle of the screen.
fn spawn_help(commands: &mut Commands) {
    let sections: [(&str, Vec<(String, &str)>); 4] = [
        (
            "Select and move",
            vec![
                (String::from("Click"), "select (Shift: add or remove)"),
                (String::from("Drag empty space"), "box-select"),
                (String::from("Drag selection"), "move it (Alt: fine grid)"),
                (String::from("Drag an edge"), "resize the selected block"),
                (String::from("Tab"), "next item of the same kind"),
                (format!("{COMMAND_KEY}+A"), "select all"),
                (String::from("Esc"), "deselect, then leave"),
            ],
        ),
        (
            "Edit",
            vec![
                (
                    String::from("P / D / E / G / O"),
                    "add platform / decoration / enemy / vent / prop",
                ),
                (String::from("Del"), "delete the selection"),
                (format!("{COMMAND_KEY}+D"), "duplicate"),
                (
                    format!("{COMMAND_KEY}+C / X / V"),
                    "copy / cut / paste at cursor",
                ),
                (format!("{COMMAND_KEY}+Z"), "undo"),
                (format!("{COMMAND_KEY}+Shift+Z, {COMMAND_KEY}+Y"), "redo"),
                (String::from("Material, color"), "paint the selection"),
            ],
        ),
        (
            "Inspector",
            vec![
                (String::from("Up / Down"), "pick a row (or click it)"),
                (String::from("Left / Right"), "change it (Shift: x4)"),
                (String::from("Enter"), "type into a text row"),
                (String::from("C"), "clear the row"),
            ],
        ),
        (
            "View and room",
            vec![
                (String::from("Scroll, right drag"), "pan"),
                (String::from("Pinch, wheel"), "zoom"),
                (String::from("Home"), "fit the room"),
                (String::from("F"), "viewport frame"),
                (String::from("V"), "what idle enemies see"),
                (String::from("PgUp / PgDn"), "previous / next room"),
                (format!("{COMMAND_KEY}+S"), "save"),
                (format!("{COMMAND_KEY}+R"), "playtest"),
                (String::from("B (in play)"), "hitbox overlay"),
            ],
        ),
    ];
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: percent(100),
                height: percent(100),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.45)),
            GlobalZIndex(10),
            Visibility::Hidden,
            Pickable::IGNORE,
            HelpCard,
            EditorUi,
        ))
        .with_children(|screen| {
            screen
                .spawn(card(18.0))
                .insert(Node {
                    padding: UiRect::all(px(18)),
                    border: UiRect::all(px(1)),
                    border_radius: BorderRadius::all(px(10)),
                    flex_direction: FlexDirection::Row,
                    flex_wrap: FlexWrap::Wrap,
                    max_width: px(1040),
                    column_gap: px(28),
                    row_gap: px(16),
                    ..default()
                })
                .with_children(|card| {
                    for (title, entries) in sections {
                        card.spawn(Node {
                            width: px(480),
                            flex_direction: FlexDirection::Column,
                            row_gap: px(3),
                            ..default()
                        })
                        .with_children(|section| {
                            section.spawn(heading(title));
                            for (keys, what) in entries {
                                section
                                    .spawn(Node {
                                        column_gap: px(10),
                                        ..default()
                                    })
                                    .with_children(|line| {
                                        line.spawn((
                                            Text::new(keys),
                                            font(SMALL_PX + 1.0),
                                            TextColor(ACCENT),
                                            Node {
                                                width: px(210),
                                                flex_shrink: 0.0,
                                                ..default()
                                            },
                                        ));
                                        line.spawn((
                                            Text::new(what),
                                            font(SMALL_PX + 1.0),
                                            TextColor(TEXT),
                                        ));
                                    });
                            }
                        });
                    }
                });
        });
}

/// The playtest settings: a row per setting (click to change it) and the
/// buttons that play the room or go back to editing it.
fn spawn_playtest(commands: &mut Commands) {
    let names = ["Infinite health", "Enemies", "Starting bonuses"];
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: percent(100),
                height: percent(100),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.55)),
            GlobalZIndex(10),
            Visibility::Hidden,
            RelativeCursorPosition::default(),
            EditorBlocker,
            PlaytestCard,
            EditorUi,
        ))
        .with_children(|screen| {
            screen
                .spawn(card(18.0))
                .insert(Node {
                    padding: UiRect::all(px(18)),
                    border: UiRect::all(px(1)),
                    border_radius: BorderRadius::all(px(10)),
                    flex_direction: FlexDirection::Column,
                    row_gap: px(6),
                    width: px(380),
                    ..default()
                })
                .with_children(|card| {
                    card.spawn((
                        Text::new(""),
                        font(FONT_PX + 3.0),
                        TextColor(TEXT),
                        Part::PlaytestTitle,
                    ));
                    card.spawn(heading("How should it play?"));
                    for (row, name) in names.into_iter().enumerate().take(PLAYTEST_ROWS) {
                        card.spawn((
                            Node {
                                padding: UiRect::axes(px(10), px(6)),
                                border_radius: BorderRadius::all(px(5)),
                                justify_content: JustifyContent::SpaceBetween,
                                ..default()
                            },
                            BackgroundColor(BUTTON_BG),
                            RelativeCursorPosition::default(),
                            EditorButton(EditorAction::PlaytestChange(row, 1)),
                        ))
                        .with_children(|line| {
                            line.spawn((Text::new(name), font(FONT_PX), TextColor(MUTED)));
                            line.spawn((
                                Text::new(""),
                                font(FONT_PX),
                                TextColor(TEXT),
                                Part::PlaytestValue(row),
                            ));
                        });
                    }
                    card.spawn((
                        Text::new("Click a setting or use the arrows to change it."),
                        font(SMALL_PX),
                        TextColor(MUTED),
                    ));
                    card.spawn(Node {
                        margin: UiRect::top(px(8)),
                        column_gap: px(8),
                        justify_content: JustifyContent::FlexEnd,
                        ..default()
                    })
                    .with_children(|buttons| {
                        button(buttons, "Back", "Esc", EditorAction::PlaytestCancel);
                        button(buttons, "Play", "Enter", EditorAction::PlaytestStart);
                    });
                });
        });
}

/// Notes whether the cursor is over a panel, so the room ignores it, and
/// sends the action of the button clicked. Where buttons nest (a row's < in
/// the row), the smallest one under the cursor is the one clicked.
pub(super) fn click_buttons(
    mouse: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut editor: ResMut<RoomEditor>,
    data: Res<GameData>,
    blockers: Query<(&RelativeCursorPosition, &InheritedVisibility), With<EditorBlocker>>,
    buttons: Query<(&RelativeCursorPosition, &ComputedNode, &EditorButton)>,
    mut actions: MessageWriter<EditorAction>,
) {
    // A hidden panel keeps the cursor it had when it was hidden (Bevy stops
    // updating it), so only the panels on show count.
    editor.pointer_over_panel = blockers
        .iter()
        .any(|(cursor, visibility)| visibility.get() && cursor.cursor_over());
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }
    let clicked = buttons
        .iter()
        .filter(|(cursor, _, _)| cursor.cursor_over())
        .min_by(|(_, a, _), (_, b, _)| {
            let area = |node: &ComputedNode| node.size().x * node.size().y;
            area(a).total_cmp(&area(b))
        });
    let Some((_, _, EditorButton(action))) = clicked else {
        return;
    };
    if !action.looks(&editor, &data).1 {
        return;
    }
    // Shift steps four at a time, as it does on the arrows.
    let action = match action {
        EditorAction::StepFieldAt(row, steps)
            if keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]) =>
        {
            EditorAction::StepFieldAt(*row, steps * 4.0)
        }
        action => action.clone(),
    };
    // A click ends any typing, as Escape would, unless it opens a row for it.
    if !matches!(action, EditorAction::EditField(_)) {
        editor.text_entry = None;
    }
    actions.write(action);
}

/// Lights each button by hover, by whether it is on, and dims the ones that
/// cannot be pressed now.
pub(super) fn style_buttons(
    editor: Res<RoomEditor>,
    data: Res<GameData>,
    mut buttons: Query<(
        &RelativeCursorPosition,
        &EditorButton,
        &mut BackgroundColor,
        Option<&Children>,
    )>,
    mut texts: Query<&mut TextColor, With<ButtonText>>,
) {
    for (cursor, EditorButton(action), mut background, children) in &mut buttons {
        let (lit, enabled) = action.looks(&editor, &data);
        let hovered = cursor.cursor_over() && enabled;
        let color = match (lit, hovered) {
            (true, true) => BUTTON_LIT_HOVER,
            (true, false) => BUTTON_LIT,
            (false, true) => BUTTON_HOVER,
            (false, false) => BUTTON_BG,
        };
        background.set_if_neq(BackgroundColor(color));
        for child in children.into_iter().flatten() {
            if let Ok(mut text) = texts.get_mut(*child) {
                text.set_if_neq(TextColor(if enabled {
                    TEXT
                } else {
                    MUTED.with_alpha(0.5)
                }));
            }
        }
    }
}
