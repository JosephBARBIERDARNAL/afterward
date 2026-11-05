use super::*;

#[test]
fn a_run_goes_from_the_menu_through_combat_the_bonus_and_the_door_to_room_two() {
    let mut game = HeadlessGame::new();
    game.start_run();
    assert_eq!(game.session().room_id, 1);
    let roster = game.data().rooms[&1].enemy_spawns.len();
    assert_eq!(game.enemies().len(), roster, "every spawn materializes");

    // Holding Shoot fires at the enemies closing in from the right.
    game.hold(KeyCode::KeyJ, 4.0);
    // Shot enemies are hurt or already dead (and gone from the roster).
    let untouched = game
        .enemies()
        .iter()
        .filter(|(health, max_health)| health >= max_health)
        .count();
    assert!(untouched < roster, "the gun's bullets hit an enemy");

    game.defeat_enemies();
    game.wait_for_screen(GameScreen::RoomClear, 1.0);
    game.wait_for_screen(GameScreen::Bonus, 2.0);
    let offers = game.session().offered_bonuses.clone();
    assert_eq!(offers.len(), 2);
    assert_ne!(offers.first(), offers.get(1), "the two offers differ");

    game.tap(KeyCode::Digit1);
    assert_eq!(game.screen(), GameScreen::Playing);
    assert_eq!(game.session().owned_bonuses, offers[..1]);
    assert!(game.session().door_open, "picking a bonus opens the door");
    let health = game.session().health;

    game.step_toward_exit();
    game.key(KeyCode::KeyD, ButtonState::Pressed);
    game.wait_for_screen(GameScreen::Transitioning, 3.0);
    game.key(KeyCode::KeyD, ButtonState::Released);
    game.wait_for_screen(GameScreen::RoomIntro, 5.0);
    game.wait_for_screen(GameScreen::Playing, 10.0);
    assert_eq!(game.session().room_id, 2);
    assert_eq!(
        game.session().owned_bonuses,
        offers[..1],
        "the build carries over"
    );
    assert!(
        game.session().health >= health,
        "health carries over between rooms"
    );
    let roster = game.data().rooms[&2].enemy_spawns.len();
    assert_eq!(game.enemies().len(), roster, "room 2's roster spawns");
}
