use super::*;
use yaml_serde::Value;

pub(super) fn room_clear_check(
    enemies: Query<&EnemyBody>,
    data: Res<GameData>,
    mut session: ResMut<RunSession>,
    mut timer: ResMut<PhaseTimer>,
    mut next_screen: ResMut<NextState<GameScreen>>,
    mut sounds: ResMut<SoundQueue>,
) {
    if session.door_open || session.health <= 0.0 {
        return;
    }
    if enemies.iter().any(|enemy| enemy.health > 0.0) {
        return;
    }
    session.door_open = false;
    session.offered_bonuses.clear();
    timer.0 = Timer::from_seconds(data.gameplay.room_flow.clear_animation_s, TimerMode::Once);
    sounds.push(SoundCue::RoomClear);
    next_screen.set(GameScreen::RoomClear);
}

pub(super) fn update_room_clear(
    time: Res<Time>,
    data: Res<GameData>,
    mut random: ResMut<RandomState>,
    mut session: ResMut<RunSession>,
    mut timer: ResMut<PhaseTimer>,
    mut next_screen: ResMut<NextState<GameScreen>>,
) {
    timer.0.tick(time.delta());
    if !timer.0.is_finished() {
        return;
    }
    session.offered_bonuses = draw_bonus_pair(&data, &session, &mut random, &BTreeSet::new());
    session.previous_offers.clear();
    let offered = session.offered_bonuses.clone();
    session.previous_offers.extend(offered);
    session.selected_option = 0;
    next_screen.set(GameScreen::Bonus);
}

#[derive(SystemParam)]
pub(super) struct BonusInputResources<'w> {
    keys: Res<'w, ButtonInput<KeyCode>>,
    controls: Res<'w, ControlBindings>,
    data: Res<'w, GameData>,
    random: ResMut<'w, RandomState>,
    session: ResMut<'w, RunSession>,
    next_screen: ResMut<'w, NextState<GameScreen>>,
    sounds: ResMut<'w, SoundQueue>,
}

pub(super) fn handle_bonus_input(mut commands: Commands, resources: BonusInputResources) {
    let BonusInputResources {
        keys,
        controls,
        data,
        mut random,
        mut session,
        mut next_screen,
        mut sounds,
    } = resources;
    let previous_selection = session.selected_option;
    let can_reroll = reroll_available(&session, &data);
    let max_option = if can_reroll { 2 } else { 1 };
    if controls.just_pressed(GameAction::MoveLeft, &keys)
        || controls.just_pressed(GameAction::NavigateUp, &keys)
    {
        session.selected_option = session.selected_option.saturating_sub(1);
    } else if controls.just_pressed(GameAction::MoveRight, &keys)
        || controls.just_pressed(GameAction::NavigateDown, &keys)
    {
        session.selected_option = (session.selected_option + 1).min(max_option);
    }
    if session.selected_option != previous_selection {
        sounds.push(SoundCue::UiMove);
    }
    if !controls.just_pressed(GameAction::Interact, &keys) {
        return;
    }
    if session.selected_option == 2 {
        if !can_reroll {
            sounds.push(SoundCue::UiInvalid);
            return;
        }
        session.rerolls -= 1;
        sounds.push(SoundCue::BonusSelected);
        session.offered_bonuses =
            draw_bonus_pair(&data, &session, &mut random, &session.previous_offers);
        session.previous_offers.clear();
        let offered = session.offered_bonuses.clone();
        session.previous_offers.extend(offered);
        session.selected_option = 0;
        return;
    }

    let Some(chosen_id) = session
        .offered_bonuses
        .get(session.selected_option)
        .cloned()
    else {
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

    let room = &data.rooms[&session.room_id];
    let door_position = Vec3::new(
        room.exit.x_h * H_WORLD_UNITS,
        room.exit.y_h * H_WORLD_UNITS + H_WORLD_UNITS / 2.0,
        2.0,
    );
    commands.spawn((
        Sprite::from_color(Color::srgb(0.34, 0.77, 0.87), Vec2::new(28.0, 48.0)),
        Transform::from_translation(door_position),
        ExitDoor,
        RoomEntity,
    ));
    session.door_open = true;
    sounds.push(SoundCue::DoorOpen);
    next_screen.set(GameScreen::Playing);
}

pub(super) fn reroll_available(session: &RunSession, data: &GameData) -> bool {
    session.rerolls > 0
        && data
            .bonuses
            .bonuses
            .iter()
            .filter(|bonus| !session.owned_bonuses.contains(&bonus.id))
            .filter(|bonus| !session.previous_offers.contains(&bonus.id))
            .count()
            >= 2
}

pub(super) fn handle_door_touch(
    player: Query<&Transform, (With<PlayerBody>, Without<ExitDoor>)>,
    doors: Query<&Transform, (With<ExitDoor>, Without<PlayerBody>)>,
    mut timer: ResMut<PhaseTimer>,
    mut next_screen: ResMut<NextState<GameScreen>>,
) {
    let (Ok(player), Ok(door)) = (player.single(), doors.single()) else {
        return;
    };
    if (player.translation.x - door.translation.x).abs() > H_WORLD_UNITS * 0.65
        || (player.translation.y - door.translation.y).abs() > H_WORLD_UNITS
    {
        return;
    }
    timer.0 = Timer::from_seconds(0.45, TimerMode::Once);
    next_screen.set(GameScreen::Transitioning);
}

pub(super) fn update_room_transition(
    mut commands: Commands,
    time: Res<Time>,
    data: Res<GameData>,
    entities: Query<Entity, With<RoomEntity>>,
    mut session: ResMut<RunSession>,
    mut timer: ResMut<PhaseTimer>,
    mut next_screen: ResMut<NextState<GameScreen>>,
) {
    timer.0.tick(time.delta());
    if !timer.0.is_finished() {
        return;
    }
    let next_room_id = data.rooms[&session.room_id].next_room_id;
    for entity in &entities {
        commands.entity(entity).despawn();
    }
    session.world_loaded = false;
    session.door_open = false;
    session.room_ward_used = false;
    if !data.rooms.contains_key(&next_room_id) {
        next_screen.set(GameScreen::Complete);
        return;
    }
    session.room_id = next_room_id;
    next_screen.set(GameScreen::Playing);
}

fn draw_bonus_pair(
    data: &GameData,
    session: &RunSession,
    random: &mut RandomState,
    excluded: &BTreeSet<String>,
) -> Vec<String> {
    let mut eligible = data
        .bonuses
        .bonuses
        .iter()
        .filter(|bonus| !session.owned_bonuses.contains(&bonus.id))
        .filter(|bonus| !excluded.contains(&bonus.id))
        .map(|bonus| bonus.id.clone())
        .collect::<Vec<_>>();
    let mut offers = Vec::with_capacity(2);
    while offers.len() < 2 && !eligible.is_empty() {
        let index = random.index(eligible.len());
        offers.push(eligible.swap_remove(index));
    }
    offers
}
