use super::*;

type EnemyAiPlayerQuery<'w, 's> = Query<
    'w,
    's,
    (&'static Transform, &'static mut PlayerBody),
    (With<PlayerBody>, Without<EnemyBody>),
>;
type EnemyAiEnemiesQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static mut EnemyBody,
        &'static mut Transform,
        &'static mut Sprite,
        &'static mut AnimatedSprite,
    ),
    (Without<PlayerBody>, Without<DeadEnemy>),
>;

#[derive(SystemParam)]
pub(in crate::game) struct EnemyAiResources<'w> {
    time: Res<'w, Time>,
    data: Res<'w, GameData>,
    sheets: Res<'w, SpriteSheets>,
    session: ResMut<'w, RunSession>,
    next_screen: ResMut<'w, NextState<GameScreen>>,
    sounds: ResMut<'w, SoundQueue>,
}

pub(in crate::game) fn update_enemy_ai(
    mut commands: Commands,
    resources: EnemyAiResources,
    mut player_query: EnemyAiPlayerQuery,
    mut enemies: EnemyAiEnemiesQuery,
) {
    let EnemyAiResources {
        time,
        data,
        sheets,
        mut session,
        mut next_screen,
        mut sounds,
    } = resources;
    let Ok((player_transform, mut player)) = player_query.single_mut() else {
        return;
    };
    let dt = time.delta_secs().min(0.05);
    player.invulnerability_timer = (player.invulnerability_timer - dt).max(0.0);
    player.dash_invulnerability_timer = (player.dash_invulnerability_timer - dt).max(0.0);

    for (mut enemy, mut transform, mut sprite, mut animation) in &mut enemies {
        if enemy.health <= 0.0 {
            continue;
        }
        if !enemy.phase_two_announced
            && enemy
                .phase_two
                .is_some_and(|(threshold, _)| enemy.health / enemy.max_health <= threshold)
        {
            enemy.phase_two_announced = true;
            sounds.push(SoundCue::BossPhase);
        }
        enemy.marked_timer = (enemy.marked_timer - dt).max(0.0);
        enemy.attack_cooldown = (enemy.attack_cooldown - dt).max(0.0);
        enemy.hurt_timer = (enemy.hurt_timer - dt).max(0.0);
        if enemy.knockback_velocity.abs() > 0.01 {
            transform.translation.x += enemy.knockback_velocity * H_WORLD_UNITS * dt;
            enemy.knockback_velocity = move_towards(enemy.knockback_velocity, 0.0, 14.0 * dt);
        }
        let dx = player_transform.translation.x - transform.translation.x;
        let dy = player_transform.translation.y - transform.translation.y;
        let distance_h = dx.abs() / H_WORLD_UNITS;
        if !enemy.engaged && distance_h <= data.enemies.detection_range_h {
            enemy.engaged = true;
        }
        if enemy.engaged
            && !data.enemies.retain_target_after_detection
            && distance_h > data.enemies.detection_range_h
        {
            enemy.engaged = false;
        }
        if !enemy.engaged {
            continue;
        }
        sprite.flip_x = dx < 0.0;

        if enemy.hurt_timer > 0.0 {
            set_enemy_animation(&mut sprite, &mut animation, &sheets, &enemy.type_id, "hurt");
            continue;
        }

        if enemy.windup_remaining > 0.0 {
            enemy.windup_remaining = (enemy.windup_remaining - dt).max(0.0);
            if enemy.type_id == "warrior"
                && enemy.windup_remaining <= 0.4
                && let Some(attack) = enemy.attacks.get(enemy.active_attack)
                && attack.kind == "jump_slam"
                && animation.sheet == "boss_jump"
            {
                set_enemy_animation(&mut sprite, &mut animation, &sheets, "warrior", "attack_3");
            }
            if enemy.windup_remaining == 0.0
                && let Some(attack) = enemy.attacks.get(enemy.active_attack).cloned()
            {
                let kind = attack.kind.as_str();
                if kind == "jump_slam" {
                    transform.translation.y = enemy.home_y;
                    set_enemy_animation(&mut sprite, &mut animation, &sheets, "warrior", "land");
                    let landing_distance_h =
                        (player_transform.translation.x - transform.translation.x).abs()
                            / H_WORLD_UNITS;
                    let landing_height =
                        (player_transform.translation.y - transform.translation.y).abs();
                    if landing_distance_h <= attack.landing_radius_h
                        && landing_height <= attack.landing_radius_h * H_WORLD_UNITS
                    {
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
                            attack.damage,
                        );
                    }
                } else if kind.contains("projectile") {
                    let direction = if dx >= 0.0 { 1.0 } else { -1.0 };
                    commands.spawn((
                        Sprite::from_color(Color::srgb(0.95, 0.3, 0.24), Vec2::new(14.0, 8.0)),
                        Transform::from_translation(
                            transform.translation + Vec3::new(direction * 24.0, 0.0, 0.4),
                        ),
                        CombatProjectile {
                            velocity_x: direction * attack.projectile_speed_h_s * H_WORLD_UNITS,
                            damage: attack.damage,
                            knockback_h_s: 0.0,
                            life_s: 3.0,
                            friendly: false,
                        },
                        RoomEntity,
                    ));
                } else {
                    if (kind == "telegraphed_lunge" || kind == "lunge")
                        && distance_h > attack.reach_h
                    {
                        transform.translation.x +=
                            dx.signum() * (distance_h - attack.reach_h).min(1.5) * H_WORLD_UNITS;
                    }
                    let strike_distance_h =
                        (player_transform.translation.x - transform.translation.x).abs()
                            / H_WORLD_UNITS;
                    if (strike_distance_h <= attack.reach_h && dy.abs() <= H_WORLD_UNITS * 1.5)
                        || (attack.landing_radius_h > 0.0
                            && strike_distance_h <= attack.landing_radius_h
                            && dy.abs() <= attack.landing_radius_h * H_WORLD_UNITS)
                    {
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
                            attack.damage,
                        );
                    }
                    if attack.hits > 1 {
                        if enemy.combo_hit_pending {
                            enemy.combo_hit_pending = false;
                        } else {
                            enemy.combo_hit_pending = true;
                            enemy.windup_remaining = if player.invulnerability_timer > 0.0 {
                                player.invulnerability_timer + 0.06
                            } else {
                                0.25
                            };
                        }
                    }
                }
            }
            continue;
        }

        let Some(attack) = enemy.attacks.get(enemy.next_attack).cloned() else {
            continue;
        };
        let (minimum_distance, maximum_distance) = if enemy.behavior == "ranged_keep_distance" {
            enemy.preferred_distance_h.unwrap_or((0.0, attack.reach_h))
        } else {
            (0.0, attack.reach_h)
        };
        let should_approach = distance_h > maximum_distance;
        let should_retreat = distance_h < minimum_distance;
        if should_approach || should_retreat {
            let direction = if should_approach {
                dx.signum()
            } else {
                -dx.signum()
            };
            let speed = enemy.movement_speed_h_s * H_WORLD_UNITS * dt;
            transform.translation.x += direction * speed;
            let room_width_h = f32::from(data.rooms[&session.room_id].size.width_viewports) * 16.0;
            transform.translation.x = transform
                .translation
                .x
                .clamp(24.0, room_width_h * H_WORLD_UNITS - 24.0);
            set_enemy_animation(&mut sprite, &mut animation, &sheets, &enemy.type_id, "walk");
        } else if enemy.attack_cooldown <= 0.0 {
            enemy.active_attack = enemy.next_attack;
            enemy.next_attack = (enemy.next_attack + 1) % enemy.attacks.len();
            enemy.windup_remaining = attack.windup_s;
            let phase_multiplier = enemy
                .phase_two
                .filter(|(threshold, _)| enemy.health / enemy.max_health <= *threshold)
                .map_or(1.0, |(_, multiplier)| multiplier);
            enemy.attack_cooldown = attack.cooldown_s * phase_multiplier;
            sounds.push(if enemy.type_id == "warrior" {
                SoundCue::BossAttack
            } else {
                SoundCue::EnemyAttack
            });
            if attack.kind == "jump_slam" {
                transform.translation.x += dx.signum() * distance_h.min(1.0) * H_WORLD_UNITS;
                transform.translation.y = enemy.home_y + H_WORLD_UNITS * 1.5;
            }
            let action = match attack.kind.as_str() {
                "telegraphed_lunge" => "attack_2",
                "jump_slam" => "jump",
                _ if enemy.type_id == "warrior" => "attack_1",
                _ => "attack",
            };
            set_enemy_animation(&mut sprite, &mut animation, &sheets, &enemy.type_id, action);
        } else {
            set_enemy_animation(&mut sprite, &mut animation, &sheets, &enemy.type_id, "idle");
        }
    }
}
