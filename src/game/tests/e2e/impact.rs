use super::*;
use crate::game::impact::Impact;

/// A landed blow holds the whole game still for a moment, and the game always
/// comes back out of it — even when the player pauses in the middle of one.
#[test]
fn a_landed_blow_freezes_the_game_for_a_moment_and_then_lets_it_go() {
    let mut game = HeadlessGame::new();
    game.begin_run();
    let frozen = |game: &HeadlessGame| game.app.world().resource::<Time<Virtual>>().is_paused();
    assert!(!frozen(&game), "nothing holds the game still at first");

    game.app.world_mut().spawn(Impact {
        freeze_s: 0.2,
        shake_h: 0.1,
    });
    game.run_seconds(0.05);
    assert!(frozen(&game), "the blow holds the game still");
    game.run_seconds(0.5);
    assert!(!frozen(&game), "and lets it go once the moment has passed");

    game.app.world_mut().spawn(Impact {
        freeze_s: 0.2,
        shake_h: 0.1,
    });
    game.run_seconds(0.05);
    game.tap(KeyCode::Escape);
    assert_eq!(game.screen(), GameScreen::Paused);
    game.run_seconds(0.5);
    assert!(
        !frozen(&game),
        "a pause in the middle of it does not trap it"
    );
}
