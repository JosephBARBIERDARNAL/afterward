//! What a room's devices do to the player: every device that can hurt keeps
//! its dangerous area up to date in a `Hurts` (`devices/`), and this is where
//! standing in one while it is live costs health, through the same path as
//! any other hit (invulnerability, the roll, Cover and the bonuses included).

use super::*;

/// The part of a device that hurts right now, in world units.
#[derive(Component, Default)]
pub(crate) struct Hurts {
    pub(crate) area: Rect,
    pub(crate) damage: f32,
    /// Whether it hurts at this moment: a laser between its beams, a fire
    /// trap at rest, a press up do not.
    pub(crate) live: bool,
}

pub(in crate::game) fn hurt_by_devices(
    mut commands: Commands,
    (data, mut session, mut sounds): (Res<GameData>, ResMut<RunSession>, ResMut<SoundQueue>),
    mut players: Query<(&Transform, &mut PlayerBody)>,
    hurts: Query<&Hurts>,
) {
    let Ok((player_transform, mut player)) = players.single_mut() else {
        return;
    };
    let player_box = player_hurtbox(player_transform.translation, &data);
    for hurt in &hurts {
        if hurt.live && !hurt.area.intersect(player_box).is_empty() {
            apply_player_damage(
                PlayerDamageResources {
                    commands: &mut commands,
                    data: &data,
                    session: &mut session,
                    sounds: &mut sounds,
                },
                PlayerDamageTarget {
                    player: &mut player,
                    transform: player_transform,
                    attacker_x: hurt.area.center().x,
                },
                hurt.damage,
            );
        }
    }
}
