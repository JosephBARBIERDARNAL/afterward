use super::*;

#[test]
fn a_reroll_replaces_every_offer_and_the_keyboard_picks_the_second() {
    let mut game = HeadlessGame::new();
    game.start_run();
    game.defeat_enemies();
    game.take_bonus();
    let first_offers = game.session().offered_bonuses.clone();
    let category = game
        .session()
        .bonus_category
        .clone()
        .expect("the item picked up names the category to offer");
    let rerolls = game.session().rerolls;

    game.tap(KeyCode::KeyR);
    let second_offers = game.session().offered_bonuses.clone();
    assert_eq!(game.session().rerolls, rerolls - 1);
    assert_eq!(first_offers.len(), 3, "a bonus item offers three bonuses");
    // A reroll draws from what the first three left, which is still a choice.
    assert!((2..=3).contains(&second_offers.len()));
    assert!(
        second_offers
            .iter()
            .all(|bonus| !first_offers.contains(bonus)),
        "a reroll offers new bonuses only: {first_offers:?} -> {second_offers:?}"
    );
    let offered_categories = first_offers
        .iter()
        .chain(&second_offers)
        .map(|id| {
            game.data()
                .bonuses
                .bonuses
                .iter()
                .find(|bonus| bonus.id == *id)
                .map(|bonus| bonus.category.clone())
                .expect("every offer is a bonus of the pool")
        })
        .collect::<BTreeSet<_>>();
    assert_eq!(
        offered_categories,
        BTreeSet::from([category]),
        "every offer comes from the picked-up item's category"
    );

    // Selection waits for the reroll animation, then Down + Enter picks the
    // second offer.
    let animation_s = game.data().gameplay.room_flow.bonus_reroll_animation_s;
    game.run_seconds(animation_s + 0.1);
    game.tap(KeyCode::ArrowDown);
    let droplets = |game: &mut HeadlessGame| {
        game.app
            .world_mut()
            .query_filtered::<Entity, With<Ripple>>()
            .iter(game.app.world())
            .count()
    };
    let walking_there = droplets(&mut game);
    game.tap(KeyCode::Enter);
    // The choice holds the screen up for a moment, with a droplet falling on
    // the box that was pressed; it goes with the screen.
    assert_eq!(game.screen(), GameScreen::Bonus);
    assert_eq!(
        droplets(&mut game),
        walking_there + 1,
        "a droplet falls on the chosen bonus"
    );
    game.wait_for_screen(GameScreen::Playing, 2.0);
    assert_eq!(
        droplets(&mut game),
        0,
        "and none is left hanging over the room"
    );
    assert_eq!(
        game.session().owned_bonuses.last(),
        second_offers.get(1),
        "picking the second offer adds it to the build"
    );
}
