//! What the editor shows: the room drawn as in play, outlines of every
//! editable item over it, and the inspector and help panels.

use super::*;

const ROOM_BOUNDS_COLOR: Color = Color::srgb(0.95, 0.3, 0.3);
const GRID_COLOR: Color = Color::srgba(1.0, 1.0, 1.0, 0.06);
const TERRAIN_COLOR: Color = Color::srgb(0.7, 0.72, 0.78);
const DECORATION_COLOR: Color = Color::srgb(0.75, 0.5, 0.95);
const MARQUEE_COLOR: Color = Color::srgb(0.4, 0.8, 1.0);
const PLATFORM_COLOR: Color = Color::srgb(0.95, 0.65, 0.3);
const PLAYER_COLOR: Color = Color::srgb(0.35, 0.95, 0.5);
const EXIT_COLOR: Color = Color::srgb(0.35, 0.8, 1.0);
const ENEMY_COLOR: Color = Color::srgb(1.0, 0.35, 0.45);
const HOVER_COLOR: Color = Color::WHITE;
const SELECTED_COLOR: Color = Color::srgb(1.0, 0.9, 0.2);

/// Redraws the edited room after a change, as the game draws it, and
/// re-validates the data so the panel can flag a broken room right away.
pub(super) fn refresh_room_view(
    mut commands: Commands,
    mut editor: ResMut<RoomEditor>,
    data: Res<GameData>,
    asset_server: Res<AssetServer>,
    sheets: Res<SpriteSheets>,
    room_entities: Query<Entity, With<RoomEntity>>,
) {
    if !editor.view_dirty {
        return;
    }
    editor.view_dirty = false;
    for entity in &room_entities {
        commands.entity(entity).despawn();
    }
    let Some(room) = data.rooms.get(&editor.room_id) else {
        return;
    };
    world::draw_room(&mut commands, room, &data, &asset_server);
    world::spawn_player(&mut commands, room, &data, &sheets, 0);
    world::spawn_enemies(&mut commands, room, &data, &sheets);
    world::spawn_exit_door(&mut commands, room, &data, &sheets);
    if editor
        .selected
        .iter()
        .any(|item| item.bounds(room).is_none())
    {
        editor.select(None);
    }
    editor.problem = data.validate().err().map(|problem| problem.to_string());
}

pub(super) fn draw_overlay(mut gizmos: Gizmos, editor: Res<RoomEditor>, data: Res<GameData>) {
    let Some(room) = data.rooms.get(&editor.room_id) else {
        return;
    };
    let room_size_h = Vec2::new(room.width_h(&data.gameplay), room.size.height_h);
    let room_size = room_size_h * H_WORLD_UNITS;
    for x in 1..room_size_h.x.ceil() as u32 {
        let x = x as f32 * H_WORLD_UNITS;
        gizmos.line_2d(Vec2::new(x, 0.0), Vec2::new(x, room_size.y), GRID_COLOR);
    }
    for y in 1..room_size_h.y.ceil() as u32 {
        let y = y as f32 * H_WORLD_UNITS;
        gizmos.line_2d(Vec2::new(0.0, y), Vec2::new(room_size.x, y), GRID_COLOR);
    }
    let mut outline = |rect: RectH, color: Color| {
        let min = Vec2::new(rect.x, rect.y) * H_WORLD_UNITS;
        let size = Vec2::new(rect.width, rect.height) * H_WORLD_UNITS;
        gizmos.rect_2d(min + size / 2.0, size, color);
    };
    outline(
        RectH {
            x: 0.0,
            y: 0.0,
            width: room_size_h.x,
            height: room_size_h.y,
        },
        ROOM_BOUNDS_COLOR,
    );

    if let (Some(Drag::Marquee { start }), Some(cursor_h)) = (&editor.drag, editor.cursor_h) {
        let area = Rect::from_corners(*start, cursor_h);
        outline(
            RectH {
                x: area.min.x,
                y: area.min.y,
                width: area.width(),
                height: area.height(),
            },
            MARQUEE_COLOR,
        );
    }

    for item in EditorItem::all(room) {
        let selected = editor.selected.contains(&item);
        let Some(rect) = item.bounds(room) else {
            continue;
        };
        let color = match item {
            _ if selected => SELECTED_COLOR,
            _ if editor.hovered == Some(item) => HOVER_COLOR,
            EditorItem::Terrain(_) => TERRAIN_COLOR,
            EditorItem::Platform(_) => PLATFORM_COLOR,
            EditorItem::Decoration(_) => DECORATION_COLOR,
            EditorItem::Enemy(_) => ENEMY_COLOR,
            EditorItem::PlayerSpawn => PLAYER_COLOR,
            EditorItem::Exit => EXIT_COLOR,
        };
        outline(rect, color);
        if selected && editor.selected.len() == 1 {
            // Corner handles mark the edges a drag resizes.
            let handle_h = 0.16;
            for (x, y) in [
                (rect.x, rect.y),
                (rect.right(), rect.y),
                (rect.x, rect.top()),
                (rect.right(), rect.top()),
            ] {
                outline(
                    RectH {
                        x: x - handle_h / 2.0,
                        y: y - handle_h / 2.0,
                        width: handle_h,
                        height: handle_h,
                    },
                    SELECTED_COLOR,
                );
            }
        }
    }
}

pub(super) fn update_panels(
    editor: Res<RoomEditor>,
    data: Res<GameData>,
    mut panel: Query<&mut Text, With<EditorPanel>>,
) {
    let Some(room) = data.rooms.get(&editor.room_id) else {
        return;
    };
    let unsaved = editor.unsaved_rooms(&data);
    let mut lines = vec![
        format!(
            "ROOM EDITOR  -  room {:02}  (chapter {})",
            room.room_id, room.chapter_id
        ),
        if unsaved.is_empty() {
            String::from("All changes saved.")
        } else {
            let rooms = unsaved
                .iter()
                .map(|room_id| format!("{room_id:02}"))
                .collect::<Vec<_>>();
            format!("UNSAVED: room {}", rooms.join(", "))
        },
    ];
    if let Some(problem) = &editor.problem {
        lines.push(format!("! {problem}"));
    }
    lines.push(String::new());
    lines.push(match editor.selected.as_slice() {
        [] => String::from("Room (click an item to select it)"),
        [item] => format!("Selected: {}", item.describe(room)),
        items => format!("Selected: {} items", items.len()),
    });
    for (index, field) in editor.fields().iter().enumerate() {
        let is_current = index == editor.field;
        let value = match &editor.text_entry {
            Some(text) if is_current => format!("{text}_"),
            _ => field.value(room, &editor.selected, &data),
        };
        lines.push(format!(
            "{} {:<18} {value}",
            if is_current { ">" } else { " " },
            field.name()
        ));
    }
    if editor.text_entry.is_some() {
        lines.push(String::from("Typing: Enter keeps, Esc cancels"));
    }
    if let Some(hovered) = editor
        .hovered
        .filter(|item| !editor.selected.contains(item))
    {
        lines.push(format!("Hover: {}", hovered.describe(room)));
    }
    if let Some(cursor_h) = editor.cursor_h {
        lines.push(format!(
            "Cursor: x {:.2}  y {:.2} H",
            cursor_h.x, cursor_h.y
        ));
    }
    if !editor.status.is_empty() {
        lines.push(String::new());
        lines.push(editor.status.clone());
    }
    for mut text in &mut panel {
        text.0 = lines.join("\n");
    }
}
