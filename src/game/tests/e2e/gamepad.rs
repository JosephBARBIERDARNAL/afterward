use super::*;
use bevy::input::gamepad::{
    GamepadConnection, GamepadConnectionEvent, RawGamepadAxisChangedEvent,
    RawGamepadButtonChangedEvent, RawGamepadEvent,
};

#[test]
fn a_gamepad_walks_the_menus_moves_the_player_and_pauses() {
    let mut game = HeadlessGame::new();
    let pad = game.app.world_mut().spawn_empty().id();
    game.app
        .world_mut()
        .write_message(GamepadConnectionEvent::new(
            pad,
            GamepadConnection::Connected {
                name: "DualSense Wireless Controller".into(),
                vendor_id: Some(0x054c),
                product_id: Some(0x0ce6),
            },
        ));
    game.app.update();

    // Pressed one frame, released the next, as `tap` does a key.
    let press = |game: &mut HeadlessGame, button: GamepadButton| {
        for value in [1.0, 0.0] {
            game.app.world_mut().write_message(RawGamepadEvent::Button(
                RawGamepadButtonChangedEvent::new(pad, button, value),
            ));
            game.app.update();
        }
    };
    let lean =
        |game: &mut HeadlessGame, value: f32| {
            game.app.world_mut().write_message(RawGamepadEvent::Axis(
                RawGamepadAxisChangedEvent::new(pad, GamepadAxis::LeftStickX, value),
            ));
        };
    let player_x = |game: &mut HeadlessGame| {
        let world = game.app.world_mut();
        world
            .query_filtered::<&Transform, With<PlayerBody>>()
            .single(world)
            .expect("the run has a player")
            .translation
            .x
    };

    // Cross begins a run and confirms the difficulty.
    assert_eq!(game.screen(), GameScreen::MainMenu);
    press(&mut game, GamepadButton::South);
    assert_eq!(game.screen(), GameScreen::Difficulty);
    press(&mut game, GamepadButton::South);
    game.wait_for_screen(GameScreen::Playing, 10.0);
    assert_eq!(
        game.app
            .world()
            .resource::<ControlBindings>()
            .primary_label(GameAction::Interact),
        "R1",
        "the prompts name the pad's buttons while it is in use"
    );

    // The stick walks the player, once the walk in is over.
    game.run_seconds(3.0);
    let before = player_x(&mut game);
    lean(&mut game, 1.0);
    game.run_seconds(0.5);
    assert!(
        player_x(&mut game) > before + H_WORLD_UNITS,
        "leaning the stick right walks the player right"
    );
    lean(&mut game, 0.0);
    game.run_seconds(0.2);
    let stopped = player_x(&mut game);
    game.run_seconds(0.2);
    assert!(
        (player_x(&mut game) - stopped).abs() < 0.1 * H_WORLD_UNITS,
        "letting the stick go stops the walk"
    );

    // Options pauses, Circle resumes.
    press(&mut game, GamepadButton::Start);
    assert_eq!(game.screen(), GameScreen::Paused);
    press(&mut game, GamepadButton::East);
    assert_eq!(game.screen(), GameScreen::Playing);

    // A key brings the keyboard's names back.
    game.tap(KeyCode::KeyD);
    assert_eq!(
        game.app
            .world()
            .resource::<ControlBindings>()
            .primary_label(GameAction::Interact),
        "E"
    );
}
