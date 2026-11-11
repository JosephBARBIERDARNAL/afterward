//! The layout overlays, each a toggle: the viewport frame (F) and what an
//! idle enemy sees (V). They answer, in the editor, questions that otherwise
//! take a playtest, so each one runs the rules the game itself plays by:
//! `RoomCamera` for the frame, `Platforms` for the sight lines, and the tuning
//! in `config/gameplay.yaml` and `enemies/enemy_types.yaml` for the rest.

use super::*;

/// The frame the camera holds when the room opens, behind the one at the cursor.
const SPAWN_FRAME_COLOR: Color = Color::srgba(0.45, 0.85, 1.0, 0.35);
const FRAME_COLOR: Color = Color::srgb(0.45, 0.85, 1.0);
/// The sight lines filling the detection band, as far into it as cover allows.
const SIGHT_COLOR: Color = Color::srgba(0.95, 0.2, 0.16, 0.22);
/// Drawn when an enemy already sees the player standing on the spawn. It is
/// the one line that has to stand out from the band around it, so it is paler
/// than the deep red filling that.
const SPOTTED_COLOR: Color = Color::srgb(1.0, 0.6, 0.55);

/// Sight lines cast around the detection band, and the halvings that place
/// each one's end against the platform that cuts it short. The lines fan out
/// from one point, so they thin with distance: enough of them that the far
/// end of the band still fills in rather than reading as stripes.
const SIGHT_LINES: usize = 220;
const SIGHT_CLIP_STEPS: usize = 12;

/// Which layout overlays are drawn.
#[derive(Default)]
pub(super) struct Overlays {
    pub(super) viewport: bool,
    pub(super) sight: bool,
}

pub(super) fn draw_overlays(mut gizmos: Gizmos, editor: Res<RoomEditor>, data: Res<GameData>) {
    let Some(room) = data.rooms.get(&editor.room_id) else {
        return;
    };
    if editor.overlays.viewport {
        draw_viewport(&mut gizmos, room, &data, editor.cursor_h);
    }
    if editor.overlays.sight {
        let surfaces = room
            .platforms
            .iter()
            .map(|block| PlatformSurface::of(block, &data))
            .collect::<Vec<_>>();
        draw_sight(
            &mut gizmos,
            room,
            &data,
            &Platforms(surfaces.iter().collect()),
        );
    }
}

/// The 16:9 window the player actually sees: where the camera sits when the
/// room opens, and where it would sit with the player under the cursor. Both
/// are clamped to the room the way the camera is, so neither shows past it.
fn draw_viewport(gizmos: &mut Gizmos, room: &RoomConfig, data: &GameData, cursor_h: Option<Vec2>) {
    let camera = RoomCamera {
        size_h: Vec2::new(room.size.width_h, room.size.height_h),
    };
    let view_h = Vec2::new(
        data.gameplay.viewport_width_h(),
        data.gameplay.viewport.height_h,
    );
    let mut frame = |center_h: Vec2, color: Color| {
        let center = camera.clamp_center_h(center_h, &data.gameplay);
        gizmos.rect_2d(center * H_WORLD_UNITS, view_h * H_WORLD_UNITS, color);
    };
    // The camera centers on the player's middle, half a body above their feet.
    frame(
        Vec2::new(room.player_spawn.x_h, room.player_spawn.y_h + 0.5),
        SPAWN_FRAME_COLOR,
    );
    if let Some(cursor_h) = cursor_h {
        frame(cursor_h, FRAME_COLOR);
    }
}

/// What every idle enemy in the room can see: the band it watches, filled with
/// sight lines that stop against the platforms that hide the far side.
fn draw_sight(gizmos: &mut Gizmos, room: &RoomConfig, data: &GameData, platforms: &Platforms) {
    let detection = &data.enemies.detection;
    let range = detection.range_h * H_WORLD_UNITS;
    let height = detection.height_h * H_WORLD_UNITS;
    let spawn = Vec2::new(room.player_spawn.x_h, room.player_spawn.y_h + 0.5) * H_WORLD_UNITS;
    let player = crate::game::combat::player_hurtbox(spawn.extend(0.0), data).center();

    for spawn in &room.enemy_spawns {
        let Some(enemy) = data.enemies.types.get(&spawn.type_id) else {
            continue;
        };
        // Sight runs between hurtbox centers, so a tall enemy looks out from
        // higher up: the same eyes `EnemyBody::sees_player` uses.
        let feet = Vec2::new(spawn.position_h.x_h, spawn.position_h.y_h) * H_WORLD_UNITS;
        let eyes = feet + Vec2::Y * enemy.hurtbox_h.height / 2.0 * H_WORLD_UNITS;
        let facing = f32::from(spawn.facing);
        let far_x = eyes.x + facing * range;
        for index in 0..=SIGHT_LINES {
            let target = band_point(eyes, far_x, height, index as f32 / SIGHT_LINES as f32);
            let seen = eyes.lerp(target, visible_fraction(eyes, target, platforms));
            gizmos.line_2d(eyes, seen, SIGHT_COLOR);
        }
        // The one line that decides whether this enemy wakes on its own: the
        // player standing where they walk in.
        let offset = player - eyes;
        if offset.x.abs() <= range
            && offset.y.abs() <= height
            && offset.x * facing >= 0.0
            && !platforms.blocks_sight(eyes, player)
        {
            gizmos.line_2d(eyes, player, SPOTTED_COLOR);
        }
    }
}

/// The point `fraction` of the way around the far side of the detection band,
/// from the top near corner over the far edge to the bottom near corner. The
/// band is a rectangle, so walking its perimeter covers every direction the
/// enemy watches, which a fan spread over the far edge alone would not.
fn band_point(eyes: Vec2, far_x: f32, height: f32, fraction: f32) -> Vec2 {
    let reach = (far_x - eyes.x).abs();
    let sides = reach / (reach + height).max(f32::EPSILON) / 2.0;
    if fraction < sides {
        let along = fraction / sides.max(f32::EPSILON);
        Vec2::new(eyes.x + (far_x - eyes.x) * along, eyes.y + height)
    } else if fraction > 1.0 - sides {
        let along = (1.0 - fraction) / sides.max(f32::EPSILON);
        Vec2::new(eyes.x + (far_x - eyes.x) * along, eyes.y - height)
    } else {
        let down = (fraction - sides) / (1.0 - 2.0 * sides).max(f32::EPSILON);
        Vec2::new(far_x, eyes.y + height - 2.0 * height * down)
    }
}

/// How far along `eyes` to `target` the enemy sees before a platform cuts in,
/// as a fraction of the way. Cover is solid, so a point hidden from the eyes
/// hides everything behind it: the first fraction that is blocked can be
/// halved in on.
fn visible_fraction(eyes: Vec2, target: Vec2, platforms: &Platforms) -> f32 {
    if !platforms.blocks_sight(eyes, target) {
        return 1.0;
    }
    let (mut visible, mut blocked) = (0.0, 1.0);
    for _ in 0..SIGHT_CLIP_STEPS {
        let middle = (visible + blocked) / 2.0;
        if platforms.blocks_sight(eyes, eyes.lerp(target, middle)) {
            blocked = middle;
        } else {
            visible = middle;
        }
    }
    visible
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn a_platform_cuts_a_sight_line_short_and_open_air_leaves_it_whole() {
        let asset_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/assets");
        let data = GameData::load(&asset_root).expect("Chapter 1 YAML should be valid");
        // A fixed layout, independent of the authored rooms: a plain floor.
        let mut room: RoomConfig = yaml_serde::from_str(
            "
room_id: 3
chapter_id: 1
size: { width_h: 48, height_h: 27 }
background_profile: clouds_bright_blue
player_spawn: { x_h: 2.5, y_h: 1 }
platforms:
  - { label: floor, material: city_pavement, rect_h: { x: 0, y: 0, width: 48, height: 1 } }
enemy_spawns: []
bonus: { x_h: 43, y_h: 1 }
exit: { x_h: 45, y_h: 1 }
next_room_id: 4
",
        )
        .expect("the test layout should parse");
        let surfaces = |room: &RoomConfig| {
            room.platforms
                .iter()
                .map(|block| PlatformSurface::of(block, &data))
                .collect::<Vec<_>>()
        };

        // Eye level above the floor, looking across the room.
        let eyes = Vec2::new(20.0, 2.0) * H_WORLD_UNITS;
        let target = Vec2::new(30.0, 2.0) * H_WORLD_UNITS;
        let open = surfaces(&room);
        assert_eq!(
            visible_fraction(eyes, target, &Platforms(open.iter().collect())),
            1.0
        );

        // A pillar halfway stops the line there.
        room.platforms.push(
            yaml_serde::from_str(
                "{ material: city_pavement, rect_h: { x: 25, y: 1, width: 1, height: 4 } }",
            )
            .expect("the pillar should parse"),
        );
        let covered = surfaces(&room);
        let seen = visible_fraction(eyes, target, &Platforms(covered.iter().collect()));
        assert!(
            (seen - 0.5).abs() < 0.02,
            "the line should stop at the pillar, half way: {seen}"
        );
    }

    #[test]
    fn the_sight_band_is_walked_from_corner_to_corner() {
        let eyes = Vec2::ZERO;
        let (far_x, height) = (-10.0, 3.0);
        assert_eq!(band_point(eyes, far_x, height, 0.0), Vec2::new(0.0, 3.0));
        assert_eq!(band_point(eyes, far_x, height, 1.0), Vec2::new(0.0, -3.0));
        let middle = band_point(eyes, far_x, height, 0.5);
        assert_eq!(
            middle,
            Vec2::new(-10.0, 0.0),
            "the middle looks straight on"
        );
    }
}
