use super::*;
use crate::game::hitbox_overlay::HitboxOverlay;

#[test]
fn b_shows_and_hides_the_hitbox_overlay_during_a_run() {
    let mut game = HeadlessGame::new();
    game.start_run();
    let visible = |game: &HeadlessGame| game.app.world().resource::<HitboxOverlay>().visible;
    assert!(!visible(&game), "the overlay starts hidden");
    game.tap(KeyCode::KeyB);
    assert!(visible(&game));
    game.run_seconds(0.5);
    assert_eq!(
        game.screen(),
        GameScreen::Playing,
        "drawing it keeps the game running"
    );
    game.tap(KeyCode::KeyB);
    assert!(!visible(&game));
}
