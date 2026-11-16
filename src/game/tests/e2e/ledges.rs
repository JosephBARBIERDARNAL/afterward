use super::*;
use crate::data::RoomConfig;

/// A platform's side whose top is within reach is climbed over rather than
/// clung to: jumping into it carries the player onto its top, playing the
/// ledge climb on the way, even when the ledge is slabs stacked on one
/// another.
#[test]
fn jumping_into_a_low_ledge_climbs_onto_it() {
    let mut game = HeadlessGame::new();
    *game
        .app
        .world_mut()
        .resource_mut::<GameData>()
        .rooms
        .get_mut(&1)
        .expect("room 1 should exist") = yaml_serde::from_str::<RoomConfig>(
        "
room_id: 1
chapter_id: 1
size: { width_h: 24, height_h: 13.5 }
background_profiles: [sci_fi_industrial]
player_spawn: { x_h: 3, y_h: 1 }
platforms:
  - { material: sci_fi_riveted_wall, rect_h: { x: 0, y: 0, width: 24, height: 1 } }
  - { material: sci_fi_riveted_wall, rect_h: { x: 5, y: 1, width: 4, height: 1.7 } }
  - { material: sci_fi_riveted_wall, rect_h: { x: 5, y: 2.7, width: 4, height: 0.5 } }
enemy_spawns: []
bonus: { x_h: 20, y_h: 1 }
exit: { x_h: 22, y_h: 1 }
next_room_id: 2
",
    )
    .expect("the ledge room should parse");
    game.start_run();
    game.key(KeyCode::KeyD, ButtonState::Pressed);
    game.run_seconds(0.4);
    game.tap(KeyCode::Space);
    let mut climbed = false;
    let mut clung = false;
    for _ in 0..60 {
        game.run_seconds(1.0 / 60.0);
        let world = game.app.world_mut();
        let (body, sprite) = world
            .query::<(&PlayerBody, &AnimatedSprite)>()
            .single(world)
            .expect("the player stands in the room");
        climbed |= body.ledge_climb.is_some() && sprite.sheet == "player_ledge_climb";
        clung |= body.wall_cling.is_some();
    }
    game.key(KeyCode::KeyD, ButtonState::Released);
    assert!(climbed, "the ledge is climbed, with its own clip");
    assert!(!clung, "a ledge within reach is never clung to");
    let world = game.app.world_mut();
    let (body, transform) = world
        .query::<(&PlayerBody, &Transform)>()
        .single(world)
        .expect("the player stands in the room");
    let feet_h = transform.translation.y / H_WORLD_UNITS - 0.5;
    assert!(body.grounded, "the climb ends standing");
    assert!(
        (feet_h - 3.2).abs() < 0.01,
        "on the ledge's top, feet at {feet_h} H"
    );
    assert!(transform.translation.x > 5.0 * H_WORLD_UNITS);
}
