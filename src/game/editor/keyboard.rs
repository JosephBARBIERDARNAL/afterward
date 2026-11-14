//! Editor shortcuts: typing into text fields, and every other key turned
//! into the `EditorAction` a button would send (actions.rs carries them out).

use super::*;
use actions::NewItem;
use bevy::input::{
    ButtonState,
    keyboard::{Key, KeyboardInput},
};

/// Feeds typed characters into the open text field; Enter keeps the text,
/// Escape drops it. Keys handled here are hidden from the shortcuts.
pub(super) fn type_text(
    mut typed: MessageReader<KeyboardInput>,
    mut editor: ResMut<RoomEditor>,
    mut data: ResMut<GameData>,
) {
    editor.keys_consumed = editor.text_entry.is_some();
    for input in typed.read() {
        if input.state != ButtonState::Pressed {
            continue;
        }
        let Some(mut text) = editor.text_entry.take() else {
            continue;
        };
        match input.key_code {
            KeyCode::Enter | KeyCode::NumpadEnter => {
                let field = editor.fields().get(editor.field).copied();
                let selection = editor.selected.clone();
                let snapshot = &*data;
                if let Some(field) = field
                    && let Some(mut room) = snapshot.rooms.get(&editor.room_id).cloned()
                {
                    field.set_text(&mut room, &selection, snapshot, &text);
                    editor.edit(&mut data, |edited| *edited = room);
                    // A typed fill is the one new blocks are painted with.
                    if field == Field::Fill {
                        editor.fill = data
                            .rooms
                            .get(&editor.room_id)
                            .and_then(|room| selection.last()?.fill(room))
                            .cloned();
                    }
                }
                continue;
            }
            KeyCode::Escape => continue,
            KeyCode::Backspace => {
                text.pop();
            }
            _ => {
                if let Some(typed_text) = &input.text {
                    text.extend(
                        typed_text
                            .chars()
                            .filter(|character| !character.is_control()),
                    );
                }
            }
        }
        editor.text_entry = Some(text);
    }
}

/// Turns the shortcuts pressed this frame into editor actions.
pub(super) fn handle_editor_keys(
    keys: Res<ButtonInput<KeyCode>>,
    mut typed: MessageReader<KeyboardInput>,
    mut editor: ResMut<RoomEditor>,
    mut actions: MessageWriter<EditorAction>,
) {
    // Presses come from the key events rather than `ButtonInput`: macOS sends
    // no key-up for a key released while Cmd is held, so a second Cmd+Z would
    // never count as "just pressed". Only the arrows repeat while held.
    let presses = typed
        .read()
        .filter(|input| {
            input.state == ButtonState::Pressed
                && (!input.repeat
                    || matches!(
                        input.key_code,
                        KeyCode::ArrowUp
                            | KeyCode::ArrowDown
                            | KeyCode::ArrowLeft
                            | KeyCode::ArrowRight
                    ))
        })
        .map(ShortcutKey::from)
        .collect::<Vec<_>>();
    if editor.just_opened {
        editor.just_opened = false;
        return;
    }
    if editor.keys_consumed || presses.is_empty() {
        return;
    }
    let command = keys.any_pressed([
        KeyCode::ControlLeft,
        KeyCode::ControlRight,
        KeyCode::SuperLeft,
        KeyCode::SuperRight,
    ]);
    let shift = keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);
    let pressed = |key: KeyCode| presses.iter().any(|press| press.code == key);
    let letter = |character: char| presses.iter().any(|press| press.letter == Some(character));
    let enter = pressed(KeyCode::Enter) || pressed(KeyCode::NumpadEnter);
    let direction = if pressed(KeyCode::ArrowLeft) { -1 } else { 1 };
    if let Some(row) = editor.playtest_row {
        // The settings Cmd+R opened take every key until they are answered.
        let action = if pressed(KeyCode::Escape) {
            EditorAction::PlaytestCancel
        } else if enter || command && letter('r') {
            EditorAction::PlaytestStart
        } else if pressed(KeyCode::ArrowUp) {
            EditorAction::PlaytestMove(-1)
        } else if pressed(KeyCode::ArrowDown) {
            EditorAction::PlaytestMove(1)
        } else if pressed(KeyCode::ArrowLeft) || pressed(KeyCode::ArrowRight) {
            EditorAction::PlaytestChange(row, direction)
        } else {
            return;
        };
        actions.write(action);
        return;
    }
    let action = if command {
        if letter('s') {
            EditorAction::Save
        } else if letter('r') {
            EditorAction::OpenPlaytest
        } else if letter('y') || shift && letter('z') {
            EditorAction::Redo
        } else if letter('z') {
            EditorAction::Undo
        } else if letter('d') {
            EditorAction::Duplicate
        } else if letter('c') {
            EditorAction::Copy
        } else if letter('x') {
            EditorAction::Cut
        } else if letter('v') {
            EditorAction::Paste
        } else if letter('a') {
            EditorAction::SelectAll
        } else {
            return;
        }
    } else if letter('f') {
        EditorAction::ToggleFrame
    } else if letter('v') {
        EditorAction::ToggleSight
    } else if letter('p') {
        EditorAction::Add(NewItem::Platform)
    } else if letter('d') {
        EditorAction::Add(NewItem::Decoration)
    } else if letter('e') {
        EditorAction::Add(NewItem::Enemy)
    } else if letter('g') {
        EditorAction::Add(NewItem::Vent)
    } else if letter('o') {
        EditorAction::Add(NewItem::Prop)
    } else if pressed(KeyCode::Tab) {
        EditorAction::SelectNextOfKind
    } else if pressed(KeyCode::Delete) || pressed(KeyCode::Backspace) {
        EditorAction::Delete
    } else if pressed(KeyCode::ArrowUp) {
        EditorAction::MoveField(-1)
    } else if pressed(KeyCode::ArrowDown) {
        EditorAction::MoveField(1)
    } else if pressed(KeyCode::ArrowLeft) || pressed(KeyCode::ArrowRight) {
        let steps = direction as f32 * if shift { 4.0 } else { 1.0 };
        EditorAction::StepField(steps)
    } else if pressed(KeyCode::PageUp) {
        EditorAction::Room(-1)
    } else if pressed(KeyCode::PageDown) {
        EditorAction::Room(1)
    } else if enter {
        EditorAction::TypeField
    } else if letter('c') {
        EditorAction::ClearField
    } else if pressed(KeyCode::Home) {
        EditorAction::FitRoom
    } else if pressed(KeyCode::Escape) {
        EditorAction::Escape
    } else {
        return;
    };
    actions.write(action);
}

/// A key press as the shortcuts see it. Letter shortcuts match the character
/// the key types on the current layout (Cmd+Z is Z on AZERTY too, where the
/// key sits at QWERTY's W); other keys match by position.
struct ShortcutKey {
    code: KeyCode,
    letter: Option<char>,
}

impl From<&KeyboardInput> for ShortcutKey {
    fn from(input: &KeyboardInput) -> Self {
        let letter = match &input.logical_key {
            Key::Character(text) => text
                .chars()
                .next()
                .map(|character| character.to_ascii_lowercase()),
            _ => None,
        };
        Self {
            code: input.key_code,
            letter,
        }
    }
}
