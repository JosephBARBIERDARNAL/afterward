use super::*;

#[test]
fn the_empty_opening_room_offers_its_bonus_at_once_and_runs_into_room_one() {
    let mut game = HeadlessGame::new();
    assert_eq!(game.screen(), GameScreen::MainMenu);
    game.tap(KeyCode::Enter);
    game.wait_for_screen(GameScreen::Playing, 10.0);
    assert_eq!(game.session().room_id, 0, "a run opens in room 0");
    assert!(game.enemies().is_empty(), "with nothing to fight");

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
    assert!(!game.session().door_open);

    // With no enemies there is nothing to clear, so the bonus item is simply
    // there from the start, with no room-clear flash on the way.
    assert_ne!(game.screen(), GameScreen::RoomClear);
    game.take_bonus();
    let offers = game.session().offered_bonuses.clone();
    assert_eq!(offers.len(), 2);
    game.tap(KeyCode::Digit1);
    assert_eq!(game.screen(), GameScreen::Playing);
    assert_eq!(game.session().owned_bonuses, offers[..1]);
    assert_eq!(door_frame(&mut game), Some(1), "which unlocks the door");

    game.key(KeyCode::KeyD, ButtonState::Pressed);
    game.wait_for_screen(GameScreen::Transitioning, 10.0);
    game.key(KeyCode::KeyD, ButtonState::Released);
    game.wait_for_screen(GameScreen::RoomIntro, 5.0);
    game.wait_for_screen(GameScreen::Playing, 10.0);
    assert_eq!(game.session().room_id, 1);
    assert_eq!(game.session().health, game.session().max_health);
    assert_eq!(
        game.enemies().len(),
        game.data().rooms[&1].enemy_spawns.len(),
        "room 1's roster waits on the other side"
    );
}
