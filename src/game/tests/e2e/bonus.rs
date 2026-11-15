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

/// The words the focused offer leans on are explained in boxes at the bottom
/// right, and they follow the focus to the next offer.
#[test]
fn the_focused_offer_is_explained_at_the_bottom_right() {
    let mut game = HeadlessGame::new();
    game.begin_run();
    game.defeat_enemies();
    game.take_bonus();
    // The title of each box, in the order the column stands them: a box
    // holds one text, whose first span names what it explains.
    let glossed = |game: &mut HeadlessGame| {
        let world = game.app.world_mut();
        let column = world
            .query_filtered::<Entity, With<Glossary>>()
            .single(world)
            .expect("the glossary column is laid at startup");
        let children = |entity: Entity| {
            world
                .get::<Children>(entity)
                .map(|children| children.to_vec())
                .unwrap_or_default()
        };
        children(column)
            .into_iter()
            .map(|card| {
                let text = children(card)[0];
                let title = children(text)[0];
                world
                    .get::<TextSpan>(title)
                    .expect("a box's text opens on its title")
                    .0
                    .trim()
                    .to_owned()
            })
            .collect::<Vec<_>>()
    };
    let expected = |game: &HeadlessGame, index: usize| {
        let data = game.data();
        let offer = &game.session().offered_bonuses[index];
        let bonus = data
            .bonuses
            .bonuses
            .iter()
            .find(|bonus| bonus.id == *offer)
            .expect("every offer is a bonus of the pool");
        data.bonuses
            .explain(bonus)
            .into_iter()
            .take(4)
            .map(|explanation| match explanation {
                crate::data::Explanation::Term(term) => term.term.clone(),
                crate::data::Explanation::Bonus(named) => named.name.clone(),
            })
            .collect::<Vec<_>>()
    };

    let first = expected(&game, 0);
    assert_eq!(glossed(&mut game), first, "the first offer is explained");
    game.tap(KeyCode::ArrowDown);
    let second = expected(&game, 1);
    assert_eq!(glossed(&mut game), second, "the boxes follow the focus");

    game.tap(KeyCode::Digit1);
    game.wait_for_screen(GameScreen::Playing, 3.0);
    assert!(
        glossed(&mut game).is_empty(),
        "and go with the bonus screen"
    );
}
