use super::*;

#[test]
fn the_flood_shows_only_while_the_main_menu_is_up() {
    let mut game = HeadlessGame::new();
    assert_eq!(game.screen(), GameScreen::MainMenu);

    // The water is one quad, laid under wherever the camera stands.
    let water = |game: &mut HeadlessGame| {
        let world = game.app.world_mut();
        world
            .query_filtered::<(&Visibility, &Transform), With<Flood>>()
            .single(world)
            .map(|(visibility, transform)| (*visibility, transform.translation.truncate()))
            .expect("the menu water is laid at startup")
    };
    assert_eq!(water(&mut game).0, Visibility::Visible);

    game.start_run();
    assert_eq!(
        water(&mut game).0,
        Visibility::Hidden,
        "the water goes once the run starts"
    );

    game.tap(KeyCode::Escape);
    game.tap(KeyCode::KeyM);
    assert_eq!(game.screen(), GameScreen::MainMenu);
    game.app.update();
    let (visibility, place) = water(&mut game);
    assert_eq!(
        visibility,
        Visibility::Visible,
        "and comes back with the menu"
    );
    let world = game.app.world_mut();
    let camera = world
        .query_filtered::<&Transform, With<Camera2d>>()
        .single(world)
        .expect("the camera is there")
        .translation
        .truncate();
    assert_eq!(place, camera, "under the camera, wherever the run left it");
}

#[test]
fn the_main_menu_opens_the_settings_and_comes_back_over_its_water() {
    let mut game = HeadlessGame::new();
    assert_eq!(game.screen(), GameScreen::MainMenu);

    let water_shown = |game: &mut HeadlessGame| {
        let world = game.app.world_mut();
        *world
            .query_filtered::<&Visibility, With<Flood>>()
            .single(world)
            .expect("the menu water is laid at startup")
    };

    game.tap(KeyCode::KeyS);
    assert_eq!(game.screen(), GameScreen::Settings);
    assert_eq!(
        water_shown(&mut game),
        Visibility::Visible,
        "the settings stand over the menu's water"
    );

    // The same settings the pause menu opens: the sliders answer to the same
    // keys, and Interact opens the controls.
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
    game.tap(KeyCode::KeyE);
    assert_eq!(game.screen(), GameScreen::Controls);
    assert_eq!(water_shown(&mut game), Visibility::Visible);
    game.tap(KeyCode::Escape);
    assert_eq!(game.screen(), GameScreen::Settings);

    game.tap(KeyCode::Escape);
    assert_eq!(
        game.screen(),
        GameScreen::MainMenu,
        "settings opened from the main menu go back to it"
    );

    // Opened from a paused run, they go back to the pause menu instead.
    game.start_run();
    game.tap(KeyCode::Escape);
    game.tap(KeyCode::KeyS);
    assert_eq!(game.screen(), GameScreen::Settings);
    assert_eq!(
        water_shown(&mut game),
        Visibility::Hidden,
        "over a run there is no menu water"
    );
    game.tap(KeyCode::Escape);
    assert_eq!(game.screen(), GameScreen::Paused);
}
