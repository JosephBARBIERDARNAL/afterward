//! The later waves of a room's enemies. `wave` on a room spawn says which one
//! it arrives with: wave 0 stands there as the room opens, and each wave
//! after it rises out of the floor once everyone before it is down. In the
//! test room the first wave comes back after the last.

use super::*;

/// An enemy of a later wave rising out of the floor. It is held where it
/// stands until the timer runs out, by the stance `world::spawn_enemies` gave
/// it.
#[derive(Component)]
pub(crate) struct Arriving(pub(crate) Timer);

/// Sends in the room's next wave a breath after the last enemy standing went
/// down. `flow::room_clear_check` leaves the room uncleared while a wave is
/// still to come.
pub(in crate::game) fn send_in_the_next_wave(
    mut commands: Commands,
    time: Res<Time>,
    data: Res<GameData>,
    sheets: Res<SpriteSheets>,
    mut session: ResMut<RunSession>,
    mut sounds: ResMut<SoundQueue>,
    enemies: Query<&EnemyBody>,
) {
    let room = &data.rooms[&session.room_id];
    let waves = room.enemy_spawns.iter().map(|spawn| spawn.wave);
    let later = waves.clone().filter(|wave| *wave > session.wave).min();
    // The test room is never cleared: its last wave down, the first comes
    // back, as into a fresh room.
    let starts_over = later.is_none() && session.in_the_test_room();
    let next = later.or_else(|| waves.min().filter(|_| starts_over));
    let Some(next) = next.filter(|_| {
        session.has_enemies(&data)
            && session.health > 0.0
            && enemies.iter().all(|enemy| enemy.health <= 0.0)
    }) else {
        session.wave_wait_s = 0.0;
        return;
    };
    session.wave_wait_s += time.delta_secs();
    if session.wave_wait_s < data.gameplay.room_flow.waves.delay_s {
        return;
    }
    session.wave_wait_s = 0.0;
    session.wave = next;
    if starts_over {
        session.health = session.max_health;
        session.untouched = true;
        session.room_ward_used = false;
        session.deflector_used = false;
    }
    world::spawn_enemies(&mut commands, room, &data, &sheets, Some(next));
    sounds.push(SoundCue::EnemiesAppear);
}

/// Raises the enemies of a wave that has just arrived.
pub(in crate::game) fn raise_the_arriving(
    mut commands: Commands,
    time: Res<Time>,
    mut arriving: Query<(
        Entity,
        &EnemyBody,
        &room_intro::EnemyEntrance,
        &mut Arriving,
        &mut Transform,
        &mut Sprite,
    )>,
) {
    for (entity, enemy, entrance, mut arrival, mut transform, mut sprite) in &mut arriving {
        arrival.0.tick(time.delta());
        entrance.pose(arrival.0.fraction(), enemy, &mut transform, &mut sprite);
        if arrival.0.is_finished() {
            commands.entity(entity).remove::<Arriving>();
        }
    }
}
