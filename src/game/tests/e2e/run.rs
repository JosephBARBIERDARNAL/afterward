use super::*;

#[test]
fn a_run_goes_from_the_menu_through_combat_the_bonus_and_the_door_to_room_two() {
    let mut game = HeadlessGame::new();
    game.start_run();
    assert_eq!(game.session().room_id, 1);
    let roster = game.data().rooms[&1].enemy_spawns.len();
    assert_eq!(game.enemies().len(), roster, "every spawn materializes");
    // The exit door's frame: 0 locked, 1 unlocked, 2 and up opening. While the
    // camera crosses into the next room, both rooms have one, so this reads the
    // one that has come furthest: the door just walked through.
    let door_frame = |game: &mut HeadlessGame| {
        let world = game.app.world_mut();
        world
            .query_filtered::<&Sprite, With<ExitDoor>>()
            .iter(world)
            .filter_map(|sprite| sprite.texture_atlas.as_ref())
            .map(|atlas| atlas.index)
            .max()
    };
    assert_eq!(door_frame(&mut game), Some(0), "the door starts locked");

    // Whether shots reach an enemy depends on the room's layout; the gun test
    // covers hitting. Here the fight is skipped to follow the room flow.
    game.defeat_enemies();
    game.wait_for_screen(GameScreen::RoomClear, 1.0);
    assert_eq!(
        door_frame(&mut game),
        Some(0),
        "clearing the room alone leaves the door locked"
    );
    game.take_bonus();
    let offers = game.session().offered_bonuses.clone();
    assert_eq!(offers.len(), 2);
    assert_ne!(offers.first(), offers.get(1), "the two offers differ");

    let carried = game.session().owned_bonuses.clone();
    game.tap(KeyCode::Digit1);
    // The screen holds for the droplet to land on the box that was pressed
    // before it hands the room back.
    assert_eq!(game.screen(), GameScreen::Bonus);
    game.wait_for_screen(GameScreen::Playing, 2.0);
    assert_eq!(
        game.session().owned_bonuses,
        [carried.clone(), offers[..1].to_vec()].concat()
    );
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
        [carried, offers[..1].to_vec()].concat(),
        "the build carries over"
    );
    assert!(
        game.session().health >= health,
        "health carries over between rooms"
    );
    let roster = game.data().rooms[&2].enemy_spawns.len();
    assert_eq!(game.enemies().len(), roster, "room 2's roster spawns");
}
