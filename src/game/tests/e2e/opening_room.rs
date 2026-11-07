use super::*;

#[test]
fn the_empty_opening_room_opens_its_door_at_once_and_runs_into_room_one() {
    let mut game = HeadlessGame::new();
    assert_eq!(game.screen(), GameScreen::MainMenu);
    game.tap(KeyCode::Enter);
    game.wait_for_screen(GameScreen::Playing, 10.0);
    assert_eq!(game.session().room_id, 0, "a run opens in room 0");
    assert!(game.enemies().is_empty(), "with nothing to fight");

    // The exit door's frame: 0 locked, 1 unlocked, 2 and up opening.
    let world = game.app.world_mut();
    let door_frame = world
        .query_filtered::<&Sprite, With<ExitDoor>>()
        .single(world)
        .ok()
        .and_then(|sprite| sprite.texture_atlas.as_ref())
        .map(|atlas| atlas.index);
    assert_eq!(
        door_frame,
        Some(1),
        "the door stands unlocked from the start"
    );
    assert!(game.session().door_open);

    // Room 0 is one flat corridor, so running right is the whole room: it
    // leads to room 1 with no room-clear screen and no bonus on the way.
    game.key(KeyCode::KeyD, ButtonState::Pressed);
    game.wait_for_screen(GameScreen::Transitioning, 10.0);
    game.key(KeyCode::KeyD, ButtonState::Released);
    game.wait_for_screen(GameScreen::RoomIntro, 5.0);
    game.wait_for_screen(GameScreen::Playing, 10.0);
    assert_eq!(game.session().room_id, 1);
    assert!(
        game.session().owned_bonuses.is_empty(),
        "crossing the opening room grants no bonus"
    );
    assert_eq!(game.session().health, game.session().max_health);
    assert_eq!(
        game.enemies().len(),
        game.data().rooms[&1].enemy_spawns.len(),
        "room 1's roster waits on the other side"
    );
}
