//! The screen that stands between Begin and the run: how hard it should be.

use super::*;

#[test]
fn begin_asks_for_a_difficulty_over_the_water_and_opens_on_the_one_already_chosen() {
    let mut game = HeadlessGame::new();
    let water_shown = |game: &mut HeadlessGame| {
        let world = game.app.world_mut();
        *world
            .query_filtered::<&Visibility, With<Flood>>()
            .single(world)
            .expect("the menu water is laid at startup")
    };

    // Begin asks rather than starting: no run has opened yet.
    game.tap(KeyCode::Enter);
    assert_eq!(game.screen(), GameScreen::Difficulty);
    assert_eq!(
        water_shown(&mut game),
        Visibility::Visible,
        "the question stands over the same water as the menu"
    );

    // Escape goes back to the menu, with nothing chosen and nothing begun.
    game.tap(KeyCode::Escape);
    assert_eq!(game.screen(), GameScreen::MainMenu);

    // The screen opens on Normal, so Enter twice begins an ordinary run.
    game.tap(KeyCode::Enter);
    game.tap(KeyCode::Enter);
    game.wait_for_screen(GameScreen::Playing, 10.0);
    assert_eq!(game.session().difficulty, Difficulty::Normal);

    // Another run opens the screen on the choice the last one was begun on,
    // so one down from Normal is Hard.
    game.tap(KeyCode::Escape);
    game.tap(KeyCode::KeyM);
    assert_eq!(game.screen(), GameScreen::MainMenu);
    game.tap(KeyCode::Enter);
    game.tap(KeyCode::ArrowDown);
    game.tap(KeyCode::Enter);
    game.wait_for_screen(GameScreen::Playing, 10.0);
    assert_eq!(game.session().difficulty, Difficulty::Hard);

    // And the next one opens on Hard, where the run before left it.
    game.tap(KeyCode::Escape);
    game.tap(KeyCode::KeyM);
    game.tap(KeyCode::Enter);
    assert_eq!(game.screen(), GameScreen::Difficulty);
    game.tap(KeyCode::ArrowUp);
    game.tap(KeyCode::ArrowUp);
    game.tap(KeyCode::Enter);
    game.wait_for_screen(GameScreen::Playing, 10.0);
    assert_eq!(
        game.session().difficulty,
        Difficulty::Easy,
        "two up from Hard is Easy"
    );
}

#[test]
fn the_mouse_picks_a_difficulty_and_each_box_is_edged_in_its_own_colour() {
    let mut game = HeadlessGame::new();
    game.tap(KeyCode::Enter);
    assert_eq!(game.screen(), GameScreen::Difficulty);

    // Every box carries the colour its difficulty is named with in the data,
    // and no two of them are alike.
    let colors = [Difficulty::Easy, Difficulty::Normal, Difficulty::Hard].map(|level| {
        let wanted = Color::from(level.edge(&game.data().gameplay.difficulty));
        let button = game.menu_button(MenuAction::ChooseDifficulty(level));
        let border = game
            .app
            .world()
            .get::<BorderColor>(button)
            .expect("a menu box is drawn with a border")
            .top;
        assert_eq!(
            border.with_alpha(1.0),
            wanted,
            "{} is edged in its own colour",
            level.label()
        );
        wanted
    });
    assert!(
        colors[0] != colors[1] && colors[1] != colors[2] && colors[0] != colors[2],
        "the three difficulties never share an edge colour"
    );

    let easy = game.menu_button(MenuAction::ChooseDifficulty(Difficulty::Easy));
    game.click_mouse_on(easy, Vec2::new(420.0, 300.0));
    game.wait_for_screen(GameScreen::Playing, 10.0);
    assert_eq!(game.session().difficulty, Difficulty::Easy);
    assert_eq!(game.session().room_id, 0, "and the run opens in room 0");
}
