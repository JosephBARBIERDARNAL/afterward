use super::*;
use crate::game::enemy_markers::EnemyMarker;

/// A hall two screens wide, so the scenario survives edits to the authored
/// rooms: one enemy by the door, in view, and one at the far end, out of it.
const LONG_HALL: &str = "
room_id: 1
chapter_id: 1
size: { width_h: 48, height_h: 13.5 }
background_profiles: [city_teal_evening]
player_spawn: { x_h: 3, y_h: 1 }
platforms:
  - { material: city_pavement, rect_h: { x: 0, y: 0, width: 48, height: 1 } }
enemy_spawns:
  - { type_id: blade_unit, position_h: { x: 12, y: 1 } }
  - { type_id: rifle_unit, position_h: { x: 44, y: 1 } }
bonus: { x_h: 40, y_h: 1 }
exit: { x_h: 46, y_h: 1 }
";

#[test]
fn an_enemy_out_of_view_is_pointed_at_from_the_edge_until_the_markers_are_turned_off() {
    let mut game = HeadlessGame::new();
    *game
        .app
        .world_mut()
        .resource_mut::<GameData>()
        .rooms
        .get_mut(&1)
        .expect("room 1 should exist") =
        yaml_serde::from_str(LONG_HALL).expect("the hall should parse");
    game.start_run();
    game.wait_for_screen(GameScreen::Playing, 5.0);
    game.run_seconds(0.2);

    let marked = |game: &mut HeadlessGame| {
        let world = game.app.world_mut();
        world
            .query::<&EnemyMarker>()
            .iter(world)
            .map(|marker| marker.enemy)
            .collect::<Vec<_>>()
    };
    let far_one = {
        let world = game.app.world_mut();
        world
            .query::<(Entity, &EnemyBody)>()
            .iter(world)
            .find(|(_, enemy)| enemy.type_id == "rifle_unit")
            .map(|(entity, _)| entity)
            .expect("the far enemy stands there")
    };
    assert_eq!(
        marked(&mut game),
        [far_one],
        "only the enemy out of view is marked"
    );

    // Turned off in the settings, the marker goes.
    game.tap(KeyCode::Escape);
    game.tap(KeyCode::KeyS);
    assert_eq!(game.screen(), GameScreen::Settings);
    let switch = game.menu_button(MenuAction::ToggleEnemyMarkers);
    game.click_mouse_on(switch, Vec2::ZERO);
    assert!(!game.app.world().resource::<PlayerSettings>().enemy_markers);
    game.tap(KeyCode::Escape);
    game.tap(KeyCode::Escape);
    assert_eq!(game.screen(), GameScreen::Playing);
    game.run_seconds(0.2);
    assert!(marked(&mut game).is_empty(), "no marker while they are off");

    // Back on, it comes back, and goes once its enemy is in view.
    game.app
        .world_mut()
        .resource_mut::<PlayerSettings>()
        .enemy_markers = true;
    game.run_seconds(0.1);
    assert_eq!(marked(&mut game), [far_one]);
    game.app
        .world_mut()
        .get_mut::<Transform>(far_one)
        .expect("the far enemy is still there")
        .translation
        .x = 8.0 * 48.0;
    game.run_seconds(0.1);
    assert!(
        marked(&mut game).is_empty(),
        "an enemy in view has no marker"
    );
}
