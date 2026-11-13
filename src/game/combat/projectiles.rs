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
    sheets: Res<'w, SpriteSheets>,
    sounds: ResMut<'w, SoundQueue>,
}

pub(in crate::game) fn update_projectiles(
    mut commands: Commands,
    resources: ProjectileResources,
    mut player_query: ProjectilePlayerQuery,
    mut projectiles: Query<(Entity, &mut Transform, &mut CombatProjectile)>,
    mut decoys: DecoyQuery,
    platforms: Query<&PlatformSurface>,
) {
    let ProjectileResources {
        time,
        data,
        mut session,
        sheets,
        mut sounds,
    } = resources;
    let dt = time.delta_secs().min(0.05);
    let Ok((player_transform, mut player)) = player_query.single_mut() else {
        return;
    };
    let room_width_world = data.rooms[&session.room_id].size.width_h * H_WORLD_UNITS;
    for (projectile_entity, mut projectile_transform, mut projectile) in &mut projectiles {
        let start = projectile_transform.translation.truncate();
        projectile.life_s -= dt;
        let travel = (projectile.velocity_x * dt)
            .clamp(-projectile.remaining_range, projectile.remaining_range);
        projectile.remaining_range -= travel.abs();
        projectile_transform.translation.x += travel;
        let end = projectile_transform.translation.truncate();
        let mut hit = projectile.life_s <= 0.0;
        let shot = Rect::from_center_half_size(end, projectile.half_size);
        // A double left by Decoy is shot at as the player is, and stands in
        // the way of a shot aimed past it.
        let struck_double = decoys
            .iter()
            .find(|(_, decoy_transform, _)| {
                !shot
                    .intersect(player_hurtbox(decoy_transform.translation, &data))
                    .is_empty()
            })
            .map(|(entity, _, _)| entity);
        if let Some(entity) = struck_double {
            hurt_the_decoy(
                &mut commands,
                &mut sounds,
                &mut decoys,
                entity,
                projectile.damage,
            );
            hit = true;
        } else if !shot
            .intersect(player_hurtbox(player_transform.translation, &data))
            .is_empty()
        {
            // Deflector turns aside the first shot of each room.
            if session.has_bonus("deflector") && !session.deflector_used {
                session.deflector_used = true;
                sounds.push(SoundCue::CoverBlock);
                commands.entity(projectile_entity).despawn();
                continue;
            }
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
            // It bursts where it ends, on a body or a wall alike.
            play_effect(
                &mut commands,
                &sheets,
                "effect_hit_spark",
                end.extend(6.0),
                data.gameplay.effects.bolt.flash_size_h,
                projectile.velocity_x < 0.0,
            );
            commands.entity(projectile_entity).despawn();
        }
    }
}
