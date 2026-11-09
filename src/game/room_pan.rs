//! The slide from one room into the next. Rooms are not cut apart by a fade:
//! the room being left is stood to the left of the one being entered, across a
//! dark gap of `room_gap_h`, and the camera crosses from one to the other over
//! `room_pan_s`. Each room keeps its own coordinates, so only the room on its
//! way out is displaced — and it is despawned the moment the camera lands.
//!
//! The player does not travel with the camera: they walk into the opening door
//! and dissolve into the portal turning over it, and `room_intro` forms them
//! back out of the portal over the entry door standing on the far side.

use super::*;

/// The portal turning over a doorway while a crossing is happening. It hangs
/// as a child of the door, so it stands where the door stands, is carried off
/// with the room the door belongs to, and goes the moment the door does.
#[derive(Component)]
pub(super) struct CrossingPortal;

/// The room being left behind, drawn to the left of the one being entered
/// until the camera has finished crossing.
#[derive(Component)]
pub(super) struct DepartingRoom;

/// Where the camera starts the slide and where it lands, how wide the room it
/// is leaving is, so that room can be stood clear of the next one, and whether
/// the room now opening was walked into through a door.
#[derive(Resource, Default)]
pub(super) struct RoomPan {
    pub(super) departing_width_h: f32,
    pub(super) came_through_a_door: bool,
    from: Vec2,
    to: Vec2,
}

#[derive(SystemParam)]
pub(super) struct DepartureResources<'w> {
    data: Res<'w, GameData>,
    pan: ResMut<'w, RoomPan>,
}

/// Everything the room being left is made of. A platform's collider is drawn
/// by nothing and so carries no `Transform`: it is displaced by nothing and
/// still has to be marked, or it would outlive its room as an invisible wall
/// in the next one.
type DepartingEntities<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        Option<&'static mut Transform>,
        Option<&'static mut BackgroundParallax>,
    ),
    (With<RoomEntity>, Without<Camera2d>),
>;

/// Stands the room being left to the left of the one about to be spawned, with
/// the camera moved along with it so nothing stirs on screen, and takes its
/// player away: they walked into the door.
pub(super) fn send_off_the_departing_room(
    mut commands: Commands,
    resources: DepartureResources,
    players: Query<Entity, With<PlayerBody>>,
    mut entities: DepartingEntities,
    mut camera: Query<&mut Transform, With<Camera2d>>,
) {
    let DepartureResources { data, mut pan } = resources;
    let gap_h = data.gameplay.room_flow.room_gap_h;
    let shift = -(pan.departing_width_h + gap_h) * H_WORLD_UNITS;
    for (entity, transform, parallax) in &mut entities {
        if players.contains(entity) {
            commands.entity(entity).despawn();
            continue;
        }
        commands.entity(entity).insert(DepartingRoom);
        if let Some(mut transform) = transform {
            transform.translation.x += shift;
        }
        if let Some(mut parallax) = parallax {
            parallax.base_x += shift;
        }
    }
    // The dark between the two rooms: a panel tall enough to fill the view,
    // hiding the skylines that carry on behind both of them.
    let view_height = data.gameplay.viewport.height_h * H_WORLD_UNITS;
    commands.spawn((
        Sprite::from_color(
            Color::BLACK,
            Vec2::new(gap_h * H_WORLD_UNITS, view_height * 3.0),
        ),
        Transform::from_xyz(-gap_h / 2.0 * H_WORLD_UNITS, view_height / 2.0, GAP_Z),
        DepartingRoom,
        RoomEntity,
    ));
    let Ok(mut camera) = camera.single_mut() else {
        return;
    };
    camera.translation.x += shift;
    pan.from = camera.translation.truncate();
}

/// Starts the slide: `world::spawn_room` has just stood the camera where the
/// new room opens, which is where the camera is headed, so it is put back
/// where the departing room left it and sent across — at a steady speed, so
/// a long crossing takes longer than a short one rather than every door
/// moving at a pace of its own.
pub(super) fn start_room_pan(
    data: Res<GameData>,
    mut pan: ResMut<RoomPan>,
    mut timer: ResMut<PhaseTimer>,
    mut next_screen: ResMut<NextState<GameScreen>>,
    mut camera: Query<&mut Transform, With<Camera2d>>,
) {
    let Ok(mut camera) = camera.single_mut() else {
        return;
    };
    pan.to = camera.translation.truncate();
    camera.translation.x = pan.from.x;
    camera.translation.y = pan.from.y;
    pan.came_through_a_door = true;
    let room_flow = &data.gameplay.room_flow;
    let travel_h = pan.from.distance(pan.to) / H_WORLD_UNITS;
    // The easing costs the crossing some of its speed, and the entrance needs
    // room to begin before the camera lands.
    let seconds = (travel_h / room_flow.room_pan_speed_h_s * EASING_SLACK)
        .max(room_flow.room_intro.early_start_s + SHORTEST_PAN_S);
    timer.0 = Timer::from_seconds(seconds, TimerMode::Once);
    next_screen.set(GameScreen::Panning);
}

/// Carries the camera across the gap, and lets the room behind it go once it
/// has landed.
pub(super) fn update_room_pan(
    mut commands: Commands,
    time: Res<Time>,
    pan: Res<RoomPan>,
    mut timer: ResMut<PhaseTimer>,
    mut next_screen: ResMut<NextState<GameScreen>>,
    departing: Query<Entity, With<DepartingRoom>>,
    mut camera: Query<&mut Transform, With<Camera2d>>,
) {
    timer.0.tick(time.delta());
    let progress = timer.0.fraction().clamp(0.0, 1.0);
    // Eased at both ends, so the view leans into the move and settles out of
    // it rather than starting and stopping flat.
    let eased = progress * progress * (3.0 - 2.0 * progress);
    if let Ok(mut camera) = camera.single_mut() {
        let center = pan.from.lerp(pan.to, eased);
        camera.translation.x = center.x;
        camera.translation.y = center.y;
    }
    if !timer.0.is_finished() {
        return;
    }
    for entity in &departing {
        commands.entity(entity).despawn();
    }
    next_screen.set(GameScreen::RoomIntro);
}

/// Walks the player the last step into the opening door and dissolves them
/// into the portal turning over it, so the room stands empty by the time the
/// camera slides off it.
pub(super) fn step_into_the_door(
    time: Res<Time>,
    data: Res<GameData>,
    sheets: Res<SpriteSheets>,
    doors: Query<&Transform, (With<ExitDoor>, Without<PlayerBody>)>,
    mut players: Query<
        (
            &mut PlayerBody,
            &mut Transform,
            &mut Sprite,
            &mut AnimatedSprite,
        ),
        Without<ExitDoor>,
    >,
) {
    let (Ok(door), Ok((mut body, mut transform, mut sprite, mut animation))) =
        (doors.single(), players.single_mut())
    else {
        return;
    };
    let step = data.gameplay.room_flow.room_intro.player_walk_in_speed_h_s
        * H_WORLD_UNITS
        * time.delta_secs();
    transform.translation.x = move_towards(transform.translation.x, door.translation.x, step);
    body.velocity = Vec2::ZERO;
    body.facing = 1.0;
    sprite.flip_x = false;
    // The body comes apart into swirls rather than fading out of the room: the
    // clip is played once and holds on its last frame, with nothing left of the
    // hero by the time the slide takes the room away.
    sprite.color = Color::WHITE;
    set_animation(
        &mut sprite,
        &mut animation,
        &sheets,
        "player_dissolve",
        false,
    );
}

/// Turns a portal over `door`, playing its frames once. Both sides of a
/// crossing call it: `flow::handle_door_touch` for the door being walked into,
/// `room_intro::open_the_entry_door` for the one being stepped out of.
pub(super) fn hang_a_portal(commands: &mut Commands, door: Entity, sheets: &SpriteSheets) {
    let Some(sheet) = sheets.0.get(PORTAL_SHEET) else {
        return;
    };
    let mut sprite = Sprite::from_atlas_image(
        sheet.image.clone(),
        TextureAtlas {
            layout: sheet.layout.clone(),
            index: 0,
        },
    );
    sprite.custom_size = Some(Vec2::splat(PORTAL_SPAN_H * H_WORLD_UNITS));
    commands.entity(door).with_child((
        sprite,
        Transform::from_xyz(0.0, 0.0, PORTAL_Z_OFFSET),
        AnimatedSprite {
            sheet: PORTAL_SHEET.to_string(),
            frame: 0,
            frames: sheet.frames,
            timer: Timer::from_seconds(1.0 / sheet.fps, TimerMode::Repeating),
            looping: false,
        },
        CrossingPortal,
    ));
}

/// Behind the platforms and the decorations of both rooms, in front of the
/// skylines that carry on past their walls.
const GAP_Z: f32 = -5.0;
/// Eased at both ends, the camera spends about this much longer crossing than
/// it would at a flat speed.
const EASING_SLACK: f32 = 1.5;
/// However short the crossing, it is still a move rather than a cut.
const SHORTEST_PAN_S: f32 = 0.25;
/// The clip the portal is drawn from: it opens and closes over its own frames.
const PORTAL_SHEET: &str = "effect_portal";
/// How tall the portal stands over a doorway, in H. A door is 1.5 H, and the
/// swirl needs room to turn around it.
const PORTAL_SPAN_H: f32 = 2.25;
/// In front of the door it hangs over, behind the body stepping through it.
const PORTAL_Z_OFFSET: f32 = 1.0;
