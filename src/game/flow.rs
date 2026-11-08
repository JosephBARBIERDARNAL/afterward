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
) {
    if session.room_cleared || session.health <= 0.0 {
        return;
    }
    if enemies.iter().any(|enemy| enemy.health > 0.0) {
        return;
    }
    session.room_cleared = true;
    session.offered_bonuses.clear();
    // A room authored without enemies (the opening room 0) has nothing to
    // clear: its bonus item is simply there from the start, with no flash.
    if data.rooms[&session.room_id].enemy_spawns.is_empty() {
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
    sounds.push(SoundCue::BonusSelected);
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
    next_screen.set(GameScreen::Playing);
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

pub(super) fn handle_door_touch(
    player: Query<&Transform, (With<PlayerBody>, Without<ExitDoor>)>,
    doors: Query<&Transform, (With<ExitDoor>, Without<PlayerBody>)>,
    session: Res<RunSession>,
    data: Res<GameData>,
    mut timer: ResMut<PhaseTimer>,
    mut next_screen: ResMut<NextState<GameScreen>>,
    mut sounds: ResMut<SoundQueue>,
) {
    // The door stays locked until a bonus is picked.
    if !session.door_open {
        return;
    }
    let (Ok(player), Ok(door)) = (player.single(), doors.single()) else {
        return;
    };
    if (player.translation.x - door.translation.x).abs() > H_WORLD_UNITS * 0.65
        || (player.translation.y - door.translation.y).abs() > H_WORLD_UNITS
    {
        return;
    }
    timer.0 = Timer::from_seconds(
        data.gameplay.room_flow.room_transition_duration_s / 2.0,
        TimerMode::Once,
    );
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
    session.world_loaded = false;
    session.door_open = false;
    session.room_cleared = false;
    session.bonus_category = None;
    session.room_ward_used = false;
    session.room_id = next_room_id;
    next_screen.set(GameScreen::RoomLoading);
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
    let mut eligible = eligible_bonuses(data, session, excluded)
        .map(|bonus| bonus.id.clone())
        .collect::<Vec<_>>();
    let mut offers = Vec::with_capacity(2);
    while offers.len() < 2 && !eligible.is_empty() {
        let index = random.index(eligible.len());
        offers.push(eligible.swap_remove(index));
    }
    offers
}

#[derive(Resource)]
pub(crate) struct PhaseTimer(pub(crate) Timer);
