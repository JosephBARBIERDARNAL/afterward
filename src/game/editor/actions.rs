//! Everything the editor does on request, as one message: a shortcut and a
//! click on a button write the same `EditorAction`, and `run_actions` carries
//! it out, so the two can never disagree.

use super::*;
use crate::data::{
    BackgroundLayerRef, DecorationConfig, DecorationLayer, EnemySpawnConfig, Facing, PropConfig,
    RoomHazardConfig, SpawnPosition,
};
use std::fs;

/// What a new item added with P, D, E, G or O (or the toolbar) is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum NewItem {
    Platform,
    Decoration,
    Enemy,
    Vent,
    Prop,
}

/// The vent G adds to a room that has none to copy: the size and cycle of
/// the ones Chapter 1 was first given.
const NEW_VENT: RoomHazardConfig = RoomHazardConfig {
    label: None,
    rect_h: RectH {
        x: 0.0,
        y: 0.0,
        width: 2.0,
        height: 1.5,
    },
    damage: 12.0,
    arm_s: 0.8,
    burn_s: 0.6,
    rest_s: 1.4,
    offset_s: 0.0,
};

/// A playtest setting: the rows of the settings Cmd+R opens.
pub(super) const PLAYTEST_ROWS: usize = 3;

#[derive(Message, Clone, Debug, PartialEq)]
pub(super) enum EditorAction {
    Save,
    /// Opens the playtest settings, which ask how the room is to be played.
    OpenPlaytest,
    Undo,
    Redo,
    Duplicate,
    Copy,
    Cut,
    Paste,
    SelectAll,
    ToggleFrame,
    ToggleSight,
    ToggleHelp,
    Add(NewItem),
    /// Tab: from the one selected item to the closest of its kind.
    SelectNextOfKind,
    Delete,
    /// The previous (-1) or next (1) room.
    Room(isize),
    FitRoom,
    /// Moves the inspector's focus up (-1) or down (1).
    MoveField(isize),
    /// Changes the focused inspector row by this many steps.
    StepField(f32),
    /// Focuses an inspector row and changes it.
    StepFieldAt(usize, f32),
    /// Focuses an inspector row, and opens it for typing if it takes text.
    EditField(usize),
    /// Opens the focused row for typing, if it takes text.
    TypeField,
    ClearField,
    ClearFieldAt(usize),
    /// Deselects, closes the help, then leaves (twice with unsaved edits).
    Escape,
    /// Hides or shows one layer of the room's backgrounds.
    ToggleLayer(BackgroundLayerRef),
    /// Shows every background layer of the room again.
    ShowAllLayers,
    /// Moves the playtest settings' focus up (-1) or down (1).
    PlaytestMove(isize),
    /// Focuses a playtest setting and changes it, one way or the other.
    PlaytestChange(usize, isize),
    PlaytestStart,
    PlaytestCancel,
}

impl EditorAction {
    /// Whether a button for this action is lit (a toggle that is on, or
    /// something waiting to be done), and whether it can be pressed at all.
    pub(super) fn looks(&self, editor: &RoomEditor, data: &GameData) -> (bool, bool) {
        let unsaved = || !editor.unsaved_rooms(data).is_empty() || editor.unsaved_hitboxes(data);
        match self {
            Self::ToggleFrame => (editor.overlays.viewport, true),
            Self::ToggleSight => (editor.overlays.sight, true),
            Self::ToggleHelp => (editor.help_pinned, true),
            Self::Save => (unsaved(), unsaved()),
            Self::Undo => (false, !editor.undo.is_empty()),
            Self::Redo => (false, !editor.redo.is_empty()),
            Self::Duplicate | Self::Copy | Self::Cut | Self::Delete => {
                (false, !editor.selected.is_empty())
            }
            Self::Paste => (false, editor.clipboard.is_some()),
            Self::EditField(row) => (*row == editor.field, true),
            Self::PlaytestChange(row, _) => (editor.playtest_row == Some(*row), true),
            Self::PlaytestStart => (true, true),
            Self::ShowAllLayers => {
                let hidden = data
                    .rooms
                    .get(&editor.room_id)
                    .is_some_and(|room| !room.hidden_background_layers.is_empty());
                (false, hidden)
            }
            _ => (false, true),
        }
    }
}

#[derive(SystemParam)]
pub(super) struct EditorActionResources<'w, 's> {
    actions: MessageReader<'w, 's, EditorAction>,
    editor: ResMut<'w, RoomEditor>,
    data: ResMut<'w, GameData>,
    session: ResMut<'w, RunSession>,
    start_room: ResMut<'w, StartRoom>,
    random: ResMut<'w, RandomState>,
    next_screen: ResMut<'w, NextState<GameScreen>>,
    camera: Query<'w, 's, (&'static mut Transform, &'static mut Projection), With<Camera2d>>,
}

pub(super) fn run_actions(resources: EditorActionResources) {
    let EditorActionResources {
        mut actions,
        mut editor,
        mut data,
        mut session,
        mut start_room,
        mut random,
        mut next_screen,
        mut camera,
    } = resources;
    for action in actions.read() {
        if editor.playtest_row.is_some() {
            // The settings take every request until they are answered.
            let playtest = &mut session.playtest;
            match action {
                EditorAction::PlaytestCancel | EditorAction::Escape => editor.playtest_row = None,
                EditorAction::PlaytestStart | EditorAction::OpenPlaytest => {
                    editor.playtest_row = None;
                    *start_room = StartRoom(editor.room_id);
                    next_screen.set(screens::reset_run(
                        &mut session,
                        &data,
                        *start_room,
                        &mut random,
                    ));
                }
                EditorAction::PlaytestMove(step) => {
                    editor.playtest_row = editor.playtest_row.map(|row| {
                        (row as isize + step).rem_euclid(PLAYTEST_ROWS as isize) as usize
                    });
                }
                EditorAction::PlaytestChange(row, step) => {
                    editor.playtest_row = Some(*row);
                    match row {
                        0 => playtest.infinite_health = !playtest.infinite_health,
                        1 => playtest.no_enemies = !playtest.no_enemies,
                        _ => {
                            let choices = STARTING_BONUSES;
                            let at = choices
                                .iter()
                                .position(|choice| *choice == playtest.starting_bonuses)
                                .unwrap_or(0) as isize;
                            playtest.starting_bonuses =
                                choices[(at + step).rem_euclid(choices.len() as isize) as usize];
                        }
                    }
                }
                _ => {}
            }
            continue;
        }
        if *action != EditorAction::Escape {
            editor.leave_armed = false;
        }
        // Where an added or pasted item lands: under the cursor, or in the
        // middle of the view when the cursor is on a panel.
        let view_center_h = camera
            .single()
            .map(|(transform, _)| transform.translation.truncate() / H_WORLD_UNITS)
            .unwrap_or_default();
        let at_h = editor
            .cursor_h
            .filter(|_| !editor.pointer_over_panel)
            .unwrap_or(view_center_h);
        match action {
            EditorAction::Save => editor.save(&mut data),
            EditorAction::OpenPlaytest => {
                // A room that fails validation would not load; the panel says why.
                if let Err(problem) = data.validate() {
                    editor.problem = Some(problem.to_string());
                } else {
                    editor.playtest_row = Some(0);
                    editor.text_entry = None;
                }
            }
            EditorAction::Undo => editor.step_history(&mut data, true),
            EditorAction::Redo => editor.step_history(&mut data, false),
            EditorAction::Duplicate => {
                let Some(copied) = data
                    .rooms
                    .get(&editor.room_id)
                    .and_then(|room| Clipboard::copy(room, &data.props, &editor.selected))
                else {
                    continue;
                };
                let mut duplicates = Vec::new();
                // Duplicates land one H to the right of the originals.
                editor.edit(&mut data, |room| {
                    duplicates = copied.paste(room, copied.origin + Vec2::X);
                });
                editor.selected = duplicates;
                editor.field = 0;
            }
            EditorAction::Copy | EditorAction::Cut => {
                let room = data.rooms.get(&editor.room_id);
                let Some(copied) =
                    room.and_then(|room| Clipboard::copy(room, &data.props, &editor.selected))
                else {
                    continue;
                };
                editor.clipboard = Some(copied);
                if *action == EditorAction::Cut {
                    let selection = editor.selected.clone();
                    editor.edit(&mut data, |room| EditorItem::remove_all(room, &selection));
                    editor.select(None);
                }
            }
            EditorAction::Paste => {
                let Some(copied) = editor.clipboard.clone() else {
                    continue;
                };
                let at = Vec2::new(snap(at_h.x, false), snap(at_h.y, false));
                let mut pasted = Vec::new();
                editor.edit(&mut data, |room| pasted = copied.paste(room, at));
                editor.selected = pasted;
                editor.field = 0;
            }
            EditorAction::SelectAll => {
                if let Some(room) = data.rooms.get(&editor.room_id) {
                    editor.selected = EditorItem::all(room)
                        .filter(|item| {
                            !matches!(
                                item,
                                EditorItem::PlayerSpawn | EditorItem::Exit | EditorItem::Bonus
                            )
                        })
                        .collect();
                    editor.field = 0;
                }
            }
            EditorAction::ToggleFrame => editor.overlays.viewport = !editor.overlays.viewport,
            EditorAction::ToggleSight => editor.overlays.sight = !editor.overlays.sight,
            EditorAction::ToggleHelp => editor.help_pinned = !editor.help_pinned,
            EditorAction::Add(new) => editor.add(&mut data, *new, at_h),
            EditorAction::SelectNextOfKind => {
                // From the one item selected to its nearest neighbor of the
                // same kind, measured middle to middle.
                let (&[selected], Some(room)) =
                    (editor.selected.as_slice(), data.rooms.get(&editor.room_id))
                else {
                    continue;
                };
                let middle = |item: EditorItem| {
                    item.bounds(room, &data.props).map(|rect| {
                        Vec2::new(rect.x + rect.width / 2.0, rect.y + rect.height / 2.0)
                    })
                };
                let Some(from) = middle(selected) else {
                    continue;
                };
                let closest = EditorItem::all(room)
                    .filter(|item| {
                        *item != selected
                            && std::mem::discriminant(item) == std::mem::discriminant(&selected)
                    })
                    .filter_map(|item| Some((item, middle(item)?.distance_squared(from))))
                    .min_by(|a, b| a.1.total_cmp(&b.1))
                    .map(|(item, _)| item);
                if closest.is_some() {
                    editor.select(closest);
                }
            }
            EditorAction::Delete => {
                if editor.selected.is_empty() {
                    continue;
                }
                let selection = editor.selected.clone();
                editor.edit(&mut data, |room| EditorItem::remove_all(room, &selection));
                editor.select(None);
            }
            EditorAction::Room(step) => {
                let room_ids = data.rooms.keys().copied().collect::<Vec<_>>();
                let next = room_ids
                    .iter()
                    .position(|id| *id == editor.room_id)
                    .and_then(|position| {
                        room_ids
                            .get(usize::try_from(position as isize + step).ok()?)
                            .copied()
                    });
                if let Some(room_id) = next {
                    editor.open_room(room_id);
                }
            }
            EditorAction::FitRoom => {
                let (Ok((mut transform, mut projection)), Some(room)) =
                    (camera.single_mut(), data.rooms.get(&editor.room_id))
                else {
                    continue;
                };
                let room_size_h = Vec2::new(room.size.width_h, room.size.height_h);
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
            }
            EditorAction::MoveField(step) => {
                let count = editor.fields().len().max(1) as isize;
                editor.field = (editor.field as isize + step).rem_euclid(count) as usize;
                editor.text_entry = None;
            }
            EditorAction::StepFieldAt(row, steps) => {
                editor.field = *row;
                editor.text_entry = None;
                editor.step_field(&mut data, *steps);
            }
            EditorAction::StepField(steps) => editor.step_field(&mut data, *steps),
            EditorAction::EditField(row) => {
                editor.field = *row;
                editor.text_entry = None;
                editor.start_typing(&data);
            }
            EditorAction::TypeField => editor.start_typing(&data),
            EditorAction::ClearFieldAt(row) => {
                editor.field = *row;
                editor.text_entry = None;
                editor.clear_field(&mut data);
            }
            EditorAction::ClearField => editor.clear_field(&mut data),
            EditorAction::Escape => {
                let unsaved =
                    !editor.unsaved_rooms(&data).is_empty() || editor.unsaved_hitboxes(&data);
                if editor.help_pinned {
                    editor.help_pinned = false;
                } else if !editor.selected.is_empty() {
                    editor.select(None);
                } else if !unsaved || editor.leave_armed {
                    next_screen.set(GameScreen::MainMenu);
                } else {
                    editor.leave_armed = true;
                }
            }
            EditorAction::ToggleLayer(layer) => {
                let layer = layer.clone();
                editor.edit(&mut data, |room| room.toggle_layer(layer));
            }
            EditorAction::ShowAllLayers => {
                editor.edit(&mut data, |room| room.hidden_background_layers.clear());
            }
            EditorAction::PlaytestMove(_)
            | EditorAction::PlaytestChange(..)
            | EditorAction::PlaytestStart
            | EditorAction::PlaytestCancel => {}
        }
    }
}

impl RoomEditor {
    /// Writes every changed room and the changed hitboxes. Saving is refused
    /// while the data is invalid, and a write that fails leaves the rest
    /// unsaved: either way the panel says why.
    fn save(&mut self, data: &mut GameData) {
        if let Err(problem) = data.validate() {
            self.problem = Some(problem.to_string());
            return;
        }
        for room_id in self.unsaved_rooms(data) {
            let Some(room) = data.rooms.get(&room_id) else {
                continue;
            };
            let path = RoomConfig::file_path(&data.asset_root, room_id);
            if let Err(error) = fs::write(&path, room.to_string()) {
                self.problem = Some(format!("Could not write {}: {error}", path.display()));
                return;
            }
            self.saved.insert(room_id, room.clone());
        }
        if self.unsaved_hitboxes(data) {
            let hitboxes = Hitboxes::of(data);
            if let Err(error) = hitboxes.save(&data.asset_root) {
                self.problem = Some(format!("Hitboxes not saved: {error}"));
                return;
            }
            self.saved_hitboxes = Some(hitboxes);
        }
    }

    /// Undoes (or redoes) one step, opening the room it changed.
    fn step_history(&mut self, data: &mut GameData, undo: bool) {
        let step = if undo {
            self.undo.pop()
        } else {
            self.redo.pop()
        };
        let Some(step) = step else {
            return;
        };
        let room_id = match &step {
            Snapshot::Room(room_id, _) => Some(*room_id),
            Snapshot::Hitboxes(_) => None,
        };
        if let Some(current) = step.restore(data) {
            if undo {
                self.redo.push(current);
            } else {
                self.undo.push(current);
            }
        }
        if let Some(room_id) = room_id.filter(|room_id| *room_id != self.room_id) {
            self.open_room(room_id);
        }
        self.view_dirty = true;
    }

    /// Adds a platform, decoration, enemy, vent or prop around `at_h` and
    /// selects it.
    fn add(&mut self, data: &mut GameData, new: NewItem, at_h: Vec2) {
        let Some(room) = data.rooms.get(&self.room_id) else {
            return;
        };
        let mut added = None;
        if new == NewItem::Enemy {
            let type_id = self
                .enemy_type
                .clone()
                .or_else(|| data.enemies.types.keys().next().cloned());
            let Some(type_id) = type_id else {
                return;
            };
            self.edit(data, |room| {
                room.enemy_spawns.push(EnemySpawnConfig {
                    label: None,
                    type_id,
                    position_h: SpawnPosition {
                        x_h: snap(at_h.x, false),
                        y_h: snap(at_h.y, false),
                    },
                    facing: Facing::default(),
                    armor_layers: 0,
                    wave: 0,
                });
                added = Some(EditorItem::Enemy(room.enemy_spawns.len() - 1));
            });
            self.select(added);
            return;
        }
        if new == NewItem::Prop {
            // The prop placed or picked last, else the catalog's first,
            // standing on the cursor.
            let Some(prop) = self
                .prop
                .clone()
                .or_else(|| data.props.props.keys().next().cloned())
            else {
                return;
            };
            self.edit(data, |room| {
                room.props.push(PropConfig {
                    label: None,
                    prop,
                    position_h: SpawnPosition {
                        x_h: snap(at_h.x, false),
                        y_h: snap(at_h.y, false),
                    },
                    layer: DecorationLayer::default(),
                });
                added = Some(EditorItem::Prop(room.props.len() - 1));
            });
            self.select(added);
            return;
        }
        if new == NewItem::Vent {
            // A copy of the room's last vent, so a row of them is laid
            // quickly, standing with its foot where the cursor is.
            let mut vent = room.hazards.last().cloned().unwrap_or(NEW_VENT);
            vent.label = None;
            vent.rect_h.x = snap(at_h.x - vent.rect_h.width / 2.0, false);
            vent.rect_h.y = snap(at_h.y, false);
            self.edit(data, |room| {
                room.hazards.push(vent);
                added = Some(EditorItem::Hazard(room.hazards.len() - 1));
            });
            self.select(added);
            return;
        }
        let decoration = new == NewItem::Decoration;
        let (width, height) = if decoration {
            (4.0, 3.0)
        } else {
            (3.0, SNAP_H)
        };
        // The fill picked last, else the one this room already paints this
        // kind of item with, else the catalog's first material.
        let same_kind = if decoration {
            room.decorations.first().map(|decoration| &decoration.fill)
        } else {
            room.platforms.first().map(|block| &block.fill)
        };
        let Some(fill) = self.fill.as_ref().or(same_kind).cloned().or_else(|| {
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
            x: snap(at_h.x - width / 2.0, false),
            y: snap(at_h.y - height / 2.0, false),
            width,
            height,
        };
        self.edit(data, |room| {
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
                    rotation_deg: 0.0,
                });
                EditorItem::Platform(room.platforms.len() - 1)
            });
        });
        self.select(added);
    }

    /// Changes the focused inspector row by `steps` (negative to decrease).
    fn step_field(&mut self, data: &mut GameData, steps: f32) {
        let Some(field) = self.fields().get(self.field).copied() else {
            return;
        };
        let direction = steps.signum();
        let selection = self.selected.clone();
        let item = self.primary();
        match field {
            Field::Room => {
                let room_ids = data.rooms.keys().copied().collect::<Vec<_>>();
                let next =
                    room_ids
                        .iter()
                        .position(|id| *id == self.room_id)
                        .and_then(|position| {
                            let next = position as isize + direction as isize;
                            room_ids.get(usize::try_from(next).ok()?).copied()
                        });
                if let Some(room_id) = next {
                    self.open_room(room_id);
                }
            }
            Field::Attack => {
                let count = self.selected_enemy_type(data).map_or(0, |type_id| {
                    let enemy = &data.enemies.types[&type_id];
                    usize::from(enemy.attack.is_some()) + enemy.attacks.len()
                });
                self.attack = (self.attack as isize + direction as isize)
                    .rem_euclid(count.max(1) as isize) as usize;
            }
            field if field.is_hitbox() => {
                let type_id = self.selected_enemy_type(data);
                let attack = self.attack;
                self.edit_hitboxes(data, |data| {
                    field.adjust_hitbox(data, type_id.as_ref(), attack, steps);
                });
            }
            field => {
                let snapshot = &*data;
                let Some(mut room) = snapshot.rooms.get(&self.room_id).cloned() else {
                    return;
                };
                field.adjust(&mut room, &selection, snapshot, steps);
                self.edit(data, |edited| *edited = room);
                if let Some(EditorItem::Enemy(index)) = item
                    && field == Field::EnemyType
                {
                    self.enemy_type = data
                        .rooms
                        .get(&self.room_id)
                        .and_then(|room| room.enemy_spawns.get(index))
                        .map(|spawn| spawn.type_id.clone());
                }
                if let Some(EditorItem::Prop(index)) = item
                    && field == Field::PropType
                {
                    self.prop = data
                        .rooms
                        .get(&self.room_id)
                        .and_then(|room| room.props.get(index))
                        .map(|placed| placed.prop.clone());
                }
                if field == Field::Fill {
                    self.fill = data
                        .rooms
                        .get(&self.room_id)
                        .and_then(|room| item.and_then(|item| item.fill(room)))
                        .cloned();
                }
            }
        }
    }

    /// Opens the focused row for typing, holding its current text.
    fn start_typing(&mut self, data: &GameData) {
        let Some(field) = self
            .fields()
            .get(self.field)
            .copied()
            .filter(|field| field.is_text())
        else {
            return;
        };
        let item = self.primary();
        self.text_entry = Some(
            data.rooms
                .get(&self.room_id)
                .and_then(|room| field.text(room, item))
                .unwrap_or_default(),
        );
    }

    fn clear_field(&mut self, data: &mut GameData) {
        if let Some(field) = self.fields().get(self.field).copied() {
            let item = self.primary();
            self.edit(data, |room| field.clear(room, item));
        }
    }
}
