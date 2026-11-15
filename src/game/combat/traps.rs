//! What a room's devices do to the player: every device that can hurt keeps
//! its dangerous area up to date in a `Hurts` (`devices/`), and this is where
//! standing in one while it is live costs health, through the same path as
//! any other hit (invulnerability, the roll, Cover and the bonuses included).

use super::*;

/// The part of a device that hurts right now: its corners in world units,
/// counter-clockwise, turned as the device is.
#[derive(Component, Default)]
pub(crate) struct Hurts {
    pub(crate) corners: [Vec2; 4],
    pub(crate) damage: f32,
    /// Whether it hurts at this moment: a laser between its beams, a fire
    /// trap at rest, a press up do not.
    pub(crate) live: bool,
}

impl Hurts {
    /// Whether it overlaps `area`: separated along none of the axes of the
    /// two shapes' sides.
    fn touches(&self, area: Rect) -> bool {
        let boxed = [
            area.min,
            Vec2::new(area.max.x, area.min.y),
            area.max,
            Vec2::new(area.min.x, area.max.y),
        ];
        let sides = (0..2).map(|index| self.corners[index + 1] - self.corners[index]);
        let axes = [Vec2::X, Vec2::Y]
            .into_iter()
            .chain(sides.map(|side| side.perp()));
        let spread = |corners: &[Vec2; 4], axis: Vec2| {
            corners
                .iter()
                .fold((f32::MAX, f32::MIN), |(low, high), corner| {
                    let along = corner.dot(axis);
                    (low.min(along), high.max(along))
                })
        };
        axes.into_iter().all(|axis| {
            let (low, high) = spread(&self.corners, axis);
            let (box_low, box_high) = spread(&boxed, axis);
            low < box_high && box_low < high
        })
    }

    /// Its middle, which way the hit comes from.
    fn middle(&self) -> Vec2 {
        self.corners.iter().sum::<Vec2>() / 4.0
    }
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
    // A won room's devices are powered down (`devices::run_cycles`).
    if session.fight_won(&data) {
        return;
    }
    let player_box = player_hurtbox(player_transform.translation, &data);
    for hurt in &hurts {
        if hurt.live && hurt.touches(player_box) {
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
                    attacker_x: hurt.middle().x,
                },
                hurt.damage,
            );
        }
    }
}
