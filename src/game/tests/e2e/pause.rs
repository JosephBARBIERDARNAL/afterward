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
        .resource::<AudioSettings>()
        .music_volume_percent;
    game.tap(KeyCode::ArrowLeft);
    assert_eq!(
        game.app
            .world()
            .resource::<AudioSettings>()
            .music_volume_percent,
        music.saturating_sub(5)
    );
    game.tap(KeyCode::Escape);
    assert_eq!(game.screen(), GameScreen::Paused);
    game.tap(KeyCode::Enter);
    assert_eq!(game.screen(), GameScreen::Playing);

    // Restart begins the run again in the start room, at full health.
    game.tap(KeyCode::Escape);
    game.tap(KeyCode::KeyR);
    game.wait_for_screen(GameScreen::Playing, 10.0);
    assert_eq!(game.session().room_id, 1);
    assert_eq!(game.session().health, max_health);

    game.tap(KeyCode::Escape);
    game.tap(KeyCode::KeyM);
    assert_eq!(game.screen(), GameScreen::MainMenu);
    assert!(game.enemies().is_empty(), "leaving the run clears the room");
}
