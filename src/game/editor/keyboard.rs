//! Editor shortcuts: typing into text fields, adding, deleting, duplicating,
//! copying, and pasting items, the inspector, undo, switching rooms, saving,
//! playtesting, and leaving.

use super::*;
use crate::data::{
    DecorationConfig, DecorationLayer, EnemySpawnConfig, Fill, PlatformConfig, SpawnPosition,
};
use bevy::input::{
    ButtonState,
    keyboard::{Key, KeyboardInput},
};
use std::fs;

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
                let item = editor.primary();
                if let Some(field) = field {
                    editor.edit(&mut data, |room| field.set_text(room, item, &text));
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

#[derive(SystemParam)]
pub(super) struct EditorKeyResources<'w, 's> {
    keys: Res<'w, ButtonInput<KeyCode>>,
    typed: MessageReader<'w, 's, KeyboardInput>,
    editor: ResMut<'w, RoomEditor>,
    data: ResMut<'w, GameData>,
    session: ResMut<'w, RunSession>,
    start_room: ResMut<'w, StartRoom>,
    next_screen: ResMut<'w, NextState<GameScreen>>,
    camera: Query<'w, 's, (&'static mut Transform, &'static mut Projection), With<Camera2d>>,
}

pub(super) fn handle_editor_keys(resources: EditorKeyResources) {
    let EditorKeyResources {
        keys,
        mut typed,
        mut editor,
        mut data,
        mut session,
        mut start_room,
        mut next_screen,
        mut camera,
    } = resources;
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
    if !pressed(KeyCode::Escape) {
        editor.leave_armed = false;
    }
    let room_ids = data.rooms.keys().copied().collect::<Vec<_>>();
    let field = editor.fields().get(editor.field).copied();
    let cursor_h = editor.cursor_h.unwrap_or_default();

    if command && letter('s') {
        if let Err(problem) = data.validate() {
            editor.status = format!("Not saved: {problem}");
            return;
        }
        let mut written = Vec::new();
        for room_id in editor.unsaved_rooms(&data) {
            let Some(room) = data.rooms.get(&room_id) else {
                continue;
            };
            let path = RoomConfig::file_path(&data.asset_root, room_id);
            if let Err(error) = fs::write(&path, room.to_string()) {
                editor.status = format!("Could not write {}: {error}", path.display());
                return;
            }
            editor.saved.insert(room_id, room.clone());
            written.push(format!("room_{room_id:03}.yaml"));
        }
        if editor.unsaved_hitboxes(&data) {
            let hitboxes = Hitboxes::of(&data);
            if let Err(error) = hitboxes.save(&data.asset_root) {
                editor.status = format!("Hitboxes not saved: {error}");
                return;
            }
            editor.saved_hitboxes = Some(hitboxes);
            written.push(String::from("hitboxes (gameplay.yaml, enemy_types.yaml)"));
        }
        editor.status = if written.is_empty() {
            String::from("Nothing to save.")
        } else {
            format!("Saved {}.", written.join(", "))
        };
    } else if command && letter('r') {
        if let Err(problem) = data.validate() {
            editor.status = format!("Can't playtest: {problem}");
            return;
        }
        *start_room = StartRoom(editor.room_id);
        next_screen.set(screens::reset_run(&mut session, &data, *start_room));
    } else if command && (letter('y') || shift && letter('z')) {
        let Some(step) = editor.redo.pop() else {
            editor.status = String::from("Nothing to redo.");
            return;
        };
        let room_id = match &step {
            Snapshot::Room(room_id, _) => Some(*room_id),
            Snapshot::Hitboxes(_) => None,
        };
        if let Some(current) = step.restore(&mut data) {
            editor.undo.push(current);
        }
        if let Some(room_id) = room_id.filter(|room_id| *room_id != editor.room_id) {
            editor.open_room(room_id);
        }
        editor.view_dirty = true;
    } else if command && letter('z') {
        let Some(step) = editor.undo.pop() else {
            editor.status = String::from("Nothing to undo.");
            return;
        };
        let room_id = match &step {
            Snapshot::Room(room_id, _) => Some(*room_id),
            Snapshot::Hitboxes(_) => None,
        };
        if let Some(current) = step.restore(&mut data) {
            editor.redo.push(current);
        }
        if let Some(room_id) = room_id.filter(|room_id| *room_id != editor.room_id) {
            editor.open_room(room_id);
        }
        editor.view_dirty = true;
    } else if command && letter('d') {
        let Some(copied) = data
            .rooms
            .get(&editor.room_id)
            .and_then(|room| Clipboard::copy(room, &editor.selected))
        else {
            return;
        };
        let mut duplicates = Vec::new();
        // Duplicates land one H to the right of the originals.
        editor.edit(&mut data, |room| {
            duplicates = copied.paste(room, copied.origin + Vec2::X);
        });
        editor.selected = duplicates;
        editor.field = 0;
    } else if command && (letter('c') || letter('x')) {
        let room = data.rooms.get(&editor.room_id);
        let Some(copied) = room.and_then(|room| Clipboard::copy(room, &editor.selected)) else {
            editor.status =
                String::from("Nothing to copy: select blocks, decorations, or enemies.");
            return;
        };
        let count = copied.len();
        editor.clipboard = Some(copied);
        if letter('x') {
            let selection = editor.selected.clone();
            editor.edit(&mut data, |room| EditorItem::remove_all(room, &selection));
            editor.select(None);
            editor.status = format!("Cut {count} items.");
        } else {
            editor.status = format!("Copied {count} items.");
        }
    } else if command && letter('v') {
        let Some(copied) = editor.clipboard.clone() else {
            editor.status = String::from("Nothing to paste.");
            return;
        };
        let at = Vec2::new(snap(cursor_h.x, false), snap(cursor_h.y, false));
        let mut pasted = Vec::new();
        editor.edit(&mut data, |room| pasted = copied.paste(room, at));
        editor.status = format!("Pasted {} items.", pasted.len());
        editor.selected = pasted;
        editor.field = 0;
    } else if command && letter('a') {
        if let Some(room) = data.rooms.get(&editor.room_id) {
            editor.selected = EditorItem::all(room)
                .filter(|item| !matches!(item, EditorItem::PlayerSpawn | EditorItem::Exit))
                .collect();
            editor.field = 0;
        }
    } else if command {
    } else if letter('p') || letter('d') {
        let decoration = letter('d');
        let (width, height) = if decoration {
            (4.0, 3.0)
        } else {
            (3.0, SNAP_H)
        };
        let Some(room) = data.rooms.get(&editor.room_id) else {
            return;
        };
        // The fill picked last, else the one this room already paints this
        // kind of item with, else the catalog's first material.
        let same_kind = if decoration {
            room.decorations.first().map(|decoration| &decoration.fill)
        } else {
            room.platforms.first().map(|block| &block.fill)
        };
        let Some(fill) = editor.fill.as_ref().or(same_kind).cloned().or_else(|| {
            data.surfaces
                .materials
                .keys()
                .next()
                .cloned()
                .map(Fill::Material)
        }) else {
            return;
        };
        let rect_h = RectH {
            x: snap(cursor_h.x - width / 2.0, false),
            y: snap(cursor_h.y - height / 2.0, false),
            width,
            height,
        };
        let mut added = None;
        editor.edit(&mut data, |room| {
            added = Some(if decoration {
                room.decorations.push(DecorationConfig {
                    label: None,
                    fill,
                    layer: DecorationLayer::Background,
                    rect_h,
                });
                EditorItem::Decoration(room.decorations.len() - 1)
            } else {
                room.platforms.push(PlatformConfig {
                    label: None,
                    fill,
                    rect_h,
                });
                EditorItem::Platform(room.platforms.len() - 1)
            });
        });
        editor.select(added);
    } else if letter('e') {
        let type_id = editor
            .enemy_type
            .clone()
            .or_else(|| data.enemies.types.keys().next().cloned());
        let Some(type_id) = type_id else {
            return;
        };
        let mut added = None;
        editor.edit(&mut data, |room| {
            let x_h = snap(cursor_h.x, false);
            let y_h = snap(cursor_h.y, false);
            room.enemy_spawns.push(EnemySpawnConfig {
                label: None,
                type_id,
                position_h: SpawnPosition { x_h, y_h },
                stat_overrides: None,
            });
            added = Some(EditorItem::Enemy(room.enemy_spawns.len() - 1));
        });
        editor.select(added);
    } else if pressed(KeyCode::Delete) || pressed(KeyCode::Backspace) {
        if editor.selected.is_empty() {
            return;
        }
        let selection = editor.selected.clone();
        editor.edit(&mut data, |room| EditorItem::remove_all(room, &selection));
        if selection
            .iter()
            .any(|item| matches!(item, EditorItem::PlayerSpawn | EditorItem::Exit))
        {
            editor.status = String::from("The player spawn and the exit can only be moved.");
        }
        editor.select(None);
    } else if pressed(KeyCode::ArrowUp) || pressed(KeyCode::ArrowDown) {
        let count = editor.fields().len().max(1);
        editor.field = if pressed(KeyCode::ArrowUp) {
            (editor.field + count - 1) % count
        } else {
            (editor.field + 1) % count
        };
    } else if pressed(KeyCode::ArrowLeft) || pressed(KeyCode::ArrowRight) {
        let direction = if pressed(KeyCode::ArrowLeft) {
            -1.0
        } else {
            1.0
        };
        let steps = if shift { direction * 4.0 } else { direction };
        let selection = editor.selected.clone();
        let item = editor.primary();
        match field {
            Some(Field::Room) => {
                let position = room_ids.iter().position(|id| *id == editor.room_id);
                let next = position.and_then(|position| {
                    let next = position as isize + direction as isize;
                    room_ids.get(usize::try_from(next).ok()?).copied()
                });
                if let Some(room_id) = next {
                    editor.open_room(room_id);
                }
            }
            Some(Field::Attack) => {
                let count = editor.selected_enemy_type(&data).map_or(0, |type_id| {
                    let enemy = &data.enemies.types[&type_id];
                    usize::from(enemy.attack.is_some()) + enemy.attacks.len()
                });
                editor.attack = (editor.attack as isize + direction as isize)
                    .rem_euclid(count.max(1) as isize) as usize;
            }
            Some(field) if field.is_hitbox() => {
                let type_id = editor.selected_enemy_type(&data);
                let attack = editor.attack;
                editor.edit_hitboxes(&mut data, |data| {
                    field.adjust_hitbox(data, type_id.as_ref(), attack, steps);
                });
            }
            Some(field) => {
                let snapshot = &*data;
                let mut room = snapshot.rooms.get(&editor.room_id).cloned();
                if let Some(room) = &mut room {
                    field.adjust(room, &selection, snapshot, steps);
                }
                if let Some(room) = room {
                    editor.edit(&mut data, |edited| *edited = room);
                }
                if let Some(EditorItem::Enemy(index)) = item
                    && field == Field::EnemyType
                {
                    editor.enemy_type = data
                        .rooms
                        .get(&editor.room_id)
                        .and_then(|room| room.enemy_spawns.get(index))
                        .map(|spawn| spawn.type_id.clone());
                }
                if field == Field::Fill {
                    editor.fill = data
                        .rooms
                        .get(&editor.room_id)
                        .and_then(|room| item.and_then(|item| item.fill(room)))
                        .cloned();
                }
            }
            None => {}
        }
    } else if pressed(KeyCode::PageUp) || pressed(KeyCode::PageDown) {
        let position = room_ids.iter().position(|id| *id == editor.room_id);
        let next = position.and_then(|position| {
            let next = if pressed(KeyCode::PageUp) {
                position.checked_sub(1)?
            } else {
                position + 1
            };
            room_ids.get(next).copied()
        });
        if let Some(room_id) = next {
            editor.open_room(room_id);
        }
    } else if pressed(KeyCode::Enter) || pressed(KeyCode::NumpadEnter) {
        if let Some(field) = field.filter(|field| field.is_text()) {
            let item = editor.primary();
            editor.text_entry = Some(
                data.rooms
                    .get(&editor.room_id)
                    .and_then(|room| field.text(room, item))
                    .unwrap_or_default(),
            );
        }
    } else if letter('c') {
        if let Some(field) = field {
            let item = editor.primary();
            editor.edit(&mut data, |room| field.clear(room, item));
        }
    } else if pressed(KeyCode::Home) {
        let (Ok((mut transform, mut projection)), Some(room)) =
            (camera.single_mut(), data.rooms.get(&editor.room_id))
        else {
            return;
        };
        let room_size_h = Vec2::new(room.width_h(&data.gameplay), room.size.height_h);
        let view_size_h = Vec2::new(
            data.gameplay.viewport_width_h(),
            data.gameplay.viewport.height_h,
        );
        if let Projection::Orthographic(orthographic) = &mut *projection {
            orthographic.scale = (room_size_h / view_size_h * 1.05)
                .max_element()
                .clamp(MIN_ZOOM, MAX_ZOOM);
        }
        transform.translation.x = room_size_h.x / 2.0 * H_WORLD_UNITS;
        transform.translation.y = room_size_h.y / 2.0 * H_WORLD_UNITS;
    } else if pressed(KeyCode::Escape) {
        let unsaved = !editor.unsaved_rooms(&data).is_empty() || editor.unsaved_hitboxes(&data);
        if !editor.selected.is_empty() {
            editor.select(None);
        } else if !unsaved || editor.leave_armed {
            next_screen.set(GameScreen::MainMenu);
        } else {
            editor.leave_armed = true;
            editor.status = format!(
                "Unsaved edits: {COMMAND_KEY}+S saves them, Esc again leaves (edits stay in memory until the game quits)."
            );
        }
    }
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
