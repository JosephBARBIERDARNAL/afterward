use super::*;

type ProjectilePlayerQuery<'w, 's> = Query<
    'w,
    's,
    (&'static Transform, &'static mut PlayerBody),
    (With<PlayerBody>, Without<CombatProjectile>),
>;
type ProjectileEnemiesQuery<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static mut EnemyBody,
        &'static mut Transform,
        &'static mut Sprite,
        &'static mut AnimatedSprite,
    ),
    (
        Without<PlayerBody>,
        Without<CombatProjectile>,
        Without<DeadEnemy>,
    ),
>;

#[derive(SystemParam)]
pub(in crate::game) struct ProjectileResources<'w> {
    time: Res<'w, Time>,
    data: Res<'w, GameData>,
    sheets: Res<'w, SpriteSheets>,
    random: ResMut<'w, RandomState>,
    session: ResMut<'w, RunSession>,
    next_screen: ResMut<'w, NextState<GameScreen>>,
    sounds: ResMut<'w, SoundQueue>,
}

pub(in crate::game) fn update_projectiles(
    mut commands: Commands,
    resources: ProjectileResources,
    mut player_query: ProjectilePlayerQuery,
    mut enemies: ProjectileEnemiesQuery,
    mut projectiles: Query<(Entity, &mut Transform, &mut CombatProjectile)>,
    platforms: Query<&PlatformSurface>,
) {
    let ProjectileResources {
        time,
        data,
        sheets,
        mut random,
        mut session,
        mut next_screen,
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
        if projectile.friendly {
            for (enemy_entity, mut enemy, mut enemy_transform, mut sprite, mut animation) in
                &mut enemies
            {
                if enemy.health <= 0.0 {
                    continue;
                }
                let enemy_left = start.x.min(end.x) - projectile.half_size.x - 24.0;
                let enemy_right = start.x.max(end.x) + projectile.half_size.x + 24.0;
                let vertical_distance = (enemy_transform.translation.y - start.y).abs();
                if (enemy_left..=enemy_right).contains(&enemy_transform.translation.x)
                    && vertical_distance <= 32.0 + projectile.half_size.y
                {
                    let direction = if projectile.velocity_x > 0.0 {
                        1.0
                    } else {
                        -1.0
                    };
                    let marked_multiplier =
                        if enemy.marked_timer > 0.0 && has_bonus(&session, "marked_prey") {
                            data.bonus_effect("marked_prey", "marked_gun_damage_multiplier")
                        } else {
                            1.0
                        };
                    let killed = damage_enemy(
                        EnemyDamageResources {
                            commands: &mut commands,
                            data: &data,
                            session: &session,
                            random: &mut random,
                            sheets: &sheets,
                            sounds: &mut sounds,
                        },
                        EnemyDamageTarget {
                            entity: enemy_entity,
                            enemy: &mut enemy,
                            transform: &mut enemy_transform,
                            sprite: &mut sprite,
                            animation: &mut animation,
                        },
                        EnemyHit {
                            damage: projectile.damage * marked_multiplier,
                            knockback_h_s: projectile.knockback_h_s,
                            knockback_direction: direction,
                            sword_hit: false,
                        },
                    );
                    if killed {
                        player.chain_reaction_timer =
                            data.bonus_effect("chain_reaction", "kill_damage_duration_s");
                    }
                    register_hit(&mut player, &session, &data);
                    sounds.push(SoundCue::BulletHit);
                    if let Some(sheet) = sheets.0.get("effect_gun_hit") {
                        let mut impact_sprite = Sprite::from_atlas_image(
                            sheet.image.clone(),
                            TextureAtlas {
                                layout: sheet.layout.clone(),
                                index: 0,
                            },
                        );
                        impact_sprite.flip_x = direction < 0.0;
                        commands.spawn((
                            impact_sprite,
                            Transform::from_translation(end.extend(4.0)),
                            AnimatedSprite {
                                sheet: "effect_gun_hit".to_string(),
                                frame: 0,
                                frames: sheet.frames,
                                timer: Timer::from_seconds(1.0 / sheet.fps, TimerMode::Repeating),
                                looping: false,
                            },
                            ImpactEffect {
                                timer: Timer::from_seconds(
                                    sheet.frames as f32 / sheet.fps,
                                    TimerMode::Once,
                                ),
                            },
                            RoomEntity,
                        ));
                    }
                    hit = true;
                    break;
                }
            }
        } else {
            let delta = player_transform.translation - projectile_transform.translation;
            if delta.x.abs() <= 24.0 && delta.y.abs() <= 36.0 {
                apply_player_damage(
                    PlayerDamageResources {
                        commands: &mut commands,
                        data: &data,
                        session: &mut session,
                        next_screen: &mut next_screen,
                        sounds: &mut sounds,
                    },
                    PlayerDamageTarget {
                        player: &mut player,
                        transform: player_transform,
                    },
                    projectile.damage,
                );
                hit = true;
            }
        }
        let half_size = projectile.half_size;
        let outside_room = end.x - half_size.x < 0.0 || end.x + half_size.x > room_width_world;
        if !hit
            && (outside_room
                || platforms
                    .iter()
                    .any(|platform| platform.intersects_projectile(start, end, half_size)))
        {
            sounds.push(SoundCue::BulletEnvironmentHit);
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
