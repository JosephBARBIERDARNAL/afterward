//! Patches of the room that burn: the one a `hazard` attack lays under its
//! mark and the blast a type with a `death_blast` leaves where it dies. Both
//! warn before they hurt, go once they have burned, and are drawn as fire by
//! the `Flame` they carry (`shaders/fire.rs`), or as a sheet rising out of the
//! floor (`HazardRise`) when the attack names one.

use super::*;
use bevy::sprite::Anchor;

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

    /// Lays the patch in the room drawn as `sheet`, as wide as the patch and
    /// standing on its floor: `update_hazards` raises it.
    pub(crate) fn lay_drawn(self, commands: &mut Commands, sheet: &SpriteSheet, rise_frame: usize) {
        let scale = self.area.width() / sheet.frame_size.x;
        commands.spawn((
            Transform::from_xyz(self.area.center().x, self.area.min.y, 3.5)
                .with_scale(Vec3::splat(scale)),
            Sprite::from_atlas_image(
                sheet.image.clone(),
                TextureAtlas {
                    layout: sheet.layout.clone(),
                    index: 0,
                },
            ),
            Anchor::BOTTOM_CENTER,
            AnimatedSprite {
                sheet: String::new(),
                frame: 0,
                frames: sheet.frames,
                timer: Timer::from_seconds(1.0 / sheet.fps, TimerMode::Repeating),
                looping: false,
            },
            HazardRise { rise_frame },
            self,
            RoomEntity,
        ));
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

/// A patch drawn as a sheet: the frames before `rise_frame` are spread over
/// its warning, the rest over its burn.
#[derive(Component)]
pub(crate) struct HazardRise {
    rise_frame: usize,
}

/// The sheet a drawn patch rises in, and where it stands in it.
type DrawnHazard<'a> = (&'a HazardRise, &'a mut Sprite, &'a mut AnimatedSprite);

/// What may still hurt the player once the fight is won: the patches burning
/// and the shots in the air.
type Leftovers = Or<(With<Hazard>, With<CombatProjectile>)>;

/// Once the fight is won, nothing still flying or burning can hurt the
/// player: the shots in the air and the patches on the floor go out.
pub(in crate::game) fn put_out_a_won_room(
    mut commands: Commands,
    data: Res<GameData>,
    session: Res<RunSession>,
    leftovers: Query<Entity, Leftovers>,
) {
    if !session.fight_won(&data) {
        return;
    }
    for entity in &leftovers {
        commands.entity(entity).despawn();
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
    mut hazards: Query<(Entity, &mut Hazard, Option<&mut Flame>, Option<DrawnHazard>)>,
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
    for (entity, mut hazard, flame, drawn) in &mut hazards {
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
        if let Some(mut flame) = flame {
            let stoked = Flame {
                heat,
                burning,
                ..*flame
            };
            if *flame != stoked {
                *flame = stoked;
            }
        }
        if let Some((rise, mut sprite, mut animation)) = drawn {
            let frame = if burning {
                rise.rise_frame as f32
                    + (hazard.elapsed_s - hazard.arm_s) / hazard.burn_s
                        * animation.frames.saturating_sub(rise.rise_frame) as f32
            } else {
                heat * rise.rise_frame as f32
            };
            animation.hold_frame(&mut sprite, frame as usize);
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
                },
                hazard.damage,
            );
        }
    }
}
