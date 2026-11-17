use super::*;
use crate::game::{
    bonus_screen::BonusChoice,
    touch::{Spot, Touch},
};

pub(super) fn room_clear_check(
    enemies: Query<&EnemyBody>,
    data: Res<GameData>,
    mut session: ResMut<RunSession>,
    mut timer: ResMut<PhaseTimer>,
    mut next_screen: ResMut<NextState<GameScreen>>,
    mut sounds: ResMut<SoundQueue>,
    mut players: Query<&mut PlayerBody>,
) {
    // The test room is never cleared: `combat::send_in_the_next_wave` starts
    // its waves over.
    if session.room_cleared || session.health <= 0.0 || session.in_the_test_room() {
        return;
    }
    if enemies.iter().any(|enemy| enemy.health > 0.0) {
        return;
    }
    // A wave still to come keeps the room open: `combat::send_in_the_next_wave`
    // brings it in.
    if session.has_enemies(&data)
        && data.rooms[&session.room_id]
            .enemy_spawns
            .iter()
            .any(|spawn| spawn.wave > session.wave)
    {
        return;
    }
    session.room_cleared = true;
    // A fight won without a wound is paid for: see `draw_bonus_offers`.
    session.flawless = session.untouched && session.has_enemies(&data);
    session.offered_bonuses.clear();
    // A cleared room gives the player all their health back, under the
    // healing glow when there was any to give.
    if session.health < session.max_health {
        session.health = session.max_health;
        if let Ok(mut player) = players.single_mut() {
            player.start_action(
                PlayerAction::Healing,
                data.player_animations.clips.healing.duration_s(),
            );
        }
    }
    // A room authored without enemies (the opening room 0) has nothing to
    // clear: its bonus item is simply there from the start, with no flash.
    if !session.has_enemies(&data) {
        return;
    }
    timer.0 = Timer::from_seconds(data.gameplay.room_flow.clear_animation_s, TimerMode::Once);
    sounds.push(SoundCue::RoomClear);
    next_screen.set(GameScreen::RoomClear);
}

/// Holds the room-clear flash, then hands the room back so the player can walk
/// to the bonus item that `bonus_pickup::spawn_bonus_pickup` lays out.
pub(super) fn update_room_clear(
    time: Res<Time>,
    mut timer: ResMut<PhaseTimer>,
    mut next_screen: ResMut<NextState<GameScreen>>,
) {
    timer.0.tick(time.delta());
    if timer.0.is_finished() {
        next_screen.set(GameScreen::Playing);
    }
}

#[derive(SystemParam)]
pub(super) struct BonusInputResources<'w> {
    time: Res<'w, Time>,
    keys: Res<'w, ButtonInput<KeyCode>>,
    controls: Res<'w, ControlBindings>,
    data: Res<'w, GameData>,
    random: ResMut<'w, RandomState>,
    session: ResMut<'w, RunSession>,
    next_screen: ResMut<'w, NextState<GameScreen>>,
    sounds: ResMut<'w, SoundQueue>,
}

pub(super) fn handle_bonus_input(
    resources: BonusInputResources,
    mut doors: Query<&mut Sprite, With<ExitDoor>>,
    mut players: Query<&mut PlayerBody>,
    choices: Query<(Entity, &BonusChoice)>,
    mut pointer_overs: MessageReader<PointerOver>,
    mut pointer_clicks: MessageReader<PointerClick>,
    mut touches: MessageWriter<Touch>,
) {
    let BonusInputResources {
        time,
        keys,
        controls,
        data,
        mut random,
        mut session,
        mut next_screen,
        mut sounds,
    } = resources;
    let hovered = pointer_overs
        .read()
        .filter_map(|over| {
            choices
                .get(over.entity)
                .ok()
                .map(|(entity, choice)| (entity, *choice, over.pointer.position))
        })
        .last();
    let clicked = pointer_clicks
        .read()
        .filter(|click| click.button == PointerButton::Primary)
        .filter_map(|click| {
            choices
                .get(click.entity)
                .ok()
                .map(|(entity, choice)| (entity, *choice, click.pointer.position))
        })
        .last();
    // A bonus has been chosen and the screen is only still up for the spark
    // to land on the box that was pressed; nothing else is listened to.
    if session.bonus_chosen_hold_remaining_s > 0.0 {
        session.bonus_chosen_hold_remaining_s =
            (session.bonus_chosen_hold_remaining_s - time.delta_secs().min(0.05)).max(0.0);
        if session.bonus_chosen_hold_remaining_s <= 0.0 {
            next_screen.set(GameScreen::Playing);
        }
        return;
    }
    let was_animating = session.bonus_reroll_animation_remaining_s > 0.0;
    session.bonus_reroll_animation_remaining_s =
        (session.bonus_reroll_animation_remaining_s - time.delta_secs().min(0.05)).max(0.0);
    if was_animating && session.bonus_reroll_animation_remaining_s > 0.0 {
        return;
    }
    if was_animating {
        session.selected_choice = BonusChoice::Offer(0);
    }

    let can_reroll = reroll_available(&session, &data);
    let previous_choice = session.selected_choice;
    if let Some((_, choice, _)) =
        hovered.filter(|(_, choice, _)| *choice != BonusChoice::Reroll || can_reroll)
    {
        session.selected_choice = choice;
    } else if controls.just_pressed(GameAction::NavigateUp, &keys) {
        session.selected_choice = previous_choice.previous();
    } else if controls.just_pressed(GameAction::NavigateDown, &keys) {
        session.selected_choice = previous_choice.next(can_reroll);
    }
    if session.selected_choice != previous_choice {
        sounds.push(SoundCue::UiMove);
    }
    // The brackets snap onto the box the focus reached.
    if session.selected_choice != previous_choice {
        touches.write_batch(
            choices
                .iter()
                .find(|(_, choice)| **choice == session.selected_choice)
                .map(|(entity, _)| Touch::Lock(entity)),
        );
    }

    // Pressing a box throws a spark on it: where the click landed, or in its
    // middle for a key.
    let mut strike = |choice: BonusChoice| {
        let spot = clicked.map(|(_, _, at)| Spot::Pointer(at)).or_else(|| {
            choices
                .iter()
                .find(|(_, pressed)| **pressed == choice)
                .map(|(entity, _)| Spot::Middle(entity))
        });
        touches.write_batch(spot.map(Touch::Strike));
    };

    let chosen = if let Some((_, choice, _)) = clicked {
        choice
    } else if controls.just_pressed(GameAction::RerollBonus, &keys) {
        BonusChoice::Reroll
    } else if let Some(index) = BonusChoice::KEYS
        .iter()
        .position(|action| controls.just_pressed(*action, &keys))
    {
        BonusChoice::Offer(index)
    } else if controls.just_pressed(GameAction::Interact, &keys)
        || controls.just_pressed(GameAction::Confirm, &keys)
    {
        session.selected_choice
    } else {
        return;
    };
    let BonusChoice::Offer(chosen_index) = chosen else {
        if !can_reroll {
            sounds.push(SoundCue::UiInvalid);
            return;
        }
        session.rerolls -= 1;
        strike(BonusChoice::Reroll);
        sounds.push(SoundCue::BonusRerolled);
        session.offered_bonuses =
            draw_bonus_offers(&data, &session, &mut random, &session.previous_offers);
        session.previous_offers = session
            .offered_bonuses
            .iter()
            .map(|offer| offer.id.clone())
            .collect();
        session.bonus_reroll_animation_remaining_s =
            data.gameplay.room_flow.bonus_reroll_animation_s;
        return;
    };
    let Some(chosen_pick) = session.offered_bonuses.get(chosen_index).cloned() else {
        return;
    };
    // Lion's Share takes the whole choice: the other offers come with it.
    let takes_every_offer = data
        .bonuses
        .bonus(&chosen_pick.id)
        .is_some_and(|bonus| bonus.has_effect("takes_every_offer"));
    session.chosen_offers = if takes_every_offer {
        (0..session.offered_bonuses.len()).collect()
    } else {
        vec![chosen_index]
    };
    session.take(&data, chosen_pick);
    if takes_every_offer {
        for (index, pick) in session.offered_bonuses.clone().into_iter().enumerate() {
            if index != chosen_index {
                session.take(&data, pick);
            }
        }
    }
    strike(chosen);
    sounds.push(SoundCue::BonusSelected);
    // The build is worn for a moment: the player comes back into the room
    // wrapped in the glow of what they just took.
    if let Ok(mut player) = players.single_mut() {
        player.start_action(
            PlayerAction::Aura,
            data.player_animations.clips.aura.duration_s(),
        );
    }

    for mut sprite in &mut doors {
        if let Some(atlas) = sprite.texture_atlas.as_mut() {
            atlas.index = ExitDoor::UNLOCKED_FRAME;
        }
    }
    session.door_open = true;
    sounds.push(SoundCue::DoorAppear);
    session.bonus_chosen_hold_remaining_s = data.gameplay.room_flow.bonus_chosen_hold_s;
    if session.bonus_chosen_hold_remaining_s <= 0.0 {
        next_screen.set(GameScreen::Playing);
    }
}

pub(super) fn reroll_available(session: &RunSession, data: &GameData) -> bool {
    session.rerolls > 0
        && eligible_bonuses(data, session, &session.previous_offers)
            .map(BonusConfig::kind)
            .collect::<BTreeSet<_>>()
            .len()
            >= 2
}

/// The bonuses the player may still be offered: not already owned, and not
/// in `excluded`.
fn eligible_bonuses<'a>(
    data: &'a GameData,
    session: &'a RunSession,
    excluded: &'a BTreeSet<String>,
) -> impl Iterator<Item = &'a BonusConfig> {
    data.bonuses
        .bonuses
        .iter()
        .filter(|bonus| session.may_be_offered(bonus))
        .filter(|bonus| !excluded.contains(&bonus.id))
}

/// What using the door reads and writes.
#[derive(SystemParam)]
pub(super) struct DoorTouchResources<'w> {
    session: Res<'w, RunSession>,
    data: Res<'w, GameData>,
    sheets: Res<'w, SpriteSheets>,
    keys: Res<'w, ButtonInput<KeyCode>>,
    controls: Res<'w, ControlBindings>,
    in_reach: ResMut<'w, InReach>,
    timer: ResMut<'w, PhaseTimer>,
    next_screen: ResMut<'w, NextState<GameScreen>>,
    sounds: ResMut<'w, SoundQueue>,
}

/// Leaves the room when Interact is pressed within reach of its unlocked
/// door, turning the portal over it.
pub(super) fn handle_door_touch(
    mut commands: Commands,
    resources: DoorTouchResources,
    doors: Query<Entity, With<ExitDoor>>,
) {
    let DoorTouchResources {
        session,
        data,
        sheets,
        keys,
        controls,
        mut in_reach,
        mut timer,
        mut next_screen,
        mut sounds,
    } = resources;
    // The door stays locked until a bonus is picked.
    if !session.door_open {
        return;
    }
    let Ok(door_entity) = doors.single() else {
        return;
    };
    if in_reach.0 != Some(door_entity) || !controls.just_pressed(GameAction::Interact, &keys) {
        return;
    }
    in_reach.0 = None;
    timer.0 = Timer::from_seconds(
        data.gameplay.room_flow.room_transition_duration_s / 2.0,
        TimerMode::Once,
    );
    // The way out turns open as it is used. This only runs while the room
    // is being played, which the same frame leaves, so one portal is hung and
    // it rides the door off with the room.
    room_pan::hang_a_portal(&mut commands, door_entity, &sheets);
    sounds.push(SoundCue::DoorOpen);
    next_screen.set(GameScreen::Transitioning);
}

pub(super) fn start_boss_intro(data: Res<GameData>, mut timer: ResMut<PhaseTimer>) {
    timer.0 = Timer::from_seconds(
        data.gameplay.room_flow.boss_intro_duration_s,
        TimerMode::Once,
    );
}

pub(super) fn update_boss_intro(
    time: Res<Time>,
    mut timer: ResMut<PhaseTimer>,
    mut next_screen: ResMut<NextState<GameScreen>>,
) {
    timer.0.tick(time.delta());
    if timer.0.is_finished() {
        next_screen.set(GameScreen::Playing);
    }
}

pub(super) fn update_room_transition(
    time: Res<Time>,
    data: Res<GameData>,
    mut pan: ResMut<RoomPan>,
    mut session: ResMut<RunSession>,
    mut timer: ResMut<PhaseTimer>,
    mut next_screen: ResMut<NextState<GameScreen>>,
) {
    timer.0.tick(time.delta());
    if !timer.0.is_finished() {
        return;
    }
    let next_room_id = session.next_room_id();
    if !data.rooms.contains_key(&next_room_id) {
        timer.0 = Timer::from_seconds(
            data.gameplay.room_flow.room_transition_duration_s / 2.0,
            TimerMode::Once,
        );
        next_screen.set(GameScreen::TransitioningOut);
        return;
    }
    // The room stays on screen: `room_pan` stands it to the left of the next
    // one and lets it go once the camera has crossed.
    pan.departing_width_h = data.rooms[&session.room_id].size.width_h;
    // The door out of the opening room is the way into the city, and the city
    // is spoken for before it is entered.
    let opening_room = session.room_id == 0;
    session.world_loaded = false;
    session.door_open = false;
    session.room_cleared = false;
    session.room_ward_used = false;
    session.room_id = next_room_id;
    next_screen.set(if opening_room {
        GameScreen::Prologue
    } else {
        GameScreen::RoomLoading
    });
}

/// Plays the door's opening frames over the first half of the transition.
pub(super) fn animate_exit_door(
    timer: Res<PhaseTimer>,
    data: Res<GameData>,
    mut doors: Query<&mut Sprite, With<ExitDoor>>,
) {
    let frames = data.gameplay.room_flow.exit_door.frames;
    let opening_frames = frames - ExitDoor::FIRST_OPENING_FRAME;
    let step = (timer.0.fraction() * opening_frames as f32) as usize;
    for mut sprite in &mut doors {
        if let Some(atlas) = sprite.texture_atlas.as_mut() {
            atlas.index = (ExitDoor::FIRST_OPENING_FRAME + step).min(frames - 1);
        }
    }
}

pub(super) fn update_room_transition_fade_out(
    time: Res<Time>,
    session: Res<RunSession>,
    mut timer: ResMut<PhaseTimer>,
    mut next_screen: ResMut<NextState<GameScreen>>,
) {
    timer.0.tick(time.delta());
    if !timer.0.is_finished() {
        return;
    }
    next_screen.set(if session.door_open {
        GameScreen::Complete
    } else {
        GameScreen::RoomIntro
    });
}

/// Draws the bonus screen's offers: three different bonuses, each at one of
/// its rarities. Every version of every eligible bonus is a ticket weighing
/// its rarity's weight, so commons come up often and legendaries rarely; a
/// room cleared untouched weighs every version above common heavier still,
/// and a bonus that works with the build owned weighs heavier whatever its
/// rarity.
pub(super) fn draw_bonus_offers(
    data: &GameData,
    session: &RunSession,
    random: &mut RandomState,
    excluded: &BTreeSet<String>,
) -> Vec<BonusPick> {
    let untouched = data.gameplay.room_flow.untouched.rarer_weight_multiplier;
    let mut tickets = eligible_bonuses(data, session, excluded)
        .flat_map(|bonus| {
            let synergy = if session.synergy(data, bonus).is_some() {
                data.bonuses.synergy.weight_multiplier
            } else {
                1
            };
            bonus.rarities.keys().map(move |rarity| {
                let lean = if session.flawless && *rarity != BonusRarity::Common {
                    untouched
                } else {
                    1
                };
                let weight = data.bonuses.rarities[*rarity].weight * lean * synergy;
                ((BonusPick::new(&bonus.id, *rarity), bonus.kind()), weight)
            })
        })
        .collect::<Vec<_>>();
    let mut offers = Vec::with_capacity(BonusChoice::KEYS.len());
    while offers.len() < BonusChoice::KEYS.len() && !tickets.is_empty() {
        let total = tickets.iter().map(|(_, weight)| weight).sum::<u32>() as usize;
        let mut drawn = random.index(total.max(1)) as u32;
        let index = tickets
            .iter()
            .position(|(_, weight)| {
                let landed = drawn < *weight;
                drawn -= if landed { 0 } else { *weight };
                landed
            })
            .unwrap_or(0);
        let ((pick, kind), _) = tickets.swap_remove(index);
        // A bonus is offered once, at the one rarity drawn, and never beside
        // another of its family.
        tickets.retain(|((_, other), _)| *other != kind);
        offers.push(pick);
    }
    offers
}

#[derive(Resource)]
pub(crate) struct PhaseTimer(pub(crate) Timer);

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn the_draw_offers_three_different_bonuses_and_leans_on_the_common_ones() {
        let data = GameData::load(&Path::new(env!("CARGO_MANIFEST_DIR")).join("src/assets"))
            .expect("the game data should be valid");
        let session = RunSession::default();
        let mut random = RandomState(12345);
        let mut drawn = BTreeMap::<BonusRarity, usize>::new();
        for _ in 0..500 {
            let offers = draw_bonus_offers(&data, &session, &mut random, &BTreeSet::new());
            assert_eq!(offers.len(), 3);
            assert_eq!(
                offers
                    .iter()
                    .map(|offer| &offer.id)
                    .collect::<BTreeSet<_>>()
                    .len(),
                3,
                "a draw never offers the same bonus twice"
            );
            assert_eq!(
                offers
                    .iter()
                    .filter_map(|offer| data.bonuses.bonus(&offer.id))
                    .map(BonusConfig::kind)
                    .collect::<BTreeSet<_>>()
                    .len(),
                3,
                "a draw never offers two bonuses of one family"
            );
            for offer in offers {
                assert!(
                    data.bonuses
                        .bonus(&offer.id)
                        .is_some_and(|bonus| bonus.rarities.contains_key(&offer.rarity)),
                    "a bonus is offered at one of its own rarities"
                );
                *drawn.entry(offer.rarity).or_default() += 1;
            }
        }
        let count = |rarity: BonusRarity| drawn.get(&rarity).copied().unwrap_or(0);
        assert!(
            count(BonusRarity::Common) > count(BonusRarity::Rare),
            "commons come up more often than rares: {drawn:?}"
        );
        assert!(
            count(BonusRarity::Rare) > count(BonusRarity::Epic),
            "rares come up more often than epics: {drawn:?}"
        );
        assert!(
            count(BonusRarity::Epic) > count(BonusRarity::Legendary),
            "epics come up more often than legendaries: {drawn:?}"
        );
    }
}
