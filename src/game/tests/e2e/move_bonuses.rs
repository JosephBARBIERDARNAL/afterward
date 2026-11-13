use super::*;

/// A hall of its own with a row of enemies ahead of the player, so that the
/// roll ends beside one of them however far its tuning carries it.
const HALL: &str = "
room_id: 1
chapter_id: 1
size: { width_h: 24, height_h: 13.5 }
background_profiles: [clouds_bright_blue]
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
fn aftershock_makes_the_end_of_a_roll_hit_what_stands_beside_it() {
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
        .owned_bonuses = vec![String::from("aftershock")];

    game.key(KeyCode::KeyS, ButtonState::Pressed);
    game.tap(KeyCode::Space);
    game.key(KeyCode::KeyS, ButtonState::Released);
    game.run_seconds(1.5);
    assert!(
        game.enemies()
            .into_iter()
            .any(|(health, max_health)| health < max_health),
        "the shock a roll ends in wounds the enemy beside it"
    );
}

#[test]
fn a_bonus_two_others_make_together_is_only_offered_to_a_run_owning_both() {
    let game = HeadlessGame::new();
    let riptide = game
        .data()
        .bonuses
        .bonuses
        .iter()
        .find(|bonus| bonus.id == "riptide")
        .expect("Riptide is a bonus");
    let mut session = RunSession {
        owned_bonuses: vec![String::from("aftershock")],
        ..default()
    };
    assert!(!session.may_be_offered(riptide), "half of it is not enough");
    session.owned_bonuses.push(String::from("undertow"));
    assert!(session.may_be_offered(riptide));
    session.owned_bonuses.push(String::from("riptide"));
    assert!(!session.may_be_offered(riptide), "nor is it offered twice");
}
