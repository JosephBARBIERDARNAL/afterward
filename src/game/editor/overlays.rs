//! The layout overlays, each a toggle: the viewport frame (F), where a jump
//! and a dash reach (J), and what an idle enemy sees (V). They answer, in the
//! editor, questions that otherwise take a playtest, so each one runs the
//! rules the game itself plays by: `Platforms` for the collisions and the
//! sight lines, `RoomCamera` for the frame, and the tuning in
//! `config/gameplay.yaml` and `enemies/enemy_types.yaml` for the rest.

use super::*;

/// The frame the camera holds when the room opens, behind the one at the cursor.
const SPAWN_FRAME_COLOR: Color = Color::srgba(0.45, 0.85, 1.0, 0.35);
const FRAME_COLOR: Color = Color::srgb(0.45, 0.85, 1.0);
/// The player's jump, and the dash that carries it further.
const JUMP_COLOR: Color = Color::srgb(0.4, 0.95, 0.75);
const DASH_COLOR: Color = Color::srgba(0.6, 0.7, 1.0, 0.75);
/// An enemy's jump, which is shorter and has no dash.
const ENEMY_JUMP_COLOR: Color = Color::srgb(1.0, 0.5, 0.4);
/// Where an arc touches down.
const LANDING_COLOR: Color = Color::srgb(1.0, 0.95, 0.5);
/// The detection band, and the sight lines filling what is not behind cover.
const SIGHT_EDGE_COLOR: Color = Color::srgba(1.0, 0.85, 0.35, 0.7);
const SIGHT_COLOR: Color = Color::srgba(1.0, 0.8, 0.3, 0.16);
/// Drawn when an enemy already sees the player standing on the spawn.
const SPOTTED_COLOR: Color = Color::srgb(1.0, 0.35, 0.3);

/// How long a simulated jump is followed before it is given up on, and the
/// step it is simulated at. The step is finer than a frame, so an arc drawn
/// over a tight gap does not step past the ledge that stops it in play.
const FLIGHT_STEP_S: f32 = 1.0 / 120.0;
const MAX_FLIGHT_S: f32 = 4.0;
/// Sight lines cast around the detection band, and the halvings that place
/// each one's end against the platform that cuts it short.
const SIGHT_LINES: usize = 48;
const SIGHT_CLIP_STEPS: usize = 12;
/// Half-width of the cross marking a landing, in world units.
const LANDING_MARK: f32 = 0.12 * H_WORLD_UNITS;

/// Which layout overlays are drawn.
#[derive(Default)]
pub(super) struct Overlays {
    pub(super) viewport: bool,
    pub(super) reach: bool,
    pub(super) sight: bool,
}

impl Overlays {
    /// The ones that are on, for the inspector panel.
    pub(super) fn describe(&self) -> String {
        let shown = [
            (self.viewport, "frame"),
            (self.reach, "reach"),
            (self.sight, "sight"),
        ]
        .into_iter()
        .filter_map(|(on, name)| on.then_some(name))
        .collect::<Vec<_>>();
        if shown.is_empty() {
            String::from("off")
        } else {
            shown.join(", ")
        }
    }
}

pub(super) fn draw_overlays(mut gizmos: Gizmos, editor: Res<RoomEditor>, data: Res<GameData>) {
    let Some(room) = data.rooms.get(&editor.room_id) else {
        return;
    };
    if editor.overlays.viewport {
        draw_viewport(&mut gizmos, room, &data, editor.cursor_h);
    }
    if !editor.overlays.reach && !editor.overlays.sight {
        return;
    }
    let surfaces = room
        .platforms
        .iter()
        .map(|block| PlatformSurface::of(block, &data))
        .collect::<Vec<_>>();
    let platforms = Platforms(surfaces.iter().collect());
    if editor.overlays.reach {
        draw_reach(&mut gizmos, &editor, room, &data, &platforms);
    }
    if editor.overlays.sight {
        draw_sight(&mut gizmos, room, &data, &platforms);
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

/// What a body can reach from where it stands: a running jump out of each side
/// of every selected platform (or from the cursor when nothing is selected),
/// and for the player the same jump carried further by a dash.
fn draw_reach(
    gizmos: &mut Gizmos,
    editor: &RoomEditor,
    room: &RoomConfig,
    data: &GameData,
    platforms: &Platforms,
) {
    for (feet, direction, reach) in launches(editor, room, data) {
        for dash in [false, true] {
            if dash && reach.dash_distance.is_none() {
                continue;
            }
            let (path, landing) = arc(feet, direction, reach, dash, room, platforms);
            gizmos.linestrip_2d(path, if dash { DASH_COLOR } else { reach.color });
            if let Some(landing) = landing {
                gizmos.cross_2d(landing, LANDING_MARK, LANDING_COLOR);
            }
        }
    }
}

/// Where each drawn arc starts, which way it goes, and whose legs it uses.
fn launches(editor: &RoomEditor, room: &RoomConfig, data: &GameData) -> Vec<(Vec2, f32, Reach)> {
    let player = Reach::player(data);
    let mut launches = Vec::new();
    for item in &editor.selected {
        match item {
            // A platform is left over either ledge, the last place a body is
            // still held up and so the furthest a jump off it can start.
            EditorItem::Platform(index) => {
                if let Some(block) = room.platforms.get(*index) {
                    let rect = block.rect_h;
                    let top = rect.top() * H_WORLD_UNITS;
                    launches.push((Vec2::new(rect.x * H_WORLD_UNITS, top), -1.0, player));
                    launches.push((Vec2::new(rect.right() * H_WORLD_UNITS, top), 1.0, player));
                }
            }
            EditorItem::Enemy(index) => {
                if let Some(spawn) = room.enemy_spawns.get(*index) {
                    let feet = Vec2::new(spawn.position_h.x_h, spawn.position_h.y_h);
                    let reach = Reach::enemy(&spawn.type_id, data);
                    launches.push((feet * H_WORLD_UNITS, -1.0, reach));
                    launches.push((feet * H_WORLD_UNITS, 1.0, reach));
                }
            }
            EditorItem::PlayerSpawn => {
                let feet = Vec2::new(room.player_spawn.x_h, room.player_spawn.y_h);
                launches.push((feet * H_WORLD_UNITS, -1.0, player));
                launches.push((feet * H_WORLD_UNITS, 1.0, player));
            }
            EditorItem::Exit | EditorItem::Bonus | EditorItem::Decoration(_) => {}
        }
    }
    if launches.is_empty()
        && let Some(cursor_h) = editor.cursor_h
    {
        launches.push((cursor_h * H_WORLD_UNITS, -1.0, player));
        launches.push((cursor_h * H_WORLD_UNITS, 1.0, player));
    }
    launches
}

/// A body's jump, in world units: how hard it leaves the ground, how fast it
/// moves through the air, and how far a dash adds to that.
#[derive(Clone, Copy)]
struct Reach {
    jump_impulse: f32,
    air_speed: f32,
    gravity: f32,
    dash_distance: Option<f32>,
    color: Color,
}

impl Reach {
    fn player(data: &GameData) -> Self {
        let player = &data.gameplay.player;
        Self {
            jump_impulse: player.jump_impulse_h_s * H_WORLD_UNITS,
            // The arc is the running jump, the one that reaches furthest: a
            // jump from standing rises just as high but lands shorter.
            air_speed: player.air_max_speed_h_s * H_WORLD_UNITS,
            gravity: player.gravity_h_s2 * H_WORLD_UNITS,
            dash_distance: Some(player.dash_distance_h * H_WORLD_UNITS),
            color: JUMP_COLOR,
        }
    }

    /// An enemy jumps to a height rather than with an impulse, and falls under
    /// the player's gravity; it has no dash.
    fn enemy(type_id: &EnemyTypeId, data: &GameData) -> Self {
        let jumping = &data.enemies.jumping;
        let height = data
            .enemies
            .types
            .get(type_id)
            .and_then(|enemy| enemy.jump_height_h)
            .unwrap_or(jumping.jump_height_h)
            * H_WORLD_UNITS;
        let gravity = data.gameplay.player.gravity_h_s2 * H_WORLD_UNITS;
        Self {
            jump_impulse: (2.0 * gravity * height).sqrt(),
            air_speed: jumping.air_speed_h_s * H_WORLD_UNITS,
            gravity,
            dash_distance: None,
            color: ENEMY_JUMP_COLOR,
        }
    }
}

/// The path a body leaving `feet` toward `direction` follows, and where it
/// touches down. Platforms stop it exactly as they do in play: it bumps its
/// head on an underside, stops against a side, and lands on a top. `dash`
/// spends the dash at the apex, which is where it shows best; wherever it is
/// spent it carries the jump the same distance.
fn arc(
    feet: Vec2,
    direction: f32,
    reach: Reach,
    dash: bool,
    room: &RoomConfig,
    platforms: &Platforms,
) -> (Vec<Vec2>, Option<Vec2>) {
    let room_width = room.size.width_h * H_WORLD_UNITS;
    let room_top = room.size.height_h * H_WORLD_UNITS;
    let slide = |from: Vec2, distance: f32| {
        let wanted = (from.x + direction * distance).clamp(
            HALF_BODY_WIDTH,
            (room_width - HALF_BODY_WIDTH).max(HALF_BODY_WIDTH),
        );
        Vec2::new(platforms.blocked_x(from.x, wanted, from.y), from.y)
    };

    let mut path = vec![feet];
    let mut position = feet;
    let mut vertical = reach.jump_impulse;
    let mut dash_left = dash.then_some(reach.dash_distance).flatten();
    let mut landing = None;
    for _ in 0..(MAX_FLIGHT_S / FLIGHT_STEP_S) as usize {
        // At the apex the dash slides the body sideways with gravity paused,
        // drawn in steps so a wall stops it part way.
        if let Some(distance) = dash_left.take_if(|_| vertical <= 0.0) {
            for _ in 0..8 {
                position = slide(position, distance / 8.0);
                path.push(position);
            }
        }
        let previous = position;
        // The rise over the step, and the speed left at its end. Taking the
        // two apart keeps the curve the parabola the tuning describes, where
        // stepping the speed first would cut `jump_height_h` short by half a
        // step's worth of fall.
        let rise = vertical * FLIGHT_STEP_S - reach.gravity * FLIGHT_STEP_S * FLIGHT_STEP_S / 2.0;
        vertical -= reach.gravity * FLIGHT_STEP_S;
        // The sideways step takes the step's highest feet, as the player's own
        // movement does, so a rise over a ledge is never cut short.
        let rising_feet = previous.y.max(previous.y + rise);
        position = slide(
            Vec2::new(previous.x, rising_feet),
            reach.air_speed * FLIGHT_STEP_S,
        );
        position.y = previous.y + rise;
        if vertical > 0.0
            && let Some(bottom) = platforms.ceiling(
                position.x,
                previous.y + H_WORLD_UNITS,
                position.y + H_WORLD_UNITS,
            )
        {
            position.y = bottom - H_WORLD_UNITS;
            vertical = 0.0;
        }
        if position.y + H_WORLD_UNITS > room_top {
            position.y = room_top - H_WORLD_UNITS;
            vertical = vertical.min(0.0);
        }
        if vertical <= 0.0
            && let Some(platform) = platforms.landing(position.x, previous.y, position.y)
        {
            position.y = platform.top;
            path.push(position);
            landing = Some(position);
            break;
        }
        path.push(position);
        // A fall out of the room kills: the arc ends there.
        if position.y < 0.0 {
            break;
        }
    }
    (path, landing)
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
        let far_x = eyes.x + world::IDLE_FACING * range;
        gizmos.rect_2d(
            Vec2::new((eyes.x + far_x) / 2.0, eyes.y),
            Vec2::new(range, 2.0 * height),
            SIGHT_EDGE_COLOR,
        );
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
            && offset.x * world::IDLE_FACING >= 0.0
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

    /// A floor split by a 4 H gap, which a running jump clears.
    fn test_room() -> RoomConfig {
        yaml_serde::from_str(
            "
room_id: 3
chapter_id: 1
size: { width_h: 48, height_h: 27 }
background_profile: clouds_bright_blue
player_spawn: { x_h: 2.5, y_h: 1 }
platforms:
  - { label: floor, material: city_pavement, rect_h: { x: 0, y: 0, width: 10, height: 1 } }
  - { label: far side, material: city_pavement, rect_h: { x: 14, y: 0, width: 34, height: 1 } }
enemy_spawns: []
bonus: { x_h: 43, y_h: 1 }
exit: { x_h: 45, y_h: 1 }
next_room_id: 4
",
        )
        .expect("the test layout should parse")
    }

    #[test]
    fn a_jump_rises_to_the_tuned_height_and_a_dash_carries_it_further() {
        let asset_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/assets");
        let data = GameData::load(&asset_root).expect("Chapter 1 YAML should be valid");
        let room = test_room();
        let surfaces = room
            .platforms
            .iter()
            .map(|block| PlatformSurface::of(block, &data))
            .collect::<Vec<_>>();
        let platforms = Platforms(surfaces.iter().collect());
        let player = Reach::player(&data);
        // Off the right edge of the floor, over the gap.
        let feet = Vec2::new(10.0 * H_WORLD_UNITS, H_WORLD_UNITS);

        let (path, landing) = arc(feet, 1.0, player, false, &room, &platforms);
        let apex = path.iter().map(|point| point.y).fold(f32::MIN, f32::max);
        let rise_h = (apex - feet.y) / H_WORLD_UNITS;
        let tuned = data.gameplay.player.jump_height_h;
        assert!(
            (rise_h - tuned).abs() < 0.05,
            "the jump rose {rise_h} H, tuned for {tuned} H"
        );
        let landing = landing.expect("a jump over the gap lands on the far side");
        assert!(
            (landing.y - H_WORLD_UNITS).abs() < f32::EPSILON,
            "it lands on the far platform's top"
        );

        let (_, dashed) = arc(feet, 1.0, player, true, &room, &platforms);
        let dashed = dashed.expect("the dash lands too");
        let further_h = (dashed.x - landing.x) / H_WORLD_UNITS;
        let dash_distance_h = data.gameplay.player.dash_distance_h;
        assert!(
            (further_h - dash_distance_h).abs() < 0.2,
            "the dash carried the jump {further_h} H further, dash is {dash_distance_h} H"
        );
    }

    #[test]
    fn an_arc_into_a_gap_falls_out_of_the_room_without_landing() {
        let asset_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/assets");
        let data = GameData::load(&asset_root).expect("Chapter 1 YAML should be valid");
        let mut room = test_room();
        // Only the near floor is left, so a jump off its right edge has
        // nothing to come down on.
        room.platforms.truncate(1);
        let surfaces = room
            .platforms
            .iter()
            .map(|block| PlatformSurface::of(block, &data))
            .collect::<Vec<_>>();
        let platforms = Platforms(surfaces.iter().collect());
        let feet = Vec2::new(10.0 * H_WORLD_UNITS, H_WORLD_UNITS);
        let (path, landing) = arc(feet, 1.0, Reach::player(&data), false, &room, &platforms);
        assert_eq!(landing, None, "nothing holds the body up past the ledge");
        assert!(
            path.last().is_some_and(|point| point.y < 0.0),
            "the arc ends below the room, where the fall kills"
        );
    }

    #[test]
    fn a_platform_cuts_a_sight_line_short_and_open_air_leaves_it_whole() {
        let asset_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/assets");
        let data = GameData::load(&asset_root).expect("Chapter 1 YAML should be valid");
        let mut room = test_room();
        let surfaces = room
            .platforms
            .iter()
            .map(|block| PlatformSurface::of(block, &data))
            .collect::<Vec<_>>();
        let platforms = Platforms(surfaces.iter().collect());
        // Eye level above the floor, looking across the room.
        let eyes = Vec2::new(20.0, 2.0) * H_WORLD_UNITS;
        let target = Vec2::new(30.0, 2.0) * H_WORLD_UNITS;
        assert_eq!(visible_fraction(eyes, target, &platforms), 1.0);

        // A pillar halfway stops the line there.
        room.platforms.push(
            yaml_serde::from_str(
                "{ material: city_pavement, rect_h: { x: 25, y: 1, width: 1, height: 4 } }",
            )
            .expect("the pillar should parse"),
        );
        let surfaces = room
            .platforms
            .iter()
            .map(|block| PlatformSurface::of(block, &data))
            .collect::<Vec<_>>();
        let platforms = Platforms(surfaces.iter().collect());
        let seen = visible_fraction(eyes, target, &platforms);
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
