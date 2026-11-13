use super::*;
use crate::game::{
    bonus_screen::BonusChoice,
    ripple::{Splash, Spot},
};
use yaml_serde::Value;

pub(super) fn room_clear_check(
    enemies: Query<&EnemyBody>,
    data: Res<GameData>,
    mut session: ResMut<RunSession>,
    mut timer: ResMut<PhaseTimer>,
    mut next_screen: ResMut<NextState<GameScreen>>,
    mut sounds: ResMut<SoundQueue>,
    mut players: Query<&mut PlayerBody>,
) {
    if session.room_cleared || session.health <= 0.0 {
        return;
    }
    if enemies.iter().any(|enemy| enemy.health > 0.0) {
        return;
    }
    session.room_cleared = true;
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
    mut splashes: MessageWriter<Splash>,
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
                .map(|(_, choice)| (*choice, over.pointer.position))
        })
        .last();
    let clicked = pointer_clicks
        .read()
        .filter(|click| click.button == PointerButton::Primary)
        .filter_map(|click| {
            choices
                .get(click.entity)
                .ok()
                .map(|(_, choice)| (*choice, click.pointer.position))
        })
        .last();
    // A bonus has been chosen and the screen is only still up for the droplet
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
    if let Some((choice, _)) =
        hovered.filter(|(choice, _)| *choice != BonusChoice::Reroll || can_reroll)
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
    // A droplet falls on the box touched: where the mouse landed on it, or in
    // its middle when an arrow walked there.
    let moved = session.selected_choice != previous_choice;
    if let Some((_, at)) = clicked.or_else(|| hovered.filter(|_| moved)) {
        splashes.write(Splash(Spot::Pointer(at)));
    } else if moved {
        splashes.write_batch(
            choices
                .iter()
                .find(|(_, choice)| **choice == session.selected_choice)
                .map(|(entity, _)| Splash(Spot::Middle(entity))),
        );
    }

    // Pressing a box drops a droplet on it too: a click already dropped one
    // exactly where it landed, so this is the one a key press owes the box.
    let mut droplet_on = |choice: BonusChoice| {
        if clicked.is_none() {
            splashes.write_batch(
                choices
                    .iter()
                    .find(|(_, pressed)| **pressed == choice)
                    .map(|(entity, _)| Splash(Spot::Middle(entity))),
            );
        }
    };

    let chosen = if let Some((choice, _)) = clicked {
        choice
    } else if controls.just_pressed(GameAction::RerollBonus, &keys) {
        BonusChoice::Reroll
    } else if controls.just_pressed(GameAction::ChooseFirstBonus, &keys) {
        BonusChoice::Offer(0)
    } else if controls.just_pressed(GameAction::ChooseSecondBonus, &keys) {
        BonusChoice::Offer(1)
    } else if controls.just_pressed(GameAction::Interact, &keys) {
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
        droplet_on(BonusChoice::Reroll);
        sounds.push(SoundCue::BonusRerolled);
        session.offered_bonuses =
            draw_bonus_pair(&data, &session, &mut random, &session.previous_offers);
        session.previous_offers.clear();
        let offered = session.offered_bonuses.clone();
        session.previous_offers.extend(offered);
        session.bonus_reroll_animation_remaining_s =
            data.gameplay.room_flow.bonus_reroll_animation_s;
        return;
    };
    let Some(chosen_id) = session.offered_bonuses.get(chosen_index).cloned() else {
        return;
    };
    session.owned_bonuses.push(chosen_id.clone());
    droplet_on(chosen);
    sounds.push(SoundCue::BonusSelected);
    // The build is worn for a moment: the player comes back into the room
    // wrapped in the glow of the category they just took from.
    if let Ok(mut player) = players.single_mut()
        && let Some(aura) = session
            .bonus_category
            .as_deref()
            .and_then(|category| category.parse::<BonusAura>().ok())
    {
        let duration_s = aura.clip(&data.player_animations.clips).duration_s();
        player.start_action(PlayerAction::Aura(aura), duration_s);
    }
    if let Some(bonus) = data
        .bonuses
        .bonuses
        .iter()
        .find(|bonus| bonus.id == chosen_id)
    {
        if let Some(increase) = bonus
            .effects
            .get("max_health_increase")
            .and_then(Value::as_f64)
        {
            session.max_health += increase as f32;
        }
        if let Some(restore) = bonus.effects.get("restore_health").and_then(Value::as_f64) {
            session.health = (session.health + restore as f32).min(session.max_health);
        }
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
    session.rerolls > 0 && eligible_bonuses(data, session, &session.previous_offers).count() >= 2
}

/// The bonuses of the picked-up item's category the player may still be
/// offered: not already owned, and not in `excluded`.
fn eligible_bonuses<'a>(
    data: &'a GameData,
    session: &'a RunSession,
    excluded: &'a BTreeSet<String>,
) -> impl Iterator<Item = &'a crate::data::BonusConfig> {
    data.bonuses
        .bonuses
        .iter()
        .filter(|bonus| session.bonus_category.as_ref() == Some(&bonus.category))
        .filter(|bonus| !session.owned_bonuses.contains(&bonus.id))
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
    let next_room_id = data.rooms[&session.room_id].next_room_id;
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
    session.bonus_category = None;
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

pub(super) fn draw_bonus_pair(
    data: &GameData,
    session: &RunSession,
    random: &mut RandomState,
    excluded: &BTreeSet<String>,
) -> Vec<String> {
    let mut eligible = eligible_bonuses(data, session, excluded).collect::<Vec<_>>();
    let mut offers = Vec::with_capacity(2);
    // A rarer bonus weighs less in the draw, so commons come up often and
    // legendaries rarely.
    let weight = |bonus: &crate::data::BonusConfig| data.bonuses.rarities[bonus.rarity].weight;
    while offers.len() < 2 && !eligible.is_empty() {
        let total = eligible.iter().copied().map(weight).sum::<u32>() as usize;
        let mut drawn = random.index(total.max(1)) as u32;
        let index = eligible
            .iter()
            .copied()
            .position(|bonus| {
                let weight = weight(bonus);
                let landed = drawn < weight;
                drawn -= if landed { 0 } else { weight };
                landed
            })
            .unwrap_or(0);
        offers.push(eligible.swap_remove(index).id.clone());
    }
    offers
}

#[derive(Resource)]
pub(crate) struct PhaseTimer(pub(crate) Timer);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::BonusRarity;
    use std::path::Path;

    #[test]
    fn the_draw_offers_two_different_bonuses_and_leans_on_the_common_ones() {
        let data = GameData::load(&Path::new(env!("CARGO_MANIFEST_DIR")).join("src/assets"))
            .expect("the game data should be valid");
        let session = RunSession {
            bonus_category: Some("combat".to_owned()),
            ..default()
        };
        let mut random = RandomState(12345);
        let mut drawn = BTreeMap::<String, usize>::new();
        for _ in 0..500 {
            let pair = draw_bonus_pair(&data, &session, &mut random, &BTreeSet::new());
            assert_eq!(pair.len(), 2);
            assert_ne!(pair[0], pair[1], "a pair never offers the same bonus twice");
            for id in pair {
                let rarity = data
                    .bonuses
                    .bonuses
                    .iter()
                    .find(|bonus| bonus.id == id)
                    .expect("an offered bonus exists")
                    .rarity;
                *drawn.entry(rarity.to_string()).or_default() += 1;
            }
        }
        let count = |rarity: BonusRarity| drawn.get(&rarity.to_string()).copied().unwrap_or(0);
        assert!(
            count(BonusRarity::Common) > count(BonusRarity::Rare),
            "commons come up more often than rares: {drawn:?}"
        );
        assert!(
            count(BonusRarity::Rare) > count(BonusRarity::Epic),
            "rares come up more often than epics: {drawn:?}"
        );
    }
}
