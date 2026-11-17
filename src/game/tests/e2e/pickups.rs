use super::*;
use crate::{data::RoomConfig, game::combat::HealthPickup};

#[test]
fn a_health_drop_falls_to_the_floor_under_where_it_dropped() {
    // A small hall of its own, so the scenario survives edits to the
    // authored rooms: a floor 1 H thick and no enemies.
    let room: RoomConfig = yaml_serde::from_str(
        "
room_id: 1
chapter_id: 1
size: { width_h: 24, height_h: 13.5 }
background_profiles: [sci_fi_industrial]
player_spawn: { x_h: 3, y_h: 1 }
platforms:
  - { material: sci_fi_riveted_wall, rect_h: { x: 0, y: 0, width: 24, height: 1 } }
enemy_spawns: []
bonus: { x_h: 20, y_h: 1 }
exit: { x_h: 22, y_h: 1 }
",
    )
    .expect("the hall should parse");
    let mut game = HeadlessGame::new();
    *game
        .app
        .world_mut()
        .resource_mut::<GameData>()
        .rooms
        .get_mut(&1)
        .expect("room 1 should exist") = room;
    game.start_run();

    // Dropped in mid-air, as by a body struck down while it jumped.
    let half_height = game.data().gameplay.health_pickup.size_h * H_WORLD_UNITS / 2.0;
    game.app.world_mut().spawn((
        Transform::from_xyz(12.0 * H_WORLD_UNITS, 6.0 * H_WORLD_UNITS, 0.8),
        HealthPickup {
            healing: 10.0,
            fall_speed: 0.0,
        },
        RoomEntity,
    ));
    game.run_seconds(2.0);

    let world = game.app.world_mut();
    let lying = world
        .query_filtered::<&Transform, With<HealthPickup>>()
        .single(world)
        .expect("the drop is still there")
        .translation;
    assert!(
        (lying.y - half_height - H_WORLD_UNITS).abs() < 0.01,
        "it lies on the floor, at {} H",
        (lying.y - half_height) / H_WORLD_UNITS
    );
}
