//! Walking the menus and pressing their buttons, with the keyboard and with
//! the mouse, and the droplet that falls on whatever was touched.

use super::*;

#[test]
fn the_arrows_walk_a_menu_and_the_confirm_key_presses_the_button_reached() {
    let mut game = HeadlessGame::new();
    game.start_run();
    game.tap(KeyCode::Escape);
    assert_eq!(game.screen(), GameScreen::Paused);

    // The menu opens on its first button, so Enter still resumes, as it did
    // before any of them could be walked to.
    game.tap(KeyCode::Enter);
    assert_eq!(game.screen(), GameScreen::Playing);

    // Two buttons down from Continue is Restart; Enter presses that instead.
    game.tap(KeyCode::Escape);
    game.tap(KeyCode::ArrowDown);
    game.tap(KeyCode::ArrowDown);
    game.tap(KeyCode::Enter);
    game.wait_for_screen(GameScreen::Playing, 10.0);
    assert_eq!(game.session().room_id, 0, "back in the opening room");

    // The walk wraps, and each menu opens on its own first button again.
    game.tap(KeyCode::Escape);
    game.tap(KeyCode::ArrowUp);
    game.tap(KeyCode::Enter);
    assert_eq!(
        game.app.should_exit(),
        Some(AppExit::Success),
        "one button up from the first is Quit, at the end of the list"
    );
}

#[test]
fn the_mouse_points_at_a_button_and_a_click_presses_it() {
    let mut game = HeadlessGame::new();
    assert_eq!(game.screen(), GameScreen::MainMenu);

    // Clicking a button presses it wherever the focus happened to be.
    let settings = game.menu_button(MenuAction::OpenSettings);
    game.click_mouse_on(settings, Vec2::new(420.0, 300.0));
    assert_eq!(game.screen(), GameScreen::Settings);

    // Pointing at a button only walks the focus to it; the confirm key then
    // presses the one under the mouse rather than the one the menu opened on.
    let controls = game.menu_button(MenuAction::OpenControls);
    game.point_mouse_at(controls, Vec2::new(420.0, 360.0));
    game.tap(KeyCode::Enter);
    assert_eq!(game.screen(), GameScreen::Controls);

    let back = game.menu_button(MenuAction::Back);
    game.click_mouse_on(back, Vec2::new(420.0, 520.0));
    assert_eq!(game.screen(), GameScreen::Settings);
    let back = game.menu_button(MenuAction::Back);
    game.click_mouse_on(back, Vec2::new(420.0, 520.0));
    assert_eq!(game.screen(), GameScreen::MainMenu);
}

#[test]
fn a_droplet_falls_on_every_button_touched_and_dries_up_after_it() {
    let mut game = HeadlessGame::new();
    assert_eq!(game.droplets(), 0, "an untouched menu is still water");

    // An arrow onto another button drops one, and so does the confirm key on
    // the button it reaches.
    game.tap(KeyCode::ArrowDown);
    assert_eq!(game.droplets(), 1);
    game.tap(KeyCode::ArrowDown);
    assert_eq!(game.droplets(), 2);

    // Pointing at a button the mouse is already on does not splash again.
    let quit = game.menu_button(MenuAction::Quit);
    game.point_mouse_at(quit, Vec2::new(420.0, 460.0));
    let after_pointing = game.droplets();
    assert!(after_pointing > 2, "reaching a new button splashes");
    game.point_mouse_at(quit, Vec2::new(420.0, 460.0));
    assert_eq!(
        game.droplets(),
        after_pointing,
        "resting on the same button does not"
    );

    // They all dry up once they have played out.
    let ripple = &game.data().gameplay.effects.ripple;
    game.run_seconds(ripple.duration_s + 0.2);
    assert_eq!(game.droplets(), 0);
}

#[test]
fn the_controls_screen_rebinds_from_a_click_and_restores_its_defaults() {
    let mut game = HeadlessGame::new();
    game.tap(KeyCode::KeyS);
    game.tap(KeyCode::KeyC);
    assert_eq!(game.screen(), GameScreen::Controls);

    // Clicking a binding row listens for the key to put in its place.
    let jump = game.menu_button(MenuAction::Rebind(GameAction::Jump));
    game.click_mouse_on(jump, Vec2::new(300.0, 300.0));
    assert_eq!(
        game.app
            .world()
            .resource::<ControlBindings>()
            .rebinding
            .expect("the clicked row is listening"),
        GameAction::Jump
    );
    game.tap(KeyCode::KeyJ);
    assert_eq!(
        game.app
            .world()
            .resource::<ControlBindings>()
            .binding_label(GameAction::Jump),
        "J"
    );
    assert!(
        game.app
            .world()
            .resource::<ControlBindings>()
            .rebinding
            .is_none(),
        "and stops listening once it has one"
    );

    // Restoring the defaults is a button of its own now, and still the R key.
    let restore = game.menu_button(MenuAction::RestoreDefaultBindings);
    game.click_mouse_on(restore, Vec2::new(700.0, 520.0));
    assert_eq!(
        game.app
            .world()
            .resource::<ControlBindings>()
            .binding_label(GameAction::Jump),
        "Space"
    );
}
