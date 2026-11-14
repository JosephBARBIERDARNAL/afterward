use super::*;
use crate::game::combat::Hazard;

/// A small hall of its own, so the scenario survives edits to the authored
/// rooms: one enemy standing there as it opens, one more in each later wave.
const HALL: &str = "
room_id: 1
chapter_id: 1
size: { width_h: 24, height_h: 13.5 }
background_profiles: [city_teal_evening]
player_spawn: { x_h: 3, y_h: 1 }
platforms:
  - { material: city_pavement, rect_h: { x: 0, y: 0, width: 24, height: 1 } }
enemy_spawns:
  - { type_id: blade_unit, position_h: { x: 18, y: 1 } }
  - { type_id: catto_rust, position_h: { x: 20, y: 1 }, wave: 1 }
  - { type_id: mad_ghost_gold, position_h: { x: 21, y: 1 }, wave: 2 }
bonus: { x_h: 20, y_h: 1 }
exit: { x_h: 22, y_h: 1 }
next_room_id: 2
";

#[test]
fn a_room_sends_its_waves_in_one_after_the_other_and_is_only_cleared_after_the_last() {
    let mut game = HeadlessGame::new();
    *game
        .app
        .world_mut()
        .resource_mut::<GameData>()
        .rooms
        .get_mut(&1)
        .expect("room 1 should exist") = yaml_serde::from_str(HALL).expect("the hall should parse");
    game.start_run();
    assert_eq!(game.enemies().len(), 1, "only the first wave stands there");

    let living = |game: &mut HeadlessGame| {
        let world = game.app.world_mut();
        world
            .query::<&EnemyBody>()
            .iter(world)
            .filter(|enemy| enemy.health > 0.0)
            .map(|enemy| (enemy.type_id.to_string(), enemy.engaged, enemy.armor))
            .collect::<Vec<_>>()
    };
    let kill_everyone = |game: &mut HeadlessGame| {
        let world = game.app.world_mut();
        for mut enemy in world.query::<&mut EnemyBody>().iter_mut(world) {
            enemy.health = 0.0;
        }
    };

    kill_everyone(&mut game);
    game.run_seconds(2.0);
    assert_eq!(
        game.screen(),
        GameScreen::Playing,
        "a wave is still to come"
    );
    assert!(!game.session().room_cleared);
    assert_eq!(
        living(&mut game),
        [(String::from("catto_rust"), true, 0)],
        "the second wave arrives knowing where the player is"
    );

    kill_everyone(&mut game);
    game.run_seconds(2.0);
    assert_eq!(
        living(&mut game),
        [(String::from("mad_ghost_gold"), true, 1)],
        "a gold monster walks in armored"
    );

    kill_everyone(&mut game);
    game.wait_for_screen(GameScreen::RoomClear, 1.0);
    assert!(
        game.session().flawless,
        "nothing touched the player, and the room says so"
    );

    // The door, once unlocked, promises the category of the next room's item.
    game.take_bonus();
    game.tap(KeyCode::Digit1);
    game.wait_for_screen(GameScreen::Playing, 3.0);
    game.run_seconds(0.1);
    assert!(game.session().next_bonus_category.is_some());
}

#[test]
fn a_patch_that_burns_warns_first_and_a_hit_ends_the_untouched_room() {
    let mut game = HeadlessGame::new();
    game.start_run();
    let health = game.session().health;
    assert!(game.session().untouched);
    let world = game.app.world_mut();
    let player = world
        .query_filtered::<&Transform, With<PlayerBody>>()
        .single(world)
        .expect("the player stands in the room")
        .translation
        .truncate();
    world.spawn((
        Flame {
            size: Vec2::splat(2.0 * H_WORLD_UNITS),
            heat: 0.0,
            burning: false,
            grate: false,
        },
        Hazard {
            area: Rect::from_center_half_size(player, Vec2::splat(H_WORLD_UNITS)),
            damage: 10.0,
            arm_s: 0.5,
            burn_s: 0.3,
            rest_s: None,
            elapsed_s: 0.0,
        },
    ));
    game.run_seconds(0.3);
    assert_eq!(game.session().health, health, "the warning costs nothing");
    game.run_seconds(0.4);
    assert!(game.session().health < health, "the burn does");
    assert!(!game.session().untouched);
    game.run_seconds(0.5);
    let world = game.app.world_mut();
    assert_eq!(
        world.query::<&Hazard>().iter(world).count(),
        0,
        "a patch that is not a vent goes once it has burned"
    );
}
