use super::*;
use crate::game::{
    audio::TypingSound,
    prologue::{Prologue, PrologueText},
};

/// The door out of the opening room leads through the words the city is
/// entered on: the water rises over the room, the lines are typed out with the
/// typing heard under them, and a key hurries it along.
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
    let written = game.written_prologue();
    assert!(
        !written.is_empty() && lines.starts_with(&written) && written != lines,
        "a line is written out a character at a time, not all at once: {written:?}"
    );

    // The first key finishes the typing, and the typing stops with it.
    game.tap(KeyCode::Space);
    assert_eq!(game.written_prologue(), lines);
    assert_eq!(game.typing_sounds(), 0);
    assert_eq!(game.screen(), GameScreen::Prologue, "the words still stand");

    // The next one leaves for the city.
    game.tap(KeyCode::Space);
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
