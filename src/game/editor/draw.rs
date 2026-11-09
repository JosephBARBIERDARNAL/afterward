//! What the editor shows: the room drawn as in play, outlines of every
//! editable item over it, and the inspector and help panels.

use super::*;
use bevy::input::keyboard::Key;

const ROOM_BOUNDS_COLOR: Color = Color::srgb(0.95, 0.3, 0.3);
const GRID_COLOR: Color = Color::srgba(1.0, 1.0, 1.0, 0.06);
const DECORATION_COLOR: Color = Color::srgb(0.75, 0.5, 0.95);
const MARQUEE_COLOR: Color = Color::srgb(0.4, 0.8, 1.0);
const PLATFORM_COLOR: Color = Color::srgb(0.95, 0.65, 0.3);
/// The flood's own `foam` (`#7FE6E8`): a pool is outlined in the pale cyan
/// light the water carries.
const WATER_COLOR: Color = Color::srgb(0.498, 0.902, 0.91);
const PLAYER_COLOR: Color = Color::srgb(0.35, 0.95, 0.5);
const EXIT_COLOR: Color = Color::srgb(0.35, 0.8, 1.0);
const ENEMY_COLOR: Color = Color::srgb(1.0, 0.35, 0.45);
const BONUS_COLOR: Color = Color::srgb(1.0, 0.78, 0.3);
const HOVER_COLOR: Color = Color::WHITE;
const SELECTED_COLOR: Color = Color::srgb(1.0, 0.9, 0.2);
// The selected body's hitboxes, colored as in the B overlay.
const PLAYER_HURTBOX_COLOR: Color = Color::srgb(0.3, 1.0, 0.45);
const ENEMY_HURTBOX_COLOR: Color = Color::srgb(1.0, 0.9, 0.2);
const STRIKE_COLOR: Color = Color::srgb(1.0, 0.2, 0.3);

/// Redraws the edited room after a change, as the game draws it, and
/// re-validates the data so the panel can flag a broken room right away.
pub(super) fn refresh_room_view(
    mut commands: Commands,
    mut editor: ResMut<RoomEditor>,
    data: Res<GameData>,
    asset_server: Res<AssetServer>,
    sheets: Res<SpriteSheets>,
    mut shaders: ShaderAssets,
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
    world::draw_room(&mut commands, room, &data, &asset_server, &mut shaders);
    world::spawn_player(&mut commands, room, &data, &sheets);
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
    let room_size_h = Vec2::new(room.size.width_h, room.size.height_h);
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

    // The selected body's hitboxes, drawn by the code combat uses. An enemy
    // strikes the way the room points it, as it does on its first attack.
    let mut world_outline = |rect: Rect, color: Color| {
        let min = rect.min / H_WORLD_UNITS;
        let size = rect.size() / H_WORLD_UNITS;
        outline(
            RectH {
                x: min.x,
                y: min.y,
                width: size.x,
                height: size.y,
            },
            color,
        );
    };
    match editor.selected.as_slice() {
        [EditorItem::PlayerSpawn] => {
            let feet = Vec2::new(room.player_spawn.x_h, room.player_spawn.y_h) * H_WORLD_UNITS;
            world_outline(
                crate::game::combat::player_hurtbox(
                    (feet + Vec2::Y * H_WORLD_UNITS / 2.0).extend(0.0),
                    &data,
                ),
                PLAYER_HURTBOX_COLOR,
            );
        }
        [EditorItem::Enemy(index)] => {
            let enemy = room
                .enemy_spawns
                .get(*index)
                .and_then(|spawn| Some((spawn, data.enemies.types.get(&spawn.type_id)?)));
            if let Some((spawn, enemy)) = enemy {
                let feet = Vec2::new(spawn.position_h.x_h, spawn.position_h.y_h) * H_WORLD_UNITS;
                let hurtbox = Vec2::new(enemy.hurtbox_h.width, enemy.hurtbox_h.height);
                world_outline(
                    Rect::from_corners(
                        feet - Vec2::new(hurtbox.x / 2.0, 0.0) * H_WORLD_UNITS,
                        feet + Vec2::new(hurtbox.x / 2.0, hurtbox.y) * H_WORLD_UNITS,
                    ),
                    ENEMY_HURTBOX_COLOR,
                );
                let facing = f32::from(spawn.facing);
                let strike = enemy
                    .attack
                    .iter()
                    .chain(&enemy.attacks)
                    .nth(editor.attack)
                    .and_then(|attack| EnemyAttack::from(attack).hitbox_at(feet, facing));
                if let Some(strike) = strike {
                    world_outline(strike, STRIKE_COLOR);
                }
            }
        }
        _ => {}
    }

    for item in EditorItem::all(room) {
        let selected = editor.selected.contains(&item);
        let Some(rect) = item.bounds(room) else {
            continue;
        };
        let color = match item {
            _ if selected => SELECTED_COLOR,
            _ if editor.hovered == Some(item) => HOVER_COLOR,
            EditorItem::Platform(_) => PLATFORM_COLOR,
            EditorItem::Decoration(_) => DECORATION_COLOR,
            EditorItem::Water(_) => WATER_COLOR,
            EditorItem::Enemy(_) => ENEMY_COLOR,
            EditorItem::PlayerSpawn => PLAYER_COLOR,
            EditorItem::Exit => EXIT_COLOR,
            EditorItem::Bonus => BONUS_COLOR,
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
        format!("Room {}  (chapter {})", room.room_id, room.chapter_id),
        {
            let mut parts = Vec::new();
            if !unsaved.is_empty() {
                let rooms = unsaved
                    .iter()
                    .map(|room_id| format!("{room_id:02}"))
                    .collect::<Vec<_>>();
                parts.push(format!("room {}", rooms.join(", ")));
            }
            if editor.unsaved_hitboxes(&data) {
                parts.push(String::from("hitboxes"));
            }
            if parts.is_empty() {
                String::from("All changes saved.")
            } else {
                format!("UNSAVED: {}", parts.join(" | "))
            }
        },
    ];
    if let Some(problem) = &editor.problem {
        lines.push(format!("! {problem}"));
    }
    if editor.leave_armed {
        lines.push(format!(
            "Esc again leaves: {COMMAND_KEY}+S saves first, and edits stay in memory until the game quits."
        ));
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
            _ => field.value(room, &editor.selected, &data, editor.attack),
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
    if editor
        .fields()
        .get(editor.field)
        .is_some_and(|field| field.is_hitbox())
    {
        lines.push(match editor.selected_enemy_type(&data) {
            Some(type_id) => format!("Hitboxes are shared by every {type_id} (enemy_types.yaml)"),
            None => String::from("The player's hurtbox (gameplay.yaml)"),
        });
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
    lines.push(format!("Overlays (F/V): {}", editor.overlays.describe()));
    lines.push(String::from("\nHold H: shortcuts"));
    for mut text in &mut panel {
        text.0 = lines.join("\n");
    }
}

/// Shows the shortcuts panel while H is held (by the character it types, so
/// on any layout), unless a label is being typed.
pub(super) fn show_help_while_h_is_held(
    keys: Res<ButtonInput<Key>>,
    editor: Res<RoomEditor>,
    mut help: Query<&mut Visibility, With<EditorHelp>>,
) {
    let held = keys.any_pressed([Key::Character("h".into()), Key::Character("H".into())])
        && editor.text_entry.is_none();
    for mut visibility in &mut help {
        visibility.set_if_neq(if held {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });
    }
}
