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
