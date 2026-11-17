use super::*;
use crate::data::RoomConfig;

/// A small hall of its own holding `enemies`, so the scenarios survive edits
/// to the authored rooms: a floor and the player on it at x 3.
fn hall(enemies: &str) -> RoomConfig {
    yaml_serde::from_str(&format!(
        "
room_id: 1
chapter_id: 1
size: {{ width_h: 24, height_h: 13.5 }}
background_profiles: [sci_fi_industrial]
player_spawn: {{ x_h: 3, y_h: 1 }}
platforms:
  - {{ material: sci_fi_riveted_wall, rect_h: {{ x: 0, y: 0, width: 24, height: 1 }} }}
enemy_spawns:
{enemies}
bonus: {{ x_h: 20, y_h: 1 }}
exit: {{ x_h: 22, y_h: 1 }}
"
    ))
    .expect("the hall should parse")
}

/// Starts a run whose first room is `room`.
fn play_in(room: RoomConfig) -> HeadlessGame {
    let mut game = HeadlessGame::new();
    *game
        .app
        .world_mut()
        .resource_mut::<GameData>()
        .rooms
        .get_mut(&1)
        .expect("room 1 should exist") = room;
    game.start_run();
    game
}

#[test]
fn a_flying_mine_flies_at_the_player_and_goes_off_on_reaching_them() {
    let mut game = play_in(hall(
        "  - { type_id: sci_fi_orb, position_h: { x: 8, y: 4 } }",
    ));
    assert!(game.session().untouched);
    game.run_seconds(4.0);
    // It was the room's last enemy, so clearing the room gave the health
    // back: what shows the hit is that the room is no longer untouched.
    assert!(
        !game.session().untouched,
        "the mine reached the player and its blast hurt them"
    );
    assert!(game.enemies().is_empty(), "and it is gone");
}

#[test]
fn a_drone_hovers_over_the_player_and_drops_a_shot_on_them() {
    let mut game = play_in(hall(
        "  - { type_id: sci_fi_drone, position_h: { x: 7, y: 2 } }",
    ));
    let health = game.session().health;
    game.run_seconds(2.5);
    let drone_y = {
        let world = game.app.world_mut();
        world
            .query::<(&Transform, &EnemyBody)>()
            .iter(world)
            .map(|(transform, _)| transform.translation.y / H_WORLD_UNITS)
            .next()
            .expect("the drone is in the room")
    };
    assert!(
        drone_y > 3.5,
        "it climbed to hover above the player, to {drone_y} H"
    );
    game.run_seconds(3.0);
    assert!(
        game.session().health < health,
        "a shot it dropped hit the player"
    );
}
