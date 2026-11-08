//! Bodies that fall out of the room. A room's lower edge is the end of the
//! world: a gap with no platform under it is bottomless, and a body that drops
//! entirely below the edge is gone. The run ends there for the player, and an
//! enemy that misses a leap dies instead of falling forever.

use super::*;

/// Whether a body whose feet are at `feet_y` has fallen out of the room: it is
/// out once its head passes the room's lower edge, so it is no longer in sight.
fn fell_out_of_the_room(feet_y: f32) -> bool {
    feet_y + H_WORLD_UNITS < 0.0
}

pub(in crate::game) fn update_falls(
    mut commands: Commands,
    mut session: ResMut<RunSession>,
    mut sounds: ResMut<SoundQueue>,
    mut next_screen: ResMut<NextState<GameScreen>>,
    players: Query<&Transform, With<PlayerBody>>,
    enemies: Query<(Entity, &EnemyBody)>,
) {
    for (entity, enemy) in &enemies {
        if fell_out_of_the_room(enemy.home_y - enemy.feet_offset) {
            // Gone counts as down, so a room still clears when an enemy pits
            // itself: the fall is the kill, with no sound and no body left.
            commands.entity(entity).despawn();
        }
    }
    let Ok(transform) = players.single() else {
        return;
    };
    if session.health > 0.0 && fell_out_of_the_room(transform.translation.y - H_WORLD_UNITS / 2.0) {
        // No death animation: it would play where nobody can see it.
        session.health = 0.0;
        sounds.push(SoundCue::RunEnded);
        next_screen.set(GameScreen::Dead);
    }
}
