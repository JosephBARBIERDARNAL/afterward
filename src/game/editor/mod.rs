//! Debug room editor (debug builds only). Opened with D from the main menu or
//! the pause menu, it edits the rooms in `GameData` in place, so a playtest
//! runs the edits right away, and saves them back to `rooms/room_NNN.yaml`.

use super::*;
use crate::data::{Fill, PlatformConfig, RectH, RoomConfig};

mod clipboard;
mod draw;
mod hitboxes;
mod inspector;
mod keyboard;
mod overlays;
mod palette;
mod pointer;

use clipboard::Clipboard;
use hitboxes::Hitboxes;
use inspector::Field;
use overlays::Overlays;

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
/// What the playtest settings offer a run to open with: no bonus, a few at
/// random, or every one of them.
const STARTING_BONUSES: [usize; 3] = [0, 5, usize::MAX];

pub(super) struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RoomEditor>()
            .add_systems(
                OnEnter(GameScreen::Editor),
                (world::cleanup_room, audio::stop_run_audio, enter_editor).chain(),
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
                    // Places the redrawn background layers on the camera before
                    // the frame renders, so an edit doesn't flash them at y = 0.
                    camera::update_background_parallax,
                    draw::draw_overlay,
                    overlays::draw_overlays,
                    draw::update_panels,
                    draw::show_playtest_settings,
                    draw::show_help_while_h_is_held,
                    palette::refresh_palette,
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
    /// Where the room's bonus item lies once every enemy is down.
    Bonus,
    Enemy(usize),
    Platform(usize),
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
            Self::Bonus => Some(body(room.bonus.x_h, room.bonus.y_h)),
            Self::Enemy(index) => room
                .enemy_spawns
                .get(index)
                .map(|spawn| body(spawn.position_h.x_h, spawn.position_h.y_h)),
            Self::Platform(index) => room.platforms.get(index).map(|block| block.rect_h),
            Self::Decoration(index) => room
                .decorations
                .get(index)
                .map(|decoration| decoration.rect_h),
        }
    }

    /// The item's outline in H, corner by corner and counter-clockwise: its
    /// `bounds`, turned as the room turns it when it is a platform.
    fn corners(self, room: &RoomConfig) -> Option<[Vec2; 4]> {
        match self {
            Self::Platform(index) => room.platforms.get(index).map(PlatformConfig::corners_h),
            _ => self.bounds(room).map(|rect| {
                [
                    Vec2::new(rect.x, rect.y),
                    Vec2::new(rect.right(), rect.y),
                    Vec2::new(rect.right(), rect.top()),
                    Vec2::new(rect.x, rect.top()),
                ]
            }),
        }
    }

    /// Whether this is a platform its room turned. Its edges no longer lie
    /// where `bounds` says, so they are not dragged: it is resized from the
    /// inspector.
    fn is_turned(self, room: &RoomConfig) -> bool {
        matches!(self, Self::Platform(index)
            if room.platforms.get(index).is_some_and(|block| block.rotation_deg != 0.0))
    }

    /// The point a drag moves: a block's lower-left corner or a spawn's feet.
    fn origin(self, room: &RoomConfig) -> Option<Vec2> {
        match self {
            Self::PlayerSpawn => Some(Vec2::new(room.player_spawn.x_h, room.player_spawn.y_h)),
            Self::Exit => Some(Vec2::new(room.exit.x_h, room.exit.y_h)),
            Self::Bonus => Some(Vec2::new(room.bonus.x_h, room.bonus.y_h)),
            Self::Enemy(index) => room
                .enemy_spawns
                .get(index)
                .map(|spawn| Vec2::new(spawn.position_h.x_h, spawn.position_h.y_h)),
            Self::Platform(_) | Self::Decoration(_) => {
                self.bounds(room).map(|rect| Vec2::new(rect.x, rect.y))
            }
        }
    }

    fn set_origin(self, room: &mut RoomConfig, origin: Vec2) {
        match self {
            Self::PlayerSpawn => (room.player_spawn.x_h, room.player_spawn.y_h) = origin.into(),
            Self::Exit => (room.exit.x_h, room.exit.y_h) = origin.into(),
            Self::Bonus => (room.bonus.x_h, room.bonus.y_h) = origin.into(),
            Self::Enemy(index) => {
                if let Some(spawn) = room.enemy_spawns.get_mut(index) {
                    (spawn.position_h.x_h, spawn.position_h.y_h) = origin.into();
                }
            }
            Self::Platform(_) | Self::Decoration(_) => {
                if let Some(rect) = self.rect_mut(room) {
                    (rect.x, rect.y) = origin.into();
                }
            }
        }
    }

    fn rect_mut(self, room: &mut RoomConfig) -> Option<&mut RectH> {
        match self {
            Self::Platform(index) => room.platforms.get_mut(index).map(|block| &mut block.rect_h),
            Self::Decoration(index) => room
                .decorations
                .get_mut(index)
                .map(|decoration| &mut decoration.rect_h),
            Self::PlayerSpawn | Self::Exit | Self::Bonus | Self::Enemy(_) => None,
        }
    }

    /// What a platform or decoration is painted with.
    fn fill(self, room: &RoomConfig) -> Option<&Fill> {
        match self {
            Self::Platform(index) => room.platforms.get(index).map(|block| &block.fill),
            Self::Decoration(index) => room
                .decorations
                .get(index)
                .map(|decoration| &decoration.fill),
            Self::PlayerSpawn | Self::Exit | Self::Bonus | Self::Enemy(_) => None,
        }
    }

    fn set_fill(self, room: &mut RoomConfig, fill: &Fill) {
        let slot = match self {
            Self::Platform(index) => room.platforms.get_mut(index).map(|block| &mut block.fill),
            Self::Decoration(index) => room
                .decorations
                .get_mut(index)
                .map(|decoration| &mut decoration.fill),
            Self::PlayerSpawn | Self::Exit | Self::Bonus | Self::Enemy(_) => None,
        };
        if let Some(slot) = slot {
            slot.clone_from(fill);
        }
    }

    /// Removes `items` from the room; the player spawn, exit and bonus stay.
    fn remove_all(room: &mut RoomConfig, items: &[Self]) {
        let mut items = items.to_vec();
        // Later indices first, so removing one keeps the others valid.
        items.sort_by_key(|item| {
            std::cmp::Reverse(match item {
                Self::Platform(index) | Self::Enemy(index) | Self::Decoration(index) => *index,
                Self::PlayerSpawn | Self::Exit | Self::Bonus => 0,
            })
        });
        for item in items {
            match item {
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
        (0..room.platforms.len())
            .map(Self::Platform)
            .chain((0..room.decorations.len()).map(Self::Decoration))
            .chain((0..room.enemy_spawns.len()).map(Self::Enemy))
            .chain([Self::PlayerSpawn, Self::Exit, Self::Bonus])
    }

    /// Spawns are a body's size; only blocks and decorations resize.
    fn is_body(self) -> bool {
        matches!(
            self,
            Self::PlayerSpawn | Self::Exit | Self::Bonus | Self::Enemy(_)
        )
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
            Self::Decoration(index) => room
                .decorations
                .get(index)
                .and_then(|decoration| decoration.label.as_deref()),
            Self::PlayerSpawn | Self::Exit | Self::Bonus => None,
        };
        let name = match self {
            Self::PlayerSpawn => String::from("player spawn"),
            Self::Exit => String::from("exit"),
            Self::Bonus => String::from("bonus"),
            Self::Enemy(index) => room
                .enemy_spawns
                .get(index)
                .map_or_else(String::new, |spawn| {
                    format!("enemy #{} {}", index + 1, spawn.type_id)
                }),
            Self::Platform(index) => format!("platform #{}", index + 1),
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
        // Inside the outline: on the inner side of each of its four edges.
        let contains = |item: &Self| {
            item.corners(room).is_some_and(|corners| {
                (0..4).all(|index| {
                    let (from, to) = (corners[index], corners[(index + 1) % 4]);
                    (to - from).perp_dot(point - from) >= 0.0
                })
            })
        };
        let spawns = [Self::PlayerSpawn, Self::Exit, Self::Bonus]
            .into_iter()
            .chain((0..room.enemy_spawns.len()).rev().map(Self::Enemy));
        let blocks = (0..room.platforms.len())
            .map(Self::Platform)
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
    /// Text being typed into a label.
    text_entry: Option<String>,
    /// The typing handled this frame's keys, so shortcuts ignore them.
    keys_consumed: bool,
    /// The mouse position in the room, in H.
    cursor_h: Option<Vec2>,
    undo: Vec<Snapshot>,
    redo: Vec<Snapshot>,
    /// Each edited room as it is on disk, to tell which rooms are unsaved.
    saved: BTreeMap<u16, RoomConfig>,
    /// The hitboxes as they are on disk, once one was edited.
    saved_hitboxes: Option<Hitboxes>,
    /// Which of the selected enemy's attacks the hitbox fields show.
    attack: usize,
    /// Enemy type the next added enemy uses: the last one placed or picked.
    enemy_type: Option<EnemyTypeId>,
    /// What new platforms and decorations are painted with: the last fill picked.
    fill: Option<Fill>,
    /// The hue the color picker's shades are mixed from, in degrees.
    hue: f32,
    /// Items copied with Cmd+C or Cmd+X, ready to paste into any room.
    clipboard: Option<Clipboard>,
    /// The cursor is over an editor panel, so clicks don't reach the room.
    pointer_over_panel: bool,
    /// The layout overlays that are drawn over the room.
    overlays: Overlays,
    /// The playtest settings are open, on this row: Cmd+R asks how the room
    /// should be played before it plays it.
    playtest_row: Option<usize>,
    /// The editor opened this frame: the key that opened it (D in a menu)
    /// must not also act as an editor shortcut.
    just_opened: bool,
    /// What the panel's `!` line says: the edited data failing validation,
    /// refreshed after each edit, or the last save refusing to write.
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
            saved_hitboxes: None,
            attack: 0,
            enemy_type: None,
            fill: None,
            hue: 0.0,
            clipboard: None,
            pointer_over_panel: false,
            overlays: Overlays::default(),
            playtest_row: None,
            just_opened: false,
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
        self.undo.push(Snapshot::Room(self.room_id, before));
        self.redo.clear();
        self.view_dirty = true;
        self.leave_armed = false;
    }

    /// Applies `change` to the hitboxes in `data` as one undoable step.
    fn edit_hitboxes(&mut self, data: &mut GameData, change: impl FnOnce(&mut GameData)) {
        let before = Hitboxes::of(data);
        change(data);
        if Hitboxes::of(data) != before {
            self.saved_hitboxes.get_or_insert_with(|| before.clone());
            self.undo.push(Snapshot::Hitboxes(before));
            self.redo.clear();
            self.view_dirty = true;
            self.leave_armed = false;
        }
    }

    /// The type of the enemy the inspector describes, if it is one.
    fn selected_enemy_type(&self, data: &GameData) -> Option<EnemyTypeId> {
        let Some(EditorItem::Enemy(index)) = self.primary() else {
            return None;
        };
        data.rooms
            .get(&self.room_id)
            .and_then(|room| room.enemy_spawns.get(index))
            .map(|spawn| spawn.type_id.clone())
            .filter(|type_id| data.enemies.types.contains_key(type_id))
    }

    fn unsaved_hitboxes(&self, data: &GameData) -> bool {
        self.saved_hitboxes
            .as_ref()
            .is_some_and(|saved| *saved != Hitboxes::of(data))
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
        self.playtest_row = None;
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
        self.attack = 0;
    }
}

/// One undo or redo step: a room, or the hitboxes, as they were.
enum Snapshot {
    Room(u16, RoomConfig),
    Hitboxes(Hitboxes),
}

impl Snapshot {
    /// Puts this state back into `data`, returning the state it replaced.
    fn restore(self, data: &mut GameData) -> Option<Self> {
        match self {
            Self::Room(room_id, room) => data
                .rooms
                .insert(room_id, room)
                .map(|current| Self::Room(room_id, current)),
            Self::Hitboxes(hitboxes) => {
                let current = Hitboxes::of(data);
                hitboxes.apply(data);
                Some(Self::Hitboxes(current))
            }
        }
    }
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

/// The shortcuts panel, shown while H is held.
#[derive(Component)]
struct EditorHelp;

/// The playtest settings, shown between Cmd+R and the playtest itself.
#[derive(Component)]
struct EditorPlaytest;

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
    if let (Ok(mut transform), Some(room)) = (camera.single_mut(), data.rooms.get(&session.room_id))
    {
        transform.translation.x = room.player_spawn.x_h * H_WORLD_UNITS;
        transform.translation.y =
            (room.player_spawn.y_h + data.gameplay.viewport.height_h / 2.0 - 1.0) * H_WORLD_UNITS;
    }
    let font = TextFont {
        font_size: bevy::text::FontSize::Px(15.0),
        ..default()
    };
    // The inspector stands top left, and the shortcuts under it.
    let column = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(12),
                top: px(12),
                max_width: px(560),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::FlexStart,
                row_gap: px(8),
                ..default()
            },
            Pickable::IGNORE,
            EditorUi,
        ))
        .id();
    let panel_node = Node {
        padding: UiRect::all(px(10)),
        ..default()
    };
    commands.spawn((
        panel_node.clone(),
        Text::new(""),
        font.clone(),
        TextColor(Color::WHITE),
        BackgroundColor(Color::srgba(0.02, 0.025, 0.05, 0.82)),
        Pickable::IGNORE,
        EditorPanel,
        ChildOf(column),
    ));
    commands.spawn((
        panel_node,
        Text::new(
            [
                String::from("Drag empty space: box-select (Shift adds)"),
                String::from("Drag the selection: move (Alt: fine grid)"),
                String::from("Drag a selected block's edge: resize"),
                String::from("Two-finger scroll, right or middle drag: pan"),
                String::from("Pinch or wheel: zoom"),
                String::from("Home: fit the room"),
                String::from("P / D / E: add platform / decoration / enemy"),
                String::from("F: viewport frame, at the spawn and at the cursor"),
                String::from("V: what idle enemies see"),
                String::from("Click a material or a color: paint the selection"),
                String::from("Tab: select the closest item of the same kind"),
                String::from("Del: delete"),
                format!("{COMMAND_KEY}+D: duplicate"),
                String::from("Up / Down: inspector field"),
                String::from("Left / Right: change it (Shift: x4)"),
                String::from("Enter: type a label"),
                String::from("C: clear the field"),
                String::from("PgUp / PgDn: previous / next room"),
                format!("{COMMAND_KEY}+Shift+Z or {COMMAND_KEY}+Y: redo"),
                format!("{COMMAND_KEY}+R: playtest, after its settings"),
                String::from("B (in play): hitbox overlay"),
                String::from("Esc: deselect, then back to the menu"),
            ]
            .join("\n"),
        ),
        font.clone(),
        TextColor(Color::srgb(0.78, 0.8, 0.86)),
        BackgroundColor(Color::srgba(0.02, 0.025, 0.05, 0.82)),
        Visibility::Hidden,
        Pickable::IGNORE,
        EditorHelp,
        ChildOf(column),
    ));
    // The playtest settings stand in the middle of the screen.
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            width: percent(100),
            height: percent(100),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        Pickable::IGNORE,
        EditorUi,
        children![(
            Node {
                padding: UiRect::all(px(16)),
                ..default()
            },
            Text::new(""),
            font,
            TextColor(Color::WHITE),
            BackgroundColor(Color::srgba(0.02, 0.025, 0.05, 0.94)),
            Visibility::Hidden,
            Pickable::IGNORE,
            EditorPlaytest,
        )],
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
        // A fixed layout, independent of the authored rooms.
        let room: RoomConfig = yaml_serde::from_str(
            "
room_id: 3
chapter_id: 1
size: { width_h: 48, height_h: 27 }
background_profiles: [clouds_bright_blue]
player_spawn: { x_h: 2.5, y_h: 1 }
platforms:
  - { label: floor, material: city_pavement, rect_h: { x: 0, y: 0, width: 48, height: 1 } }
  - { label: walkway, color: \"#4C5A74\", rect_h: { x: 13, y: 6, width: 14, height: 1 } }
  - { label: step, material: city_pavement, rect_h: { x: 28, y: 1, width: 2, height: 1.5 } }
enemy_spawns: []
bonus: { x_h: 43, y_h: 1 }
exit: { x_h: 45, y_h: 1 }
next_room_id: 4
",
        )
        .expect("the test layout should parse");
        let room = &room;

        let feet = Vec2::new(room.player_spawn.x_h, room.player_spawn.y_h + 0.5);
        assert_eq!(EditorItem::at(room, feet), Some(EditorItem::PlayerSpawn));
        // The step (x 28..30) sits on the floor (x 0..48): both cover
        // this point, and the step is the smaller block.
        let step = room
            .platforms
            .iter()
            .position(|block| block.label.as_deref() == Some("step"))
            .map(EditorItem::Platform);
        assert_eq!(EditorItem::at(room, Vec2::new(29.0, 1.0)), step);
        assert_eq!(EditorItem::at(room, Vec2::new(-5.0, -5.0)), None);

        // Turned a quarter, the walkway (14 x 1 around 20, 6.5) stands on
        // end: a click finds it where it is drawn, not where it lay.
        let mut room = room.clone();
        room.platforms[1].rotation_deg = 90.0;
        let walkway = Some(EditorItem::Platform(1));
        assert_eq!(EditorItem::at(&room, Vec2::new(20.0, 12.0)), walkway);
        assert_eq!(EditorItem::at(&room, Vec2::new(14.0, 6.5)), None);
    }

    #[test]
    fn positions_snap_to_the_grid_or_the_fine_grid() {
        assert_eq!(snap(2.37, false), 2.25);
        assert_eq!(snap(2.37, true), 2.35);
        assert!(snap(-0.1, false).is_sign_positive());
    }

    #[test]
    fn armor_steps_up_from_none_stops_at_none_and_clears_back_to_it() {
        let asset_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/assets");
        let data = GameData::load(&asset_root).expect("Chapter 1 YAML should be valid");
        let mut room = data.rooms[&1].clone();
        room.enemy_spawns[0].armor_layers = 0;
        let original = room.clone();
        let selection = [EditorItem::Enemy(0)];

        Field::Armor.adjust(&mut room, &selection, &data, 1.0);
        Field::Armor.adjust(&mut room, &selection, &data, 1.0);
        assert_eq!(room.enemy_spawns[0].armor_layers, 2);

        for _ in 0..3 {
            Field::Armor.adjust(&mut room, &selection, &data, -1.0);
        }
        assert_eq!(room.enemy_spawns[0].armor_layers, 0, "Left stops at none");
        assert_eq!(Field::Armor.value(&room, &selection, &data, 0), "none");

        Field::Armor.adjust(&mut room, &selection, &data, 1.0);
        Field::Armor.clear(&mut room, selection.last().copied());
        assert_eq!(room, original);
    }

    #[test]
    fn a_second_background_is_laid_over_the_first_and_taken_off_again() {
        let asset_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/assets");
        let data = GameData::load(&asset_root).expect("Chapter 1 YAML should be valid");
        let mut room = data.rooms[&1].clone();
        room.background_profiles.truncate(1);
        let original = room.clone();

        Field::BackgroundOver.adjust(&mut room, &[], &data, 1.0);
        assert_eq!(room.background_profiles.len(), 2);
        assert_eq!(room.background_profiles[0], original.background_profiles[0]);

        Field::BackgroundOver.adjust(&mut room, &[], &data, -1.0);
        assert_eq!(room, original);
    }

    #[test]
    fn a_copied_selection_pastes_offset_paints_together_and_deletes_together() {
        let asset_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/assets");
        let data = GameData::load(&asset_root).expect("Chapter 1 YAML should be valid");
        let mut room = data.rooms[&3].clone();
        let selection = [
            EditorItem::Platform(1),
            EditorItem::Platform(0),
            EditorItem::Enemy(0),
            EditorItem::PlayerSpawn,
        ];
        let copied = Clipboard::copy(&room, &selection).expect("blocks and enemies copy");

        let offset = Vec2::new(5.0, 2.0);
        let pasted = copied.paste(&mut room, copied.origin + offset);
        assert_eq!(pasted.len(), 3, "the player spawn is not copied");
        for (original, copy) in selection.iter().zip(&pasted) {
            let moved = copy.origin(&room).zip(original.origin(&room));
            assert_eq!(moved.map(|(copy, original)| copy - original), Some(offset));
        }

        let before = pasted[1].fill(&room).cloned();
        Field::Fill.adjust(&mut room, &pasted, &data, 1.0);
        let painted = pasted[0].fill(&room).cloned();
        assert_ne!(painted, before, "Fill cycles to another material");
        assert_eq!(
            pasted[1].fill(&room).cloned(),
            painted,
            "one step paints the whole selection"
        );

        EditorItem::remove_all(&mut room, &pasted);
        assert_eq!(room, data.rooms[&3], "only the pasted items were painted");
    }
}
