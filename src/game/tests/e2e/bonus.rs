use super::*;
use crate::game::bonus_screen::BonusChoice;

#[test]
fn a_reroll_replaces_every_offer_and_the_keyboard_picks_the_second() {
    let mut game = HeadlessGame::new();
    game.start_run();
    game.defeat_enemies();
    game.take_bonus();
    let first_offers = game.session().offered_bonuses.clone();
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
            .all(|offer| first_offers.iter().all(|first| first.id != offer.id)),
        "a reroll offers new bonuses only: {first_offers:?} -> {second_offers:?}"
    );
    assert!(
        first_offers.iter().chain(&second_offers).all(|offer| game
            .data()
            .bonuses
            .bonus(&offer.id)
            .is_some_and(|bonus| bonus.rarities.contains_key(&offer.rarity))),
        "every offer is a bonus of the pool, at one of its rarities"
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
    // opens on a text naming what it explains.
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
                world
                    .get::<Text>(children(card)[0])
                    .expect("a box opens on its title")
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
            .bonus(&offer.id)
            .expect("every offer is a bonus of the pool");
        data.bonuses
            .explain(&bonus.describe(offer.rarity), &bonus.id)
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

/// Tab opens the build window over the paused room, listing every bonus the
/// run owns; the arrows walk it, and Tab or Escape goes back to the fight.
#[test]
fn the_build_window_lists_the_run_and_closes_with_its_key_or_escape() {
    let mut game = HeadlessGame::new();
    game.start_run();
    let owned = game.session().owned_bonuses.len();
    assert!(owned >= 1, "room 0 paid out a bonus");

    game.tap(KeyCode::Tab);
    assert_eq!(game.screen(), GameScreen::Build);
    let chips = |game: &mut HeadlessGame| {
        let world = game.app.world_mut();
        world.query::<&BuildChip>().iter(world).count()
    };
    assert_eq!(chips(&mut game), owned, "one chip per owned bonus");
    let focus = |game: &HeadlessGame| game.app.world().resource::<BuildFocus>().index;
    assert_eq!(focus(&game), 0, "the window opens on its first bonus");

    game.tap(KeyCode::Tab);
    assert_eq!(game.screen(), GameScreen::Playing, "its own key closes it");

    // Holding more, the arrows walk the list and Escape closes it as well.
    let more = game
        .data()
        .bonuses
        .bonuses
        .iter()
        .filter(|bonus| !game.session().has_bonus(&bonus.id))
        .filter_map(|bonus| Some(BonusPick::new(&bonus.id, *bonus.rarities.keys().next()?)))
        .take(2)
        .collect::<Vec<_>>();
    game.app
        .world_mut()
        .resource_mut::<RunSession>()
        .owned_bonuses
        .extend(more);
    game.tap(KeyCode::Tab);
    assert_eq!(chips(&mut game), owned + 2, "the chips are laid again");
    game.tap(KeyCode::ArrowDown);
    assert_eq!(focus(&game), 1);
    game.tap(KeyCode::Escape);
    assert_eq!(game.screen(), GameScreen::Playing, "Escape does not pause");
}

/// The build window opens from the bonus screen too, so the choice is made
/// knowing what the owned bonuses do: Tab opens it on the first, a click on an
/// owned bonus opens it on that one, and closing it goes back to the same
/// offers, which can still be chosen.
#[test]
fn the_build_window_opens_over_the_offers_and_closes_back_to_them() {
    let mut game = HeadlessGame::new();
    game.start_run();
    game.defeat_enemies();
    game.take_bonus();
    assert_eq!(game.screen(), GameScreen::Bonus);
    let offers = game.session().offered_bonuses.clone();
    let owned = game.session().owned_bonuses.clone();
    assert!(!owned.is_empty(), "room 0 paid out a bonus");

    game.tap(KeyCode::Tab);
    assert_eq!(game.screen(), GameScreen::Build);
    let focus = |game: &HeadlessGame| game.app.world().resource::<BuildFocus>().index;
    assert_eq!(focus(&game), 0);
    game.tap(KeyCode::Escape);
    assert_eq!(
        game.screen(),
        GameScreen::Bonus,
        "Escape goes back to the choice"
    );
    assert_eq!(
        game.session().offered_bonuses,
        offers,
        "the offers are untouched"
    );

    let chip = {
        let world = game.app.world_mut();
        world
            .query::<(Entity, &OwnedChip)>()
            .iter(world)
            .max_by_key(|(_, chip)| chip.0)
            .map(|(entity, chip)| (entity, chip.0))
    };
    let (entity, index) = chip.expect("the owned bonuses are listed under the offers");
    game.click_mouse_on(entity, Vec2::new(640.0, 600.0));
    assert_eq!(game.screen(), GameScreen::Build);
    assert_eq!(focus(&game), index, "the window opens on the bonus clicked");
    game.tap(KeyCode::Tab);
    assert_eq!(game.screen(), GameScreen::Bonus, "its key goes back too");

    game.tap(KeyCode::Digit1);
    game.wait_for_screen(GameScreen::Playing, 2.0);
    assert_eq!(game.session().owned_bonuses.last(), offers.first());
}

/// Lion's Share takes the whole choice: picking it takes the two offers
/// beside it too, and every box of them flares while the screen holds.
#[test]
fn lions_share_takes_every_offer_and_each_box_flares() {
    let mut game = HeadlessGame::new();
    game.start_run();
    game.defeat_enemies();
    game.take_bonus();
    let mut offers = game.session().offered_bonuses.clone();
    offers[1] = BonusPick::new("lions_share", BonusRarity::Legendary);
    game.app
        .world_mut()
        .resource_mut::<RunSession>()
        .offered_bonuses
        .clone_from(&offers);

    game.tap(KeyCode::Digit2);
    assert_eq!(
        game.screen(),
        GameScreen::Bonus,
        "the screen holds a moment"
    );
    assert_eq!(game.session().chosen_offers, [0, 1, 2]);
    game.run_seconds(game.data().gameplay.room_flow.bonus_chosen_hold_s / 2.0);
    let swollen = {
        let world = game.app.world_mut();
        world
            .query::<(&BonusChoice, &UiTransform)>()
            .iter(world)
            .filter(|(_, transform)| transform.scale.x > 1.0)
            .count()
    };
    assert_eq!(swollen, 3, "every box taken swells");
    game.wait_for_screen(GameScreen::Playing, 2.0);
    let owned = &game.session().owned_bonuses;
    assert!(
        offers.iter().all(|offer| owned.contains(offer)),
        "every offer is owned: {owned:?}"
    );
}
