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
    for (projectile_entity, mut projectile_transform, mut projectile) in &mut projectiles {
        projectile.life_s -= dt;
        projectile_transform.translation.x += projectile.velocity_x * dt;
        let mut hit = projectile.life_s <= 0.0;
        if projectile.friendly {
            for (enemy_entity, mut enemy, mut enemy_transform, mut sprite, mut animation) in
                &mut enemies
            {
                if enemy.health <= 0.0 {
                    continue;
                }
                let delta = enemy_transform.translation - projectile_transform.translation;
                if delta.x.abs() <= 24.0 && delta.y.abs() <= 32.0 {
                    let direction = if projectile.velocity_x > 0.0 {
                        1.0
                    } else {
                        -1.0
                    };
                    let marked_multiplier =
                        if enemy.marked_timer > 0.0 && has_bonus(&session, "marked_prey") {
                            effect_number(&data, "marked_prey", "marked_gun_damage_multiplier")
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
                            effect_number(&data, "chain_reaction", "kill_damage_duration_s");
                    }
                    register_hit(&mut player, &session, &data);
                    sounds.push(SoundCue::BulletHit);
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
        if hit {
            commands.entity(projectile_entity).despawn();
        }
    }
}
