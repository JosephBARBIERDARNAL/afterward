//! Patches of the room that burn: the one a `hazard` attack lays under its
//! mark and the blast a type with a `death_blast` leaves where it dies. Both
//! warn before they hurt, go once they have burned, and are drawn as fire by
//! the `Flame` they carry (`shaders/fire.rs`).

use super::*;

/// A patch that warns for `arm_s`, then burns for `burn_s` and hurts the
/// player standing in it, and goes once it has burned.
#[derive(Component)]
pub(crate) struct Hazard {
    pub(crate) area: Rect,
    pub(crate) damage: f32,
    pub(crate) arm_s: f32,
    pub(crate) burn_s: f32,
    pub(crate) elapsed_s: f32,
}

impl Hazard {
    /// The blast a type that goes off as it dies leaves round `middle`.
    pub(crate) fn death_blast(blast: &crate::data::DeathBlastConfig, middle: Vec2) -> Self {
        Self {
            area: Rect::from_center_half_size(middle, Vec2::splat(blast.radius_h * H_WORLD_UNITS)),
            damage: blast.damage,
            arm_s: blast.fuse_s,
            burn_s: blast.burn_s,
            elapsed_s: 0.0,
        }
    }

    /// Lays the patch in the room, cold: `update_hazards` heats it.
    pub(crate) fn lay(self, commands: &mut Commands) {
        commands.spawn((
            Transform::from_translation(self.area.center().extend(3.5)),
            Visibility::default(),
            Flame {
                size: self.area.size(),
                heat: 0.0,
                burning: false,
            },
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
    mut hazards: Query<(Entity, &mut Hazard, &mut Flame)>,
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
    let player_box = player_hurtbox(player_transform.translation, &data);
    for (entity, mut hazard, mut flame) in &mut hazards {
        hazard.elapsed_s += time.delta_secs().min(0.05);
        if hazard.elapsed_s >= hazard.arm_s + hazard.burn_s {
            commands.entity(entity).despawn();
            continue;
        }
        let burning = hazard.elapsed_s >= hazard.arm_s;
        // The warning builds as the burn comes.
        let heat = if burning {
            1.0
        } else {
            hazard.elapsed_s / hazard.arm_s
        };
        // Only a fire that has changed is written, so one burning steadily is
        // not drawn again every frame.
        let stoked = Flame {
            heat,
            burning,
            ..*flame
        };
        if *flame != stoked {
            *flame = stoked;
        }
        if burning && !hazard.area.intersect(player_box).is_empty() {
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
