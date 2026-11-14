use super::*;
use crate::{data::RoomConfig, game::devices::Device};

/// A small hall of its own holding `devices`, so the scenarios survive edits
/// to the authored rooms: a floor, the player on it at x 3, and no enemies.
fn hall(devices: &str) -> RoomConfig {
    yaml_serde::from_str(&format!(
        "
room_id: 1
chapter_id: 1
size: {{ width_h: 24, height_h: 13.5 }}
background_profiles: [sci_fi_industrial]
player_spawn: {{ x_h: 3, y_h: 1 }}
platforms:
  - {{ material: sci_fi_riveted_wall, rect_h: {{ x: 0, y: 0, width: 24, height: 1 }} }}
devices:
{devices}
enemy_spawns: []
bonus: {{ x_h: 20, y_h: 1 }}
exit: {{ x_h: 22, y_h: 1 }}
next_room_id: 2
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

fn player_x(game: &mut HeadlessGame) -> f32 {
    let world = game.app.world_mut();
    world
        .query_filtered::<&Transform, With<PlayerBody>>()
        .single(world)
        .expect("the player stands in the room")
        .translation
        .x
}

/// How many of the room's devices are solid right now.
fn solid_devices(game: &mut HeadlessGame) -> usize {
    let world = game.app.world_mut();
    world
        .query_filtered::<(), (With<Device>, With<PlatformSurface>)>()
        .iter(world)
        .count()
}

#[test]
fn a_laser_costs_nothing_while_off_or_warning_and_hurts_once_on() {
    // Its clock starts as the room is built, so it waits long enough for
    // the walk in before it warns.
    let mut game = play_in(hall(
        "  - { kind: laser, position_h: { x: 3, y: 1 }, length_h: 3, off_s: 30, warn_s: 0.5, on_s: 2 }",
    ));
    let health = game.session().health;
    // Wind it to the end of its off stretch.
    {
        let world = game.app.world_mut();
        for mut device in world.query::<&mut Device>().iter_mut(world) {
            device.elapsed_s = 29.9;
        }
    }
    game.run_seconds(0.4);
    assert_eq!(game.session().health, health, "off, then warning");
    game.run_seconds(0.5);
    assert!(game.session().health < health, "the beam is on and hurts");
}

#[test]
fn a_button_opens_the_door_sharing_its_link() {
    let mut game = play_in(hall(
        "  - { kind: button, position_h: { x: 3.5, y: 1 }, link: 4 }
  - { kind: door, position_h: { x: 8, y: 1 }, link: 4 }
  - { kind: door, position_h: { x: 12, y: 1 }, link: 5 }",
    ));
    assert_eq!(solid_devices(&mut game), 2, "both doors stand closed");
    game.tap(KeyCode::KeyE);
    game.run_seconds(1.0);
    assert_eq!(
        solid_devices(&mut game),
        1,
        "the door of link 4 opened, the other did not"
    );
}

#[test]
fn a_belt_carries_whoever_stands_on_it() {
    let mut game = play_in(hall(
        "  - { kind: conveyor, position_h: { x: 6, y: 0.75 }, cells: 16, speed_h_s: 3 }",
    ));
    let before = player_x(&mut game);
    game.run_seconds(1.0);
    let carried = (player_x(&mut game) - before) / H_WORLD_UNITS;
    assert!(
        carried > 2.0,
        "a belt running at 3 H/s carried the player only {carried} H in a second"
    );
}

#[test]
fn a_folding_platform_gives_way_under_the_player_and_comes_back() {
    // The player walks in on the floor, onto a platform laid flush with it.
    let mut game = play_in(hall(
        "  - { kind: folding_platform, position_h: { x: 3, y: 0.59375 }, stand_s: 0.3, return_s: 1 }",
    ));
    game.run_seconds(0.1);
    let folded = |game: &mut HeadlessGame| solid_devices(game) == 0;
    game.run_seconds(0.5);
    assert!(folded(&mut game), "stood on, it gives way");
    // Back a second after it went, and the player, still standing there,
    // has not yet held it for its 0.3 s.
    game.run_seconds(0.85);
    assert!(!folded(&mut game), "and comes back");
}

#[test]
fn up_climbs_a_ladder_the_player_hangs_on_it_and_jump_lets_go() {
    let mut game = play_in(hall(
        "  - { kind: ladder, position_h: { x: 3, y: 1 }, height_h: 4 }",
    ));
    let feet = |game: &mut HeadlessGame| {
        let world = game.app.world_mut();
        world
            .query_filtered::<&Transform, With<PlayerBody>>()
            .single(world)
            .expect("the player stands in the room")
            .translation
            .y
            / H_WORLD_UNITS
            - 0.5
    };
    game.key(KeyCode::KeyW, ButtonState::Pressed);
    game.run_seconds(0.5);
    game.key(KeyCode::KeyW, ButtonState::Released);
    let climbed = feet(&mut game);
    assert!(
        climbed > 2.0,
        "half a second up the ladder reached only {climbed} H"
    );
    game.run_seconds(0.5);
    assert_eq!(
        feet(&mut game),
        climbed,
        "hanging on the ladder, nothing falls"
    );
    game.tap(KeyCode::Space);
    game.run_seconds(1.5);
    assert!(
        (feet(&mut game) - 1.0).abs() < 0.01,
        "let go, the player falls back to the floor"
    );
}
