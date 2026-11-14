//! What the editor shows over the room: the room drawn as in play, and
//! outlines of every editable item over it (the panels are ui.rs).

use super::*;

const ROOM_BOUNDS_COLOR: Color = Color::srgb(0.95, 0.3, 0.3);
const GRID_COLOR: Color = Color::srgba(1.0, 1.0, 1.0, 0.06);
const DECORATION_COLOR: Color = Color::srgb(0.75, 0.5, 0.95);
const MARQUEE_COLOR: Color = Color::srgb(0.4, 0.8, 1.0);
const PLATFORM_COLOR: Color = Color::srgb(0.95, 0.65, 0.3);
const PLAYER_COLOR: Color = Color::srgb(0.35, 0.95, 0.5);
/// How tall the selected player spawn's body is drawn (`body_height_h`).
const PLAYER_BODY_COLOR: Color = Color::srgb(0.95, 0.95, 1.0);
const EXIT_COLOR: Color = Color::srgb(0.35, 0.8, 1.0);
const ENEMY_COLOR: Color = Color::srgb(1.0, 0.35, 0.45);
const BONUS_COLOR: Color = Color::srgb(1.0, 0.78, 0.3);
const VENT_COLOR: Color = Color::srgb(1.0, 0.45, 0.1);
const HOVER_COLOR: Color = Color::WHITE;
pub(super) const SELECTED_COLOR: Color = Color::srgb(1.0, 0.9, 0.2);
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
    world::spawn_enemies(&mut commands, room, &data, &sheets, None);
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
            // How tall the body is drawn, as wide as the collision box.
            let body = Vec2::new(
                PLAYER_COLLISION_WIDTH_H,
                data.player_animations.body_height_h,
            ) * H_WORLD_UNITS;
            world_outline(
                Rect::from_corners(
                    feet - Vec2::new(body.x / 2.0, 0.0),
                    feet + Vec2::new(body.x / 2.0, body.y),
                ),
                PLAYER_BODY_COLOR,
            );
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
        let color = match item {
            _ if selected => SELECTED_COLOR,
            _ if editor.hovered == Some(item) => HOVER_COLOR,
            item => item.outline_color(),
        };
        if let Some(corners) = item.corners(room) {
            // Corner to corner and back to the first, so a turned platform
            // is outlined where it stands.
            gizmos.linestrip_2d(
                corners
                    .into_iter()
                    .chain([corners[0]])
                    .map(|corner| corner * H_WORLD_UNITS),
                color,
            );
        }
        if selected && editor.selected.len() == 1 && !item.is_turned(room) {
            // Corner handles mark the edges a drag resizes.
            let handle_h = 0.16;
            for corner in item.corners(room).into_iter().flatten() {
                gizmos.rect_2d(
                    corner * H_WORLD_UNITS,
                    Vec2::splat(handle_h * H_WORLD_UNITS),
                    SELECTED_COLOR,
                );
            }
        }
    }
}

impl EditorItem {
    /// The color the item is outlined in, which the inspector's chip repeats.
    pub(super) fn outline_color(self) -> Color {
        match self {
            Self::Platform(_) => PLATFORM_COLOR,
            Self::Decoration(_) => DECORATION_COLOR,
            Self::Enemy(_) => ENEMY_COLOR,
            Self::PlayerSpawn => PLAYER_COLOR,
            Self::Exit => EXIT_COLOR,
            Self::Bonus => BONUS_COLOR,
            Self::Hazard(_) => VENT_COLOR,
        }
    }
}
