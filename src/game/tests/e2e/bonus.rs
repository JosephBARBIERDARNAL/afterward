use super::*;

#[test]
fn a_reroll_replaces_both_offers_and_the_keyboard_picks_the_second() {
    let mut game = HeadlessGame::new();
    game.start_run();
    game.defeat_enemies();
    game.wait_for_screen(GameScreen::Bonus, 3.0);
    let first_pair = game.session().offered_bonuses.clone();
    let rerolls = game.session().rerolls;

    game.tap(KeyCode::KeyR);
    let second_pair = game.session().offered_bonuses.clone();
    assert_eq!(game.session().rerolls, rerolls - 1);
    assert_eq!(second_pair.len(), 2);
    assert!(
        second_pair.iter().all(|bonus| !first_pair.contains(bonus)),
        "a reroll offers two new bonuses: {first_pair:?} -> {second_pair:?}"
    );

    // Selection waits for the reroll animation, then Down + Enter picks the
    // second offer.
    let animation_s = game.data().gameplay.room_flow.bonus_reroll_animation_s;
    game.run_seconds(animation_s + 0.1);
    game.tap(KeyCode::ArrowDown);
    game.tap(KeyCode::Enter);
    assert_eq!(game.screen(), GameScreen::Playing);
    assert_eq!(game.session().owned_bonuses, second_pair[1..]);
}
