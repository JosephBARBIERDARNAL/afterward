use super::*;

#[test]
fn a_lethal_hit_plays_the_death_then_ends_the_run_and_a_new_run_starts_fresh() {
    let mut game = HeadlessGame::new();
    game.start_run();

    game.hit_player(1000.0);
    assert_eq!(
        game.screen(),
        GameScreen::Playing,
        "the death animation plays before the run ends"
    );
    let death_s = game.data().player_animations.clips.death.duration_s()
        + game.data().gameplay.player.death_hold_s;
    game.wait_for_screen(GameScreen::Dead, death_s + 0.5);

    // A run ends the way it began: on the water, with the opening music under
    // it. What the run itself was playing is gone.
    let world = game.app.world_mut();
    let water = *world
        .query_filtered::<&Visibility, With<Flood>>()
        .single(world)
        .expect("the menu water is laid at startup");
    assert_eq!(water, Visibility::Visible);
    assert_eq!(game.layers(AudioClass::IntroMusic), 1);
    assert_eq!(game.layers(AudioClass::Music), 0);
    assert_eq!(game.layers(AudioClass::Ambience), 0);

    game.tap(KeyCode::Enter);
    assert_eq!(game.screen(), GameScreen::MainMenu);
    assert!(game.enemies().is_empty(), "leaving the run clears the room");
    assert_eq!(
        game.layers(AudioClass::IntroMusic),
        1,
        "and the menu finds its music already playing"
    );

    game.start_run();
    assert_eq!(game.session().room_id, 1);
    assert_eq!(game.session().health, game.session().max_health);
}
