use super::*;

#[test]
fn a_run_goes_from_the_menu_through_combat_the_bonus_and_the_door_to_room_two() {
    let mut game = HeadlessGame::new();
    game.start_run();
    assert_eq!(game.session().room_id, 1);
    let roster = game.data().rooms[&1].enemy_spawns.len();
    assert_eq!(game.enemies().len(), roster, "every spawn materializes");
    // The exit door's frame: 0 locked, 1 unlocked, 2 and up opening.
    let door_frame = |game: &mut HeadlessGame| {
        let world = game.app.world_mut();
        world
            .query_filtered::<&Sprite, With<ExitDoor>>()
            .single(world)
            .ok()
            .and_then(|sprite| sprite.texture_atlas.as_ref())
            .map(|atlas| atlas.index)
    };
    assert_eq!(door_frame(&mut game), Some(0), "the door starts locked");

    // Whether shots reach an enemy depends on the room's layout; the gun test
    // covers hitting. Here the fight is skipped to follow the room flow.
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
    assert_eq!(door_frame(&mut game), Some(1), "and unlocks its sprite");
    let health = game.session().health;

    game.leave_through_the_door();
    game.run_seconds(0.2);
    assert!(
        door_frame(&mut game) >= Some(2),
        "touching the unlocked door opens it"
    );
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
