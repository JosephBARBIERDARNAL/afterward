use super::*;
use crate::game::tells::{Overhead, OverheadKind};

/// A hall of its own, with a row of idle enemies ahead of the player, near
/// enough that the shock a dash ends in reaches the first of them.
const HALL: &str = "
room_id: 1
chapter_id: 1
size: { width_h: 24, height_h: 13.5 }
background_profiles: [city_teal_evening]
player_spawn: { x_h: 3, y_h: 1 }
platforms:
  - { material: city_pavement, rect_h: { x: 0, y: 0, width: 24, height: 1 } }
enemy_spawns:
  - { type_id: heavy_unit, position_h: { x: 6, y: 1 } }
  - { type_id: heavy_unit, position_h: { x: 8.5, y: 1 } }
  - { type_id: heavy_unit, position_h: { x: 11, y: 1 } }
  - { type_id: heavy_unit, position_h: { x: 13.5, y: 1 } }
bonus: { x_h: 20, y_h: 1 }
exit: { x_h: 22, y_h: 1 }
next_room_id: 2
";

#[test]
fn a_bonus_at_work_is_seen_over_the_enemies_it_concerns_and_in_the_tray() {
    let mut game = HeadlessGame::new();
    *game
        .app
        .world_mut()
        .resource_mut::<GameData>()
        .rooms
        .get_mut(&1)
        .expect("room 1 should exist") = yaml_serde::from_str(HALL).expect("the hall should parse");
    game.start_run();
    game.app
        .world_mut()
        .resource_mut::<RunSession>()
        .owned_bonuses = ["aftershock", "marked_prey", "opportunist", "room_ward"]
        .map(String::from)
        .to_vec();
    game.run_seconds(0.1);
    let overheads = |game: &mut HeadlessGame, kind: OverheadKind| {
        game.app
            .world_mut()
            .query::<&Overhead>()
            .iter(game.app.world())
            .filter(|overhead| overhead.kind == kind)
            .count()
    };
    // The ones facing the player see them at once; the far ones do not.
    assert!(
        overheads(&mut game, OverheadKind::Unaware) >= 1,
        "Opportunist hangs a question mark over an enemy that has not noticed anyone"
    );
    assert_eq!(overheads(&mut game, OverheadKind::MarkIcon), 0);

    // A dash ends in Aftershock's shock, which marks what it reaches.
    game.tap(KeyCode::KeyL);
    game.run_seconds(1.0);
    assert!(
        overheads(&mut game, OverheadKind::MarkIcon) >= 1,
        "a marked enemy wears the reticle"
    );
    assert_eq!(
        overheads(&mut game, OverheadKind::MarkBar),
        overheads(&mut game, OverheadKind::MarkIcon),
        "with the bar of what is left of its mark"
    );
    let tray = game
        .app
        .world_mut()
        .query::<&Text>()
        .iter(game.app.world())
        .map(|text| text.0.clone())
        .collect::<Vec<_>>();
    assert!(
        tray.iter().any(|text| text == "ROOM WARD"),
        "the tray says Room Ward is still ready: {tray:?}"
    );
}
