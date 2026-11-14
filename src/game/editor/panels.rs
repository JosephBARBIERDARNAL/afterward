//! Keeps the panels of ui.rs up to date with the editor: the toolbar's room
//! and save state, the inspector's rows, the status bar, and the help and
//! playtest cards.

use super::*;
use bevy::input::keyboard::Key;
use ui::{ACCENT, GOOD, HelpCard, MUTED, Part, PlaytestCard, TEXT};

/// Every part of the panels, with what `refresh_panels` may change on it.
type Parts<'world, 'state> = Query<
    'world,
    'state,
    (
        &'static Part,
        &'static mut Node,
        Option<&'static mut Text>,
        Option<&'static mut TextColor>,
        Option<&'static mut BackgroundColor>,
    ),
>;

/// What one part shows this frame; `None` leaves that side of it alone.
#[derive(Default)]
struct Look {
    shown: Option<bool>,
    text: Option<String>,
    color: Option<Color>,
}

/// Writes the room, the selection and the save state into every part of the
/// toolbar, the inspector, the status bar and the playtest card.
pub(super) fn refresh_panels(
    editor: Res<RoomEditor>,
    data: Res<GameData>,
    session: Res<RunSession>,
    projection: Query<&Projection, With<Camera2d>>,
    mut parts: Parts,
) {
    let Some(room) = data.rooms.get(&editor.room_id) else {
        return;
    };
    let mut unsaved = editor
        .unsaved_rooms(&data)
        .iter()
        .map(|room_id| format!("{room_id:02}"))
        .collect::<Vec<_>>();
    if editor.unsaved_hitboxes(&data) {
        unsaved.push(String::from("hitboxes"));
    }
    let fields = editor.fields();
    let zoom = projection
        .single()
        .ok()
        .and_then(|projection| match projection {
            Projection::Orthographic(orthographic) => Some(100.0 / orthographic.scale),
            _ => None,
        });
    let look = |part: Part| match part {
        Part::RoomTitle => Look {
            text: Some(format!("Room {:02}  ch. {}", room.room_id, room.chapter_id)),
            ..default()
        },
        Part::SaveBadge if unsaved.is_empty() => Look {
            text: Some(String::from("All saved")),
            color: Some(GOOD),
            ..default()
        },
        Part::SaveBadge => Look {
            text: Some(format!("Unsaved {}", unsaved.join(", "))),
            color: Some(ACCENT),
            ..default()
        },
        Part::Chip => Look {
            color: Some(match editor.selected.as_slice() {
                [] => MUTED,
                [item] => item.outline_color(),
                _ => draw::SELECTED_COLOR,
            }),
            ..default()
        },
        Part::Title => Look {
            text: Some(match editor.selected.as_slice() {
                [] => format!("Room {:02}", room.room_id),
                [item] => {
                    let mut name = item.describe(room);
                    if let Some(first) = name.get_mut(..1) {
                        first.make_ascii_uppercase();
                    }
                    name
                }
                items => format!("{} items selected", items.len()),
            }),
            ..default()
        },
        Part::Problem => Look {
            shown: Some(editor.problem.is_some()),
            text: editor
                .problem
                .as_ref()
                .map(|problem| format!("! {problem}")),
            ..default()
        },
        Part::Hint => Look {
            text: Some(inspector_hint(&editor, &data)),
            ..default()
        },
        Part::Row(index) => Look {
            shown: Some(index < fields.len()),
            ..default()
        },
        Part::RowName(index) => Look {
            text: fields.get(index).map(|field| field.name().to_string()),
            ..default()
        },
        Part::RowValue(index) => match (&editor.text_entry, fields.get(index)) {
            (Some(typed), Some(_)) if index == editor.field => Look {
                text: Some(format!("{typed}_")),
                color: Some(ACCENT),
                ..default()
            },
            (_, Some(field)) => {
                let value = field.value(room, &editor.selected, &data, editor.attack);
                let blank = value.is_empty() || value == "-";
                Look {
                    text: Some(match (blank, field.is_text()) {
                        (true, true) => String::from("click to type"),
                        (true, false) => String::from("-"),
                        (false, _) => value,
                    }),
                    color: Some(if blank { MUTED } else { TEXT }),
                    ..default()
                }
            }
            (_, None) => Look::default(),
        },
        Part::RowStepper(index) => Look {
            shown: Some(fields.get(index).is_some_and(|field| field.steps())),
            ..default()
        },
        Part::RowClear(index) => Look {
            shown: Some(fields.get(index).is_some_and(|field| field.clears())),
            ..default()
        },
        Part::StatusHint => {
            let (text, color) = status_hint(&editor, room);
            Look {
                text: Some(text),
                color: Some(color),
                ..default()
            }
        }
        Part::StatusInfo => {
            let mut info = Vec::new();
            let overlays = [
                (editor.overlays.viewport, "frame"),
                (editor.overlays.sight, "sight"),
            ]
            .into_iter()
            .filter_map(|(on, name)| on.then_some(name))
            .collect::<Vec<_>>();
            if !overlays.is_empty() {
                info.push(format!("Overlays: {}", overlays.join(", ")));
            }
            if let Some(cursor_h) = editor.cursor_h {
                info.push(format!("x {:.2}  y {:.2} H", cursor_h.x, cursor_h.y));
            }
            if let Some(zoom) = zoom {
                info.push(format!("zoom {zoom:.0}%"));
            }
            info.push(format!(
                "room {} x {} H",
                room.size.width_h, room.size.height_h
            ));
            Look {
                text: Some(info.join("     ")),
                ..default()
            }
        }
        Part::PlaytestTitle => Look {
            text: Some(format!("Playtest room {:02}", editor.room_id)),
            ..default()
        },
        Part::PlaytestValue(row) => {
            let on_off = |on: bool| String::from(if on { "on" } else { "off" });
            Look {
                text: Some(match row {
                    0 => on_off(session.playtest.infinite_health),
                    1 => on_off(!session.playtest.no_enemies),
                    _ => match session.playtest.starting_bonuses {
                        0 => String::from("none"),
                        usize::MAX => String::from("all"),
                        count => format!("{count} at random"),
                    },
                }),
                ..default()
            }
        }
    };
    for (part, mut node, text, text_color, background) in &mut parts {
        let Look {
            shown,
            text: words,
            color,
        } = look(*part);
        if let Some(shown) = shown {
            let display = if shown { Display::Flex } else { Display::None };
            if node.display != display {
                node.display = display;
            }
        }
        // Only what changed is written, so the layout is not redone each frame.
        if let (Some(words), Some(mut text)) = (words, text)
            && text.0 != words
        {
            text.0 = words;
        }
        if let Some(color) = color {
            match (text_color, background) {
                (Some(mut text_color), _) => {
                    text_color.set_if_neq(TextColor(color));
                }
                (None, Some(mut background)) => {
                    background.set_if_neq(BackgroundColor(color));
                }
                (None, None) => {}
            }
        }
    }
}

/// The lines under the inspector's rows: how to use the focused row, and
/// what the selection's edits reach.
fn inspector_hint(editor: &RoomEditor, data: &GameData) -> String {
    let field = editor.fields().get(editor.field).copied();
    let mut lines = Vec::new();
    if editor.text_entry.is_some() {
        lines.push(String::from("Typing: Enter keeps it, Esc cancels."));
    } else if field.is_some_and(Field::is_text) {
        lines.push(String::from("Click the value or press Enter to type."));
    }
    if field.is_some_and(Field::is_hitbox) {
        lines.push(match editor.selected_enemy_type(data) {
            Some(type_id) => format!("Hitboxes are shared by every {type_id} (enemy_types.yaml)."),
            None => String::from(
                "The player's drawn size (animations.yaml) and hurtbox (gameplay.yaml).",
            ),
        });
    }
    match editor.selected.as_slice() {
        [] => lines.push(String::from("Click an item in the room to edit it.")),
        [_] => {}
        _ => lines.push(String::from("Fill and X / Y change every selected item.")),
    }
    lines.join("\n")
}

/// The status bar's line: a warning before leaving with unsaved edits, else
/// what is under the cursor, else what can be done from here.
fn status_hint(editor: &RoomEditor, room: &RoomConfig) -> (String, Color) {
    if editor.leave_armed {
        return (
            format!(
                "Unsaved edits: Esc again leaves anyway ({COMMAND_KEY}+S saves first; edits stay in memory until the game quits)."
            ),
            ACCENT,
        );
    }
    if let Some(hovered) = editor
        .hovered
        .filter(|item| !editor.selected.contains(item))
    {
        return (format!("{}: click to select", hovered.describe(room)), TEXT);
    }
    let hint = if editor.selected.is_empty() {
        String::from(
            "Click to select, drag empty space to box-select, P / D / E / G add at the cursor. Hold H for every shortcut.",
        )
    } else {
        format!(
            "Drag to move (Alt: fine grid), drag an edge to resize, Del deletes, {COMMAND_KEY}+D duplicates."
        )
    };
    (hint, MUTED)
}

/// Shows the shortcuts while H is held (by the character it types, so on any
/// layout) unless a row is being typed into, or while the toolbar pins them;
/// and the playtest settings while they are open.
pub(super) fn show_cards(
    keys: Res<ButtonInput<Key>>,
    editor: Res<RoomEditor>,
    mut help: Query<&mut Visibility, (With<HelpCard>, Without<PlaytestCard>)>,
    mut playtest: Query<&mut Visibility, With<PlaytestCard>>,
) {
    let held = keys.any_pressed([Key::Character("h".into()), Key::Character("H".into())])
        && editor.text_entry.is_none();
    let visibility = |shown: bool| {
        if shown {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        }
    };
    let help_shown = (held || editor.help_pinned) && editor.playtest_row.is_none();
    for mut card in &mut help {
        card.set_if_neq(visibility(help_shown));
    }
    for mut card in &mut playtest {
        card.set_if_neq(visibility(editor.playtest_row.is_some()));
    }
}
