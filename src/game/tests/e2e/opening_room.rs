use super::*;

#[test]
fn the_empty_opening_room_offers_its_bonus_at_once_and_runs_into_room_one() {
    let mut game = HeadlessGame::new();
    assert_eq!(game.screen(), GameScreen::MainMenu);
    game.begin_run();
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
    game.run_seconds(0.2);
    let world = game.app.world_mut();
    assert_eq!(
        world
            .query_filtered::<Entity, (With<BonusPickup>, With<GlowAttached>)>()
            .iter(world)
            .count(),
        1,
        "the bonus item is lit by an aura"
    );
    // Standing on the item does not take it either.
    let spot = game.data().rooms[&0].bonus;
    let world = game.app.world_mut();
    for mut transform in world
        .query_filtered::<&mut Transform, With<PlayerBody>>()
        .iter_mut(world)
    {
        transform.translation.x = spot.x_h * H_WORLD_UNITS;
        transform.translation.y = (spot.y_h + 0.5) * H_WORLD_UNITS;
    }
    game.run_seconds(0.3);
    assert_eq!(game.screen(), GameScreen::Playing);
    let world = game.app.world_mut();
    assert_eq!(
        world
            .query_filtered::<&ChildOf, (With<InteractBadge>, With<Sprite>)>()
            .iter(world)
            .filter(|badge| world.get::<BonusPickup>(badge.parent()).is_some())
            .count(),
        1,
        "the item in reach wears the key that takes it"
    );
    game.take_bonus();
    let offers = game.session().offered_bonuses.clone();
    assert_eq!(offers.len(), 2);
    game.tap(KeyCode::Digit1);
    game.wait_for_screen(GameScreen::Playing, 2.0);
    assert_eq!(game.session().owned_bonuses, offers[..1]);
    assert_eq!(door_frame(&mut game), Some(1), "which unlocks the door");

    // Walking into the unlocked door is not enough: it is used with Interact,
    // and wears the key while the player stands within reach of it.
    game.step_toward_exit();
    game.key(KeyCode::KeyD, ButtonState::Pressed);
    game.run_seconds(0.3);
    game.key(KeyCode::KeyD, ButtonState::Released);
    assert_eq!(game.screen(), GameScreen::Playing);
    game.step_toward_exit();
    game.run_seconds(0.1);
    let world = game.app.world_mut();
    let badged = world
        .query_filtered::<&ChildOf, With<InteractBadge>>()
        .iter(world)
        .map(ChildOf::parent)
        .collect::<Vec<_>>();
    assert_eq!(badged.len(), 1, "one badge stands in the room");
    assert!(
        world.get::<ExitDoor>(badged[0]).is_some(),
        "and it stands over the door"
    );
    game.tap(KeyCode::KeyE);
    game.wait_for_screen(GameScreen::Transitioning, 3.0);
    game.skip_any_prologue();
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

/// The opening music carries the menu and the opening room, and hands over to
/// a chapter track as the run crosses into room 1, where the city it opened
/// under keeps playing.
#[test]
fn the_opening_music_gives_way_to_a_chapter_track_on_the_way_into_room_one() {
    let mut game = HeadlessGame::new();
    assert_eq!(
        game.layers(AudioClass::IntroMusic),
        1,
        "the game opens on its own music"
    );
    assert_eq!(game.layers(AudioClass::Ambience), 0);
    assert_eq!(game.layers(AudioClass::Music), 0);

    game.begin_run();
    assert_eq!(game.session().room_id, 0);
    assert_eq!(
        game.layers(AudioClass::IntroMusic),
        1,
        "and keeps playing through the opening room"
    );
    assert_eq!(
        game.layers(AudioClass::Music),
        0,
        "where no chapter track stands over it"
    );
    assert_eq!(
        game.layers(AudioClass::Ambience),
        1,
        "with the city rising under it"
    );

    game.take_bonus();
    game.tap(KeyCode::Digit1);
    game.leave_through_the_door();
    game.wait_for_screen(GameScreen::Playing, 15.0);
    assert_eq!(game.session().room_id, 1);
    assert_eq!(
        game.layers(AudioClass::Music),
        1,
        "room 1 opens on a chapter track"
    );
    assert_eq!(
        game.layers(AudioClass::Ambience),
        1,
        "the city plays on under it"
    );

    // The opening music takes `intro_music_fade_out_s` to bow out, and is gone
    // once it has.
    let fade_out_s = game.data().gameplay.audio.intro_music_fade_out_s;
    game.run_seconds(fade_out_s + 0.5);
    assert_eq!(
        game.layers(AudioClass::IntroMusic),
        0,
        "and the opening music has faded away"
    );
}
