//! Debug room editor (debug builds only). Opened with D from the main menu or
//! the pause menu, it edits the rooms in `GameData` in place, so a playtest
//! runs the edits right away, and saves them back to `rooms/room_NNN.yaml`.

use super::*;
use crate::data::{MaterialId, RectH, RoomConfig};

mod clipboard;
mod draw;
mod inspector;
mod keyboard;
mod palette;
mod pointer;

use clipboard::Clipboard;
use inspector::Field;

/// Grid step, in H, that dragged and nudged positions snap to.
const SNAP_H: f32 = 0.25;
/// Snap step while Alt (Option) is held, for fine placement.
const FINE_SNAP_H: f32 = 0.05;
const MIN_ZOOM: f32 = 0.4;
/// The modifier shortcuts are shown with; Ctrl and Cmd both work everywhere.
const COMMAND_KEY: &str = if cfg!(target_os = "macos") {
    "Cmd"
} else {
    "Ctrl"
};
const MAX_ZOOM: f32 = 4.0;

pub(super) struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RoomEditor>()
            .add_systems(
                OnEnter(GameScreen::Editor),
                (world::cleanup_room, audio::stop_chapter_music, enter_editor).chain(),
            )
            .add_systems(
                OnExit(GameScreen::Editor),
                (world::cleanup_room, exit_editor),
            )
            .add_systems(
                Update,
                (
                    keyboard::type_text,
                    keyboard::handle_editor_keys,
                    palette::handle_palette,
                    pointer::handle_pointer,
                    draw::refresh_room_view,
                    draw::draw_overlay,
                    draw::update_panels,
                    palette::highlight_swatches,
                )
                    .chain()
                    .run_if(in_state(GameScreen::Editor)),
            );
    }
}

/// Something in a room the editor can select.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum EditorItem {
    PlayerSpawn,
    Exit,
    Enemy(usize),
    Platform(usize),
    Terrain(usize),
    Decoration(usize),
}

impl EditorItem {
    /// The item's outline in H: a block's rectangle, or a body standing on a spawn.
    fn bounds(self, room: &RoomConfig) -> Option<RectH> {
        let body = |x_h: f32, y_h: f32| RectH {
            x: x_h - PLAYER_COLLISION_WIDTH_H / 2.0,
            y: y_h,
            width: PLAYER_COLLISION_WIDTH_H,
            height: 1.0,
        };
        match self {
            Self::PlayerSpawn => Some(body(room.player_spawn.x_h, room.player_spawn.y_h)),
            Self::Exit => Some(body(room.exit.x_h, room.exit.y_h)),
            Self::Enemy(index) => room
                .enemy_spawns
                .get(index)
                .map(|spawn| body(spawn.position_h.x_h, spawn.position_h.y_h)),
            Self::Platform(index) => room.platforms.get(index).map(|block| block.rect_h),
            Self::Terrain(index) => room.terrain.get(index).map(|block| block.rect_h),
            Self::Decoration(index) => room
                .decorations
                .get(index)
                .map(|decoration| decoration.rect_h),
        }
    }

    /// The point a drag moves: a block's lower-left corner or a spawn's feet.
    fn origin(self, room: &RoomConfig) -> Option<Vec2> {
        match self {
            Self::PlayerSpawn => Some(Vec2::new(room.player_spawn.x_h, room.player_spawn.y_h)),
            Self::Exit => Some(Vec2::new(room.exit.x_h, room.exit.y_h)),
            Self::Enemy(index) => room
                .enemy_spawns
                .get(index)
                .map(|spawn| Vec2::new(spawn.position_h.x_h, spawn.position_h.y_h)),
            Self::Platform(_) | Self::Terrain(_) | Self::Decoration(_) => {
                self.bounds(room).map(|rect| Vec2::new(rect.x, rect.y))
            }
        }
    }

    fn set_origin(self, room: &mut RoomConfig, origin: Vec2) {
        match self {
            Self::PlayerSpawn => (room.player_spawn.x_h, room.player_spawn.y_h) = origin.into(),
            Self::Exit => (room.exit.x_h, room.exit.y_h) = origin.into(),
            Self::Enemy(index) => {
                if let Some(spawn) = room.enemy_spawns.get_mut(index) {
                    (spawn.position_h.x_h, spawn.position_h.y_h) = origin.into();
                }
            }
            Self::Platform(index) => {
                if let Some(block) = room.platforms.get_mut(index) {
                    (block.rect_h.x, block.rect_h.y) = origin.into();
                }
            }
            Self::Terrain(_) | Self::Decoration(_) => {
                if let Some(rect) = self.rect_mut(room) {
                    (rect.x, rect.y) = origin.into();
                }
            }
        }
    }

    fn rect_mut(self, room: &mut RoomConfig) -> Option<&mut RectH> {
        match self {
            Self::Platform(index) => room.platforms.get_mut(index).map(|block| &mut block.rect_h),
            Self::Terrain(index) => room.terrain.get_mut(index).map(|block| &mut block.rect_h),
            Self::Decoration(index) => room
                .decorations
                .get_mut(index)
                .map(|decoration| &mut decoration.rect_h),
            Self::PlayerSpawn | Self::Exit | Self::Enemy(_) => None,
        }
    }

    /// The terrain material a block or decoration is drawn with.
    fn material(self, room: &RoomConfig) -> Option<&MaterialId> {
        match self {
            Self::Platform(index) => room.platforms.get(index).map(|block| &block.material),
            Self::Terrain(index) => room.terrain.get(index).map(|block| &block.material),
            Self::Decoration(index) => room
                .decorations
                .get(index)
                .map(|decoration| &decoration.material),
            Self::PlayerSpawn | Self::Exit | Self::Enemy(_) => None,
        }
    }

    fn set_material(self, room: &mut RoomConfig, material: &MaterialId) {
        let slot = match self {
            Self::Platform(index) => room
                .platforms
                .get_mut(index)
                .map(|block| &mut block.material),
            Self::Terrain(index) => room.terrain.get_mut(index).map(|block| &mut block.material),
            Self::Decoration(index) => room
                .decorations
                .get_mut(index)
                .map(|decoration| &mut decoration.material),
            Self::PlayerSpawn | Self::Exit | Self::Enemy(_) => None,
        };
        if let Some(slot) = slot {
            slot.clone_from(material);
        }
    }

    /// Removes `items` from the room; the player spawn and exit stay.
    fn remove_all(room: &mut RoomConfig, items: &[Self]) {
        let mut items = items.to_vec();
        // Later indices first, so removing one keeps the others valid.
        items.sort_by_key(|item| {
            std::cmp::Reverse(match item {
                Self::Terrain(index)
                | Self::Platform(index)
                | Self::Enemy(index)
                | Self::Decoration(index) => *index,
                Self::PlayerSpawn | Self::Exit => 0,
            })
        });
        for item in items {
            match item {
                Self::Terrain(index) if index < room.terrain.len() => {
                    room.terrain.remove(index);
                }
                Self::Platform(index) if index < room.platforms.len() => {
                    room.platforms.remove(index);
                }
                Self::Enemy(index) if index < room.enemy_spawns.len() => {
                    room.enemy_spawns.remove(index);
                }
                Self::Decoration(index) if index < room.decorations.len() => {
                    room.decorations.remove(index);
                }
                _ => {}
            }
        }
    }

    /// Every item of the room, blocks first.
    fn all(room: &RoomConfig) -> impl Iterator<Item = Self> {
        (0..room.terrain.len())
            .map(Self::Terrain)
            .chain((0..room.platforms.len()).map(Self::Platform))
            .chain((0..room.decorations.len()).map(Self::Decoration))
            .chain((0..room.enemy_spawns.len()).map(Self::Enemy))
            .chain([Self::PlayerSpawn, Self::Exit])
    }

    /// Spawns stand on block tops; blocks move freely.
    fn stands_on_surface(self) -> bool {
        matches!(self, Self::PlayerSpawn | Self::Exit | Self::Enemy(_))
    }

    fn describe(self, room: &RoomConfig) -> String {
        let label = match self {
            Self::Enemy(index) => room
                .enemy_spawns
                .get(index)
                .and_then(|spawn| spawn.label.as_deref()),
            Self::Platform(index) => room
                .platforms
                .get(index)
                .and_then(|block| block.label.as_deref()),
            Self::Terrain(index) => room
                .terrain
                .get(index)
                .and_then(|block| block.label.as_deref()),
            Self::Decoration(index) => room
                .decorations
                .get(index)
                .and_then(|decoration| decoration.label.as_deref()),
            Self::PlayerSpawn | Self::Exit => None,
        };
        let name = match self {
            Self::PlayerSpawn => String::from("player spawn"),
            Self::Exit => String::from("exit"),
            Self::Enemy(index) => room
                .enemy_spawns
                .get(index)
                .map_or_else(String::new, |spawn| {
                    format!("enemy #{} {}", index + 1, spawn.type_id)
                }),
            Self::Platform(index) => format!("platform #{}", index + 1),
            Self::Terrain(index) => format!("terrain #{}", index + 1),
            Self::Decoration(index) => room
                .decorations
                .get(index)
                .map_or_else(String::new, |decoration| {
                    format!("decoration #{} ({})", index + 1, decoration.layer)
                }),
        };
        match label {
            Some(label) => format!("{name} \"{label}\""),
            None => name,
        }
    }

    /// The item under `point`: spawns first, then the smallest block or
    /// decoration.
    fn at(room: &RoomConfig, point: Vec2) -> Option<Self> {
        let contains = |item: &Self| {
            item.bounds(room).is_some_and(|rect| {
                Rect::new(rect.x, rect.y, rect.right(), rect.top()).contains(point)
            })
        };
        let spawns = [Self::PlayerSpawn, Self::Exit]
            .into_iter()
            .chain((0..room.enemy_spawns.len()).rev().map(Self::Enemy));
        let blocks = (0..room.platforms.len())
            .map(Self::Platform)
            .chain((0..room.terrain.len()).map(Self::Terrain))
            .chain((0..room.decorations.len()).map(Self::Decoration));
        spawns.into_iter().find(contains).or_else(|| {
            blocks.filter(contains).min_by(|a, b| {
                let area = |item: &Self| {
                    item.bounds(room)
                        .map_or(f32::INFINITY, |rect| rect.width * rect.height)
                };
                area(a).total_cmp(&area(b))
            })
        })
    }
}

/// Which edges of a block a resize drag moves.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Edges {
    left: bool,
    right: bool,
    bottom: bool,
    top: bool,
}

enum Drag {
    /// Moving the selection by dragging `item`, one of its items; `grab` is
    /// the cursor's offset from that item's origin, and `starts` holds each
    /// selected item's origin when the drag began.
    Move {
        item: EditorItem,
        grab: Vec2,
        starts: Vec<(EditorItem, Vec2)>,
        before: RoomConfig,
    },
    Resize {
        item: EditorItem,
        edges: Edges,
        before: RoomConfig,
    },
    /// Panning the view; `anchor` is the world point (in H) held under the cursor.
    Pan { anchor: Vec2 },
    /// Drawing a selection box from `start` (in H) to the cursor.
    Marquee { start: Vec2 },
}

#[derive(Resource)]
struct RoomEditor {
    room_id: u16,
    /// The selected items; the last one is the primary selection the
    /// inspector describes.
    selected: Vec<EditorItem>,
    hovered: Option<EditorItem>,
    drag: Option<Drag>,
    /// Inspector row Up/Down moves between.
    field: usize,
    /// Text being typed into a label or the room description.
    text_entry: Option<String>,
    /// The typing handled this frame's keys, so shortcuts ignore them.
    keys_consumed: bool,
    /// The mouse position in the room, in H.
    cursor_h: Option<Vec2>,
    undo: Vec<(u16, RoomConfig)>,
    redo: Vec<(u16, RoomConfig)>,
    /// Each edited room as it is on disk, to tell which rooms are unsaved.
    saved: BTreeMap<u16, RoomConfig>,
    /// Enemy type the next added enemy uses: the last one placed or picked.
    enemy_type: Option<EnemyTypeId>,
    /// Material new blocks and decorations use: the last one picked.
    material: Option<MaterialId>,
    /// Items copied with Cmd+C or Cmd+X, ready to paste into any room.
    clipboard: Option<Clipboard>,
    /// The cursor is over an editor panel, so clicks don't reach the room.
    pointer_over_panel: bool,
    /// The editor opened this frame: the key that opened it (D in a menu)
    /// must not also act as an editor shortcut.
    just_opened: bool,
    status: String,
    /// Validation result of the edited data, refreshed after each edit.
    problem: Option<String>,
    view_dirty: bool,
    /// Escape was pressed once with unsaved rooms; a second press leaves.
    leave_armed: bool,
}

impl Default for RoomEditor {
    fn default() -> Self {
        Self {
            room_id: 1,
            selected: Vec::new(),
            hovered: None,
            drag: None,
            field: 0,
            text_entry: None,
            keys_consumed: false,
            cursor_h: None,
            undo: Vec::new(),
            redo: Vec::new(),
            saved: BTreeMap::new(),
            enemy_type: None,
            material: None,
            clipboard: None,
            pointer_over_panel: false,
            just_opened: false,
            status: String::new(),
            problem: None,
            view_dirty: true,
            leave_armed: false,
        }
    }
}

impl RoomEditor {
    /// Applies `change` to the edited room as one undoable step.
    fn edit(&mut self, data: &mut GameData, change: impl FnOnce(&mut RoomConfig)) {
        let Some(room) = data.rooms.get_mut(&self.room_id) else {
            return;
        };
        let before = room.clone();
        change(room);
        if *room != before {
            self.record(before);
        }
    }

    /// Records `before`, the edited room's state before a change, for undo.
    fn record(&mut self, before: RoomConfig) {
        self.saved
            .entry(self.room_id)
            .or_insert_with(|| before.clone());
        self.undo.push((self.room_id, before));
        self.redo.clear();
        self.view_dirty = true;
        self.leave_armed = false;
    }

    fn unsaved_rooms(&self, data: &GameData) -> Vec<u16> {
        self.saved
            .iter()
            .filter(|(room_id, saved)| data.rooms.get(room_id) != Some(saved))
            .map(|(room_id, _)| *room_id)
            .collect()
    }

    /// Opens another room, keeping the edits made so far in memory.
    fn open_room(&mut self, room_id: u16) {
        self.room_id = room_id;
        self.selected.clear();
        self.hovered = None;
        self.drag = None;
        self.field = 0;
        self.text_entry = None;
        self.view_dirty = true;
        self.leave_armed = false;
    }

    fn fields(&self) -> &'static [Field] {
        Field::for_selection(&self.selected)
    }

    /// The item the inspector describes: the last one selected.
    fn primary(&self) -> Option<EditorItem> {
        self.selected.last().copied()
    }

    /// Selects only `item`, or nothing.
    fn select(&mut self, item: Option<EditorItem>) {
        if self.selected.last().copied() != item || self.selected.len() > 1 {
            self.field = 0;
        }
        self.selected = item.into_iter().collect();
    }
}

/// The highest block top under `x_h` at or below `y_h` (with half an H of
/// slack), where a dragged or added spawn's feet land.
fn floor_under(room: &RoomConfig, x_h: f32, y_h: f32) -> Option<f32> {
    room.terrain
        .iter()
        .chain(&room.platforms)
        .filter(|block| block.rect_h.covers_x(x_h) && block.rect_h.top() <= y_h + 0.5)
        .map(|block| block.rect_h.top())
        .max_by(f32::total_cmp)
}

/// `value` rounded to the editor grid (finer while Shift is held).
fn snap(value: f32, fine: bool) -> f32 {
    let steps_per_h = if fine {
        1.0 / FINE_SNAP_H
    } else {
        1.0 / SNAP_H
    };
    // Adding 0.0 turns -0.0 into 0.0, which the room files would show as `-0`.
    (value * steps_per_h).round() / steps_per_h + 0.0
}

/// The inspector panel, rewritten every frame.
#[derive(Component)]
struct EditorPanel;

/// Every editor panel, despawned when the editor closes.
#[derive(Component)]
struct EditorUi;

fn enter_editor(
    mut commands: Commands,
    mut editor: ResMut<RoomEditor>,
    session: Res<RunSession>,
    data: Res<GameData>,
    asset_server: Res<AssetServer>,
    mut camera: Query<&mut Transform, With<Camera2d>>,
) {
    editor.open_room(session.room_id);
    editor.just_opened = true;
    editor.status =
        format!("Editing a copy in memory: {COMMAND_KEY}+S writes it to the YAML file.");
    if let (Ok(mut transform), Some(room)) = (camera.single_mut(), data.rooms.get(&session.room_id))
    {
        transform.translation.x = room.player_spawn.x_h * H_WORLD_UNITS;
        transform.translation.y =
            (room.player_spawn.y_h + data.gameplay.viewport.height_h / 2.0 - 1.0) * H_WORLD_UNITS;
    }
    let panel_node =
        |left: Option<Val>, right: Option<Val>, top: Option<Val>, bottom: Option<Val>| Node {
            position_type: PositionType::Absolute,
            left: left.unwrap_or(Val::Auto),
            right: right.unwrap_or(Val::Auto),
            top: top.unwrap_or(Val::Auto),
            bottom: bottom.unwrap_or(Val::Auto),
            padding: UiRect::all(px(10)),
            ..default()
        };
    let font = TextFont {
        font_size: bevy::text::FontSize::Px(15.0),
        ..default()
    };
    commands.spawn((
        Node {
            max_width: px(560),
            ..panel_node(Some(px(12)), None, Some(px(12)), None)
        },
        Text::new(""),
        font.clone(),
        TextColor(Color::WHITE),
        BackgroundColor(Color::srgba(0.02, 0.025, 0.05, 0.82)),
        Pickable::IGNORE,
        EditorPanel,
        EditorUi,
    ));
    commands.spawn((
        panel_node(None, Some(px(12)), None, Some(px(12))),
        Text::new(format!(
            "Click select | Shift+click add/remove | drag empty space to box-select (Shift: add)\n\
             Drag moves the selection (Alt: fine grid) | drag a selected block's edge to resize\n\
             Two-finger scroll or right/middle drag pan | pinch or wheel zoom | Home fit room\n\
             At cursor: T terrain | P platform | D decoration | E enemy | click a material to paint\n\
             Del delete | {COMMAND_KEY}+D duplicate | {COMMAND_KEY}+C copy | {COMMAND_KEY}+X cut | \
             {COMMAND_KEY}+V paste at cursor | {COMMAND_KEY}+A select all\n\
             Up/Down field | Left/Right change (Shift x4) | Enter type text | C clear\n\
             PgUp/PgDn room | {COMMAND_KEY}+Z undo | {COMMAND_KEY}+Shift+Z or {COMMAND_KEY}+Y redo | \
             {COMMAND_KEY}+S save | {COMMAND_KEY}+R playtest\n\
             Esc deselect, then back to the menu",
        )),
        font,
        TextColor(Color::srgb(0.78, 0.8, 0.86)),
        BackgroundColor(Color::srgba(0.02, 0.025, 0.05, 0.82)),
        Pickable::IGNORE,
        EditorUi,
    ));
    palette::spawn_palette(&mut commands, &data, &asset_server);
}

fn exit_editor(
    mut commands: Commands,
    panels: Query<Entity, With<EditorUi>>,
    mut projection: Query<&mut Projection, With<Camera2d>>,
) {
    for entity in &panels {
        commands.entity(entity).despawn();
    }
    if let Ok(mut projection) = projection.single_mut()
        && let Projection::Orthographic(orthographic) = &mut *projection
    {
        orthographic.scale = 1.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn clicks_pick_spawns_first_then_the_smallest_block_under_the_cursor() {
        let asset_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/assets");
        let data = GameData::load(&asset_root).expect("Chapter 1 YAML should be valid");
        let room = &data.rooms[&3];

        let feet = Vec2::new(room.player_spawn.x_h, room.player_spawn.y_h + 0.5);
        assert_eq!(EditorItem::at(room, feet), Some(EditorItem::PlayerSpawn));
        // Room 3's step (x 28..30) sits on the floor (x 0..48): both cover
        // this point, and the step is the smaller block.
        let step = room
            .terrain
            .iter()
            .position(|block| block.label.as_deref() == Some("step"))
            .map(EditorItem::Terrain);
        assert_eq!(EditorItem::at(room, Vec2::new(29.0, 1.0)), step);
        assert_eq!(EditorItem::at(room, Vec2::new(-5.0, -5.0)), None);
    }

    #[test]
    fn dragged_spawns_land_on_the_highest_block_top_below_them() {
        let asset_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/assets");
        let data = GameData::load(&asset_root).expect("Chapter 1 YAML should be valid");
        let room = &data.rooms[&3];

        // Above room 3's west walkway (x 13..27, top 7) and the floor (top 1).
        assert_eq!(floor_under(room, 20.0, 9.0), Some(7.0));
        assert_eq!(floor_under(room, 20.0, 5.0), Some(1.0));
        assert_eq!(snap(2.37, false), 2.25);
        assert_eq!(snap(2.37, true), 2.35);
        assert!(snap(-0.1, false).is_sign_positive());
    }

    #[test]
    fn stat_overrides_step_from_the_type_default_and_clear_back_to_it() {
        let asset_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/assets");
        let data = GameData::load(&asset_root).expect("Chapter 1 YAML should be valid");
        let mut room = data.rooms[&1].clone();
        let enemy = Some(EditorItem::Enemy(0));
        let default_health = data.enemies.types[&room.enemy_spawns[0].type_id].health;

        Field::Health.adjust(&mut room, &[EditorItem::Enemy(0)], &data, 2.0);
        let overrides = room.enemy_spawns[0]
            .stat_overrides
            .clone()
            .unwrap_or_default();
        assert_eq!(overrides.health, Some(default_health + 10.0));

        Field::Health.clear(&mut room, enemy);
        assert_eq!(room.enemy_spawns[0].stat_overrides, None);
        assert_eq!(room, data.rooms[&1]);
    }

    #[test]
    fn a_copied_selection_pastes_offset_paints_together_and_deletes_together() {
        let asset_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/assets");
        let data = GameData::load(&asset_root).expect("Chapter 1 YAML should be valid");
        let mut room = data.rooms[&3].clone();
        let selection = [
            EditorItem::Terrain(1),
            EditorItem::Platform(0),
            EditorItem::Enemy(0),
            EditorItem::PlayerSpawn,
        ];
        let copied = Clipboard::copy(&room, &selection).expect("blocks and enemies copy");
        assert_eq!(copied.len(), 3, "the player spawn is not copied");

        let offset = Vec2::new(5.0, 2.0);
        let pasted = copied.paste(&mut room, copied.origin + offset);
        assert_eq!(pasted.len(), 3);
        for (original, copy) in selection.iter().zip(&pasted) {
            let moved = copy.origin(&room).zip(original.origin(&room));
            assert_eq!(moved.map(|(copy, original)| copy - original), Some(offset));
        }

        Field::Material.adjust(&mut room, &pasted, &data, 1.0);
        let painted = pasted[0].material(&room).cloned();
        assert_ne!(painted.as_ref(), selection[0].material(&room));
        assert_eq!(pasted[1].material(&room).cloned(), painted);

        EditorItem::remove_all(&mut room, &pasted);
        assert_eq!(room, data.rooms[&3], "only the pasted items were painted");
    }
}
