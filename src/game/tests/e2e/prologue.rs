use super::*;
use crate::game::{
    audio::TypingSound,
    prologue::{Prologue, PrologueButton, PrologueText},
};

/// The door out of the opening room leads through the words the city is
/// entered on: the water rises over the room, the lines are written out with
/// the typing heard under them, and they stand until the player presses the
/// button that walks in.
#[test]
fn the_city_is_spoken_for_before_the_run_walks_into_it() {
    let mut game = HeadlessGame::new();
    game.tap(KeyCode::Enter);
    game.wait_for_screen(GameScreen::Playing, 10.0);
    game.take_bonus();
    game.tap(KeyCode::Digit1);
    game.step_toward_exit();
    game.key(KeyCode::KeyD, ButtonState::Pressed);
    game.wait_for_screen(GameScreen::Transitioning, 3.0);
    game.key(KeyCode::KeyD, ButtonState::Released);

    game.wait_for_screen(GameScreen::Prologue, 3.0);
    let lines = game.data().gameplay.prologue.lines.join("\n\n");
    game.run_seconds(game.data().gameplay.prologue.veil_fade_s + 0.5);
    assert_eq!(
        game.water_shown(),
        Visibility::Visible,
        "the words are written over the water"
    );
    assert!(
        game.app.world().resource::<Prologue>().veil > 0.9,
        "which stands over the room the run is leaving"
    );
    assert_eq!(game.typing_sounds(), 1, "with the typing heard under them");
    // Written out a character at a time, with every character holding its
    // place from the first frame so the block never shifts under itself.
    let written = game.written_prologue();
    assert_ne!(written, lines, "the words are still being written");
    assert_eq!(written.chars().count(), lines.chars().count());
    assert!(
        written
            .chars()
            .zip(lines.chars())
            .all(|(written, full)| written == full || written == ' '),
        "what stands is the beginning of the words, not something else: {written:?}"
    );
    assert!(
        written.contains(char::is_alphabetic),
        "and some of it is up"
    );
    assert_eq!(
        game.way_in_shown(),
        Visibility::Hidden,
        "the way in waits until there is nothing left to read"
    );

    // Enter finishes the writing, and the typing stops with it.
    game.tap(KeyCode::Enter);
    assert_eq!(game.written_prologue(), lines);
    assert_eq!(game.typing_sounds(), 0);
    assert_eq!(
        game.screen(),
        GameScreen::Prologue,
        "the words stand until the player says to walk in"
    );
    assert_eq!(
        game.way_in_shown(),
        Visibility::Visible,
        "with the way in offered under them"
    );

    // The button is a button like any other: the mouse presses it too.
    let way_in = game.menu_button(MenuAction::EnterTheCity);
    game.click_mouse_on(way_in, Vec2::new(400.0, 500.0));
    assert_ne!(game.screen(), GameScreen::Prologue);

    game.wait_for_screen(GameScreen::Playing, 15.0);
    assert_eq!(game.session().room_id, 1);
    assert_eq!(
        game.water_shown(),
        Visibility::Hidden,
        "and the water drains away behind them"
    );
    assert_eq!(game.app.world().resource::<Prologue>().veil, 0.0);
}

impl HeadlessGame {
    fn water_shown(&mut self) -> Visibility {
        let world = self.app.world_mut();
        *world
            .query_filtered::<&Visibility, With<Flood>>()
            .single(world)
            .expect("the water is laid at startup")
    }

    fn typing_sounds(&mut self) -> usize {
        let world = self.app.world_mut();
        world
            .query_filtered::<Entity, With<TypingSound>>()
            .iter(world)
            .count()
    }

    fn way_in_shown(&mut self) -> Visibility {
        let world = self.app.world_mut();
        *world
            .query_filtered::<&Visibility, With<PrologueButton>>()
            .single(world)
            .expect("the way in is laid at startup")
    }

    fn written_prologue(&mut self) -> String {
        let world = self.app.world_mut();
        world
            .query_filtered::<&Text, With<PrologueText>>()
            .single(world)
            .expect("the prologue's words are laid at startup")
            .0
            .clone()
    }
}
