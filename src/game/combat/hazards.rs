//! Patches of the room that burn: the one a `hazard` attack lays under its
//! mark, the blast a type with a `death_blast` leaves where it dies, and the
//! vents a room lists under `hazards`. All three warn before they hurt, in
//! the warm colours kept for danger (`effects.hazard` of
//! `config/gameplay.yaml`).

use super::*;

/// A patch that warns for `arm_s`, then burns for `burn_s` and hurts the
/// player standing in it. One with a `rest_s` is a vent, which rests and
/// starts over; one without goes once it has burned.
#[derive(Component)]
pub(crate) struct Hazard {
    pub(crate) area: Rect,
    pub(crate) damage: f32,
    pub(crate) arm_s: f32,
    pub(crate) burn_s: f32,
    pub(crate) rest_s: Option<f32>,
    pub(crate) elapsed_s: f32,
}

impl Hazard {
    /// Lays the patch in the room, drawn as its warning.
    pub(crate) fn lay(self, commands: &mut Commands, data: &GameData) {
        commands.spawn((
            Sprite::from_color(
                Color::from(data.gameplay.effects.hazard.warning_color).with_alpha(0.0),
                self.area.size(),
            ),
            Transform::from_translation(self.area.center().extend(3.5)),
            self,
            RoomEntity,
        ));
    }
}

#[derive(SystemParam)]
pub(in crate::game) struct HazardResources<'w> {
    time: Res<'w, Time>,
    data: Res<'w, GameData>,
    session: ResMut<'w, RunSession>,
    sounds: ResMut<'w, SoundQueue>,
}

pub(in crate::game) fn update_hazards(
    mut commands: Commands,
    resources: HazardResources,
    mut players: Query<(&Transform, &mut PlayerBody), Without<Hazard>>,
    mut hazards: Query<(Entity, &mut Hazard, &mut Sprite)>,
) {
    let HazardResources {
        time,
        data,
        mut session,
        mut sounds,
    } = resources;
    let Ok((player_transform, mut player)) = players.single_mut() else {
        return;
    };
    let look = &data.gameplay.effects.hazard;
    let player_box = player_hurtbox(player_transform.translation, &data);
    for (entity, mut hazard, mut sprite) in &mut hazards {
        hazard.elapsed_s += time.delta_secs().min(0.05);
        let cycle_s = hazard.arm_s + hazard.burn_s + hazard.rest_s.unwrap_or(0.0);
        if hazard.rest_s.is_none() && hazard.elapsed_s >= cycle_s {
            commands.entity(entity).despawn();
            continue;
        }
        let into_s = hazard.elapsed_s.rem_euclid(cycle_s);
        if into_s < 0.0 || into_s >= hazard.arm_s + hazard.burn_s {
            // A vent at rest still shows where it is.
            sprite.color = Color::from(look.warning_color).with_alpha(look.rest_alpha);
        } else if into_s < hazard.arm_s {
            // The warning fills in as the burn comes.
            let armed = into_s / hazard.arm_s;
            sprite.color = Color::from(look.warning_color)
                .with_alpha(look.rest_alpha + (look.warning_alpha - look.rest_alpha) * armed);
        } else {
            sprite.color = Color::from(look.burn_color);
            if !hazard.area.intersect(player_box).is_empty() {
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
                        attacker_x: hazard.area.center().x,
                    },
                    hazard.damage,
                );
            }
        }
    }
}
