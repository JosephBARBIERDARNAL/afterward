use super::*;

#[test]
fn the_pause_menu_adjusts_settings_resumes_restarts_and_leaves() {
    let mut game = HeadlessGame::new();
    game.start_run();
    game.hit_player(10.0);
    let max_health = game.session().max_health;
    assert!(game.session().health < max_health);

    game.tap(KeyCode::Escape);
    assert_eq!(game.screen(), GameScreen::Paused);
    game.tap(KeyCode::KeyS);
    assert_eq!(game.screen(), GameScreen::Settings);
    let music = game
        .app
        .world()
        .resource::<PlayerSettings>()
        .music_volume_percent;
    game.tap(KeyCode::ArrowLeft);
    assert_eq!(
        game.app
            .world()
            .resource::<PlayerSettings>()
            .music_volume_percent,
        music.saturating_sub(5)
    );
    game.tap(KeyCode::Escape);
    assert_eq!(game.screen(), GameScreen::Paused);
    game.tap(KeyCode::Enter);
    assert_eq!(game.screen(), GameScreen::Playing);

    // Restart asks first, and Escape keeps the run.
    game.tap(KeyCode::Escape);
    game.tap(KeyCode::KeyR);
    assert_eq!(game.screen(), GameScreen::ConfirmLeave);
    game.tap(KeyCode::Escape);
    assert_eq!(game.screen(), GameScreen::Paused);
    assert!(game.session().health < max_health, "the run goes on");

    // Its own letter again restarts the run in the start room, at full health.
    game.tap(KeyCode::KeyR);
    game.tap(KeyCode::KeyR);
    game.wait_for_screen(GameScreen::Playing, 10.0);
    assert_eq!(game.session().room_id, 0, "back in the opening room");
    assert_eq!(game.session().health, max_health);

    // Leaving asks too: Enter stands on the way back, and the second
    // button gives the run up.
    game.tap(KeyCode::Escape);
    game.tap(KeyCode::KeyM);
    assert_eq!(game.screen(), GameScreen::ConfirmLeave);
    game.tap(KeyCode::Enter);
    assert_eq!(game.screen(), GameScreen::Paused, "Enter keeps playing");
    game.tap(KeyCode::KeyM);
    game.tap(KeyCode::ArrowDown);
    game.tap(KeyCode::Enter);
    assert_eq!(game.screen(), GameScreen::MainMenu);
    assert!(game.enemies().is_empty(), "leaving the run clears the room");
}

#[test]
fn menu_letters_follow_the_keyboard_layout_and_q_quits() {
    let mut game = HeadlessGame::new();
    game.start_run();

    // AZERTY: the key labeled M sits at QWERTY's semicolon.
    game.tap(KeyCode::Escape);
    game.typed_key(
        KeyCode::Semicolon,
        Key::Character("m".into()),
        ButtonState::Pressed,
    );
    game.app.update();
    game.typed_key(
        KeyCode::Semicolon,
        Key::Character("m".into()),
        ButtonState::Released,
    );
    game.app.update();
    assert_eq!(game.screen(), GameScreen::ConfirmLeave);
    // The same key again confirms it.
    game.typed_key(
        KeyCode::Semicolon,
        Key::Character("m".into()),
        ButtonState::Pressed,
    );
    game.app.update();
    game.typed_key(
        KeyCode::Semicolon,
        Key::Character("m".into()),
        ButtonState::Released,
    );
    game.app.update();
    assert_eq!(game.screen(), GameScreen::MainMenu);
    assert!(game.app.should_exit().is_none());

    // AZERTY: the key labeled Q sits at QWERTY's A.
    game.typed_key(
        KeyCode::KeyA,
        Key::Character("q".into()),
        ButtonState::Pressed,
    );
    game.app.update();
    assert_eq!(game.app.should_exit(), Some(AppExit::Success));
}

#[test]
fn the_pause_menu_quits_the_game() {
    let mut game = HeadlessGame::new();
    game.start_run();
    game.tap(KeyCode::Escape);
    game.tap(KeyCode::KeyQ);
    assert_eq!(game.app.should_exit(), Some(AppExit::Success));
}
