use super::*;

type ProjectilePlayerQuery<'w, 's> = Query<
    'w,
    's,
    (&'static Transform, &'static mut PlayerBody),
    (With<PlayerBody>, Without<CombatProjectile>),
>;

#[derive(SystemParam)]
pub(in crate::game) struct ProjectileResources<'w> {
    time: Res<'w, Time>,
    data: Res<'w, GameData>,
    session: ResMut<'w, RunSession>,
    sounds: ResMut<'w, SoundQueue>,
}

pub(in crate::game) fn update_projectiles(
    mut commands: Commands,
    resources: ProjectileResources,
    mut player_query: ProjectilePlayerQuery,
    mut projectiles: Query<(Entity, &mut Transform, &mut CombatProjectile)>,
    platforms: Query<&PlatformSurface>,
) {
    let ProjectileResources {
        time,
        data,
        mut session,
        mut sounds,
    } = resources;
    let dt = time.delta_secs().min(0.05);
    let Ok((player_transform, mut player)) = player_query.single_mut() else {
        return;
    };
    let room_width_world = data.rooms[&session.room_id].width_h(&data.gameplay) * H_WORLD_UNITS;
    for (projectile_entity, mut projectile_transform, mut projectile) in &mut projectiles {
        let start = projectile_transform.translation.truncate();
        projectile.life_s -= dt;
        let travel = (projectile.velocity_x * dt)
            .clamp(-projectile.remaining_range, projectile.remaining_range);
        projectile.remaining_range -= travel.abs();
        projectile_transform.translation.x += travel;
        let end = projectile_transform.translation.truncate();
        let mut hit = projectile.life_s <= 0.0;
        let hurtbox = player_hurtbox(player_transform.translation, &data);
        if !Rect::from_center_half_size(end, projectile.half_size)
            .intersect(hurtbox)
            .is_empty()
        {
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
                    attacker_x: projectile_transform.translation.x - projectile.velocity_x,
                },
                projectile.damage,
            );
            hit = true;
        }
        let half_size = projectile.half_size;
        let outside_room = end.x - half_size.x < 0.0 || end.x + half_size.x > room_width_world;
        if !hit
            && (outside_room
                || platforms
                    .iter()
                    .any(|platform| platform.intersects_projectile(start, end, half_size)))
        {
            sounds.push(SoundCue::ProjectileEnvironmentHit);
            hit = true;
        }
        if projectile.remaining_range <= 0.0 {
            hit = true;
        }
        if hit {
            commands.entity(projectile_entity).despawn();
        }
    }
}
