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
    platforms: Query<&PlatformSurface>,
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
    let gravity = data.gameplay.player.gravity_h_s2 * H_WORLD_UNITS;
    let jumping = &data.enemies.jumping;
    let level_change_threshold = jumping.level_change_threshold_h * H_WORLD_UNITS;
    let player_feet = Vec2::new(
        player_transform.translation.x,
        player_transform.translation.y - 0.5 * H_WORLD_UNITS,
    );
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
        enemy.jump_cooldown = (enemy.jump_cooldown - dt).max(0.0);
        if enemy.knockback_velocity.abs() > 0.01 {
            transform.translation.x += enemy.knockback_velocity * H_WORLD_UNITS * dt;
            enemy.knockback_velocity = move_towards(enemy.knockback_velocity, 0.0, 14.0 * dt);
        }
        let dx = player_transform.translation.x - transform.translation.x;
        let dy = player_transform.translation.y - transform.translation.y;
        let distance_h = dx.abs() / H_WORLD_UNITS;
        if !enemy.engaged && distance_h <= data.enemies.detection_range_h {
            enemy.engaged = true;
            sounds.push(SoundCue::enemy_idle(&enemy.type_id));
        }
        if enemy.engaged
            && !data.enemies.retain_target_after_detection
            && distance_h > data.enemies.detection_range_h
        {
            enemy.engaged = false;
        }
        if !enemy.engaged {
            if enemy.procedural_movement.is_some() {
                transform.translation.y = enemy.home_y;
                transform.rotation = Quat::IDENTITY;
            }
            continue;
        }
        sprite.flip_x = dx < 0.0;
        let feet = Vec2::new(transform.translation.x, enemy.home_y - enemy.feet_offset);

        // Airborne after a jump or a step off a ledge: physics owns the motion.
        if !enemy.grounded {
            transform.translation.y = enemy.home_y;
            transform.rotation = Quat::IDENTITY;
            let action = if enemy.vertical_velocity > 0.0 {
                "jump"
            } else {
                "fall"
            };
            set_enemy_animation(&mut sprite, &mut animation, &sheets, &enemy.type_id, action);
            if enemy.type_id != "evil_wizard" {
                // Without jump art, hold the current walk frame in the air.
                animation.timer.reset();
            }
            continue;
        }
        // A landed jump always ends here; a slam also resolves its impact.
        if let Some(jump) = enemy.jump.take()
            && jump.slam
            && let Some(attack) = enemy.attacks.get(enemy.active_attack).cloned()
        {
            sounds.push(SoundCue::EnemyAttack(audio::EnemySoundStyle::from(
                &enemy.type_id,
            )));
            let landing = player_feet - feet;
            if landing.x.abs() <= attack.landing_radius_h * H_WORLD_UNITS
                && landing.y.abs() <= attack.landing_radius_h * H_WORLD_UNITS
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
        }

        if enemy.hurt_timer > 0.0 {
            if enemy.procedural_movement.is_some() {
                transform.translation.y = enemy.home_y;
                transform.rotation = Quat::IDENTITY;
            }
            set_enemy_animation(&mut sprite, &mut animation, &sheets, &enemy.type_id, "hurt");
            continue;
        }

        if enemy.windup_remaining > 0.0 {
            if enemy.procedural_movement.is_some() {
                transform.translation.y = enemy.home_y;
                transform.rotation = Quat::IDENTITY;
            }
            enemy.windup_remaining = (enemy.windup_remaining - dt).max(0.0);
            if enemy.windup_remaining == 0.0
                && let Some(attack) = enemy.attacks.get(enemy.active_attack).cloned()
            {
                let kind = attack.kind.as_str();
                if kind == "jump_slam" {
                    // Leap at the player; the slam resolves on touchdown.
                    let jump_height = enemy.jump_height_h * H_WORLD_UNITS;
                    let arc = JumpArc::between(
                        feet,
                        Vec2::new(player_feet.x, player_feet.y.min(feet.y + jump_height)),
                        jump_height,
                        gravity,
                    );
                    let air_speed = jumping.air_speed_h_s * H_WORLD_UNITS;
                    enemy.vertical_velocity = arc.vertical_velocity;
                    enemy.grounded = false;
                    enemy.jump = Some(EnemyJump {
                        velocity_x: arc.horizontal_velocity.clamp(-air_speed, air_speed),
                        slam: true,
                        landed: false,
                    });
                    set_enemy_animation(
                        &mut sprite,
                        &mut animation,
                        &sheets,
                        &enemy.type_id,
                        "jump",
                    );
                    continue;
                }
                sounds.push(SoundCue::EnemyAttack(audio::EnemySoundStyle::from(
                    &enemy.type_id,
                )));
                if kind.contains("projectile") {
                    let direction = if dx >= 0.0 { 1.0 } else { -1.0 };
                    let projectile_size = Vec2::new(14.0, 8.0);
                    commands.spawn((
                        Sprite::from_color(Color::srgb(0.95, 0.3, 0.24), projectile_size),
                        Transform::from_translation(
                            transform.translation + Vec3::new(direction * 24.0, 0.0, 0.4),
                        ),
                        CombatProjectile {
                            velocity_x: direction * attack.projectile_speed_h_s * H_WORLD_UNITS,
                            damage: attack.damage,
                            life_s: 3.0,
                            remaining_range: attack.projectile_speed_h_s * H_WORLD_UNITS * 3.0,
                            half_size: projectile_size / 2.0,
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
        // Only a grounded player defines a level; a mid-jump player does not
        // send enemies scrambling for platforms.
        let rise = player_feet.y - feet.y;
        let other_level = player.grounded && rise.abs() > level_change_threshold;
        let standing = platforms.iter().find(|platform| {
            enemy.grounded
                && feet.x > platform.left
                && feet.x < platform.right
                && (platform.top - feet.y).abs() <= 1.0
        });
        let should_approach = distance_h > maximum_distance || other_level;
        let should_retreat = !other_level && distance_h < minimum_distance;
        let mut direction = match standing {
            // To reach a player below, head for the nearest ledge of this platform.
            Some(platform)
                if other_level
                    && rise < 0.0
                    && (platform.left..=platform.right).contains(&player_feet.x) =>
            {
                if platform.right - feet.x < feet.x - platform.left {
                    1.0
                } else {
                    -1.0
                }
            }
            _ if should_approach => dx.signum(),
            _ => -dx.signum(),
        };
        let plan = standing
            .filter(|_| should_approach || should_retreat)
            .and_then(|platform| {
                JumpRoute {
                    feet,
                    target_feet: player_feet,
                    direction,
                    jump_height: enemy.jump_height_h * H_WORLD_UNITS,
                    air_speed: jumping.air_speed_h_s * H_WORLD_UNITS,
                    gravity,
                    level_change_threshold,
                }
                .plan(platform, platforms.iter())
            });
        let mut waiting = false;
        match plan {
            Some(JumpPlan::Leap(arc)) if enemy.jump_cooldown == 0.0 => {
                enemy.vertical_velocity = arc.vertical_velocity;
                enemy.grounded = false;
                enemy.jump_cooldown = jumping.cooldown_s;
                enemy.jump = Some(EnemyJump {
                    velocity_x: arc.horizontal_velocity,
                    slam: false,
                    landed: false,
                });
                transform.translation.y = enemy.home_y;
                transform.rotation = Quat::IDENTITY;
                set_enemy_animation(&mut sprite, &mut animation, &sheets, &enemy.type_id, "jump");
                continue;
            }
            Some(JumpPlan::Leap(_)) => waiting = true,
            Some(JumpPlan::WalkTo(takeoff_x)) => {
                if (takeoff_x - feet.x).abs() > 1.0 {
                    direction = (takeoff_x - feet.x).signum();
                } else {
                    waiting = true;
                }
            }
            None => {
                // Directly below a higher player with no route up: wait rather than jitter.
                waiting = other_level && rise > 0.0 && dx.abs() < 0.5 * H_WORLD_UNITS;
            }
        }
        if (should_approach || should_retreat) && !waiting {
            enemy.movement_sound_timer = (enemy.movement_sound_timer - dt).max(0.0);
            if enemy.movement_sound_timer == 0.0 {
                sounds.push(SoundCue::EnemyMovement(audio::EnemySoundStyle::from(
                    &enemy.type_id,
                )));
                enemy.movement_sound_timer = data.gameplay.audio.footstep_interval_s;
            }
            if should_approach {
                sprite.flip_x = direction < 0.0;
            }
            let speed = enemy.movement_speed_h_s * H_WORLD_UNITS * dt;
            transform.translation.x += direction * speed;
            let room_width_h = data.rooms[&session.room_id].width_h(&data.gameplay);
            transform.translation.x = transform
                .translation
                .x
                .clamp(24.0, room_width_h * H_WORLD_UNITS - 24.0);
            let movement_speed_h_s = enemy.movement_speed_h_s;
            let home_y = enemy.home_y;
            if let Some(movement) = &mut enemy.procedural_movement {
                movement.phase_radians = (movement.phase_radians
                    + dt * movement_speed_h_s * movement.cycles_per_h * std::f32::consts::TAU)
                    .rem_euclid(std::f32::consts::TAU);
                let gait = movement.phase_radians.sin();
                let facing = if dx < 0.0 { -1.0 } else { 1.0 };
                transform.translation.y =
                    home_y + gait.max(0.0) * movement.bob_height_h * H_WORLD_UNITS;
                transform.rotation = Quat::from_rotation_z(gait * movement.lean_radians * facing);
            }
            set_enemy_animation(&mut sprite, &mut animation, &sheets, &enemy.type_id, "walk");
        } else {
            if let Some(movement) = &mut enemy.procedural_movement {
                movement.phase_radians = 0.0;
                transform.translation.y = enemy.home_y;
                transform.rotation = Quat::IDENTITY;
            }
            if enemy.attack_cooldown <= 0.0 && !other_level {
                enemy.movement_sound_timer = 0.0;
                enemy.active_attack = enemy.next_attack;
                enemy.next_attack = (enemy.next_attack + 1) % enemy.attacks.len();
                enemy.windup_remaining = attack.windup_s;
                let phase_multiplier = enemy
                    .phase_two
                    .filter(|(threshold, _)| enemy.health / enemy.max_health <= *threshold)
                    .map_or(1.0, |(_, multiplier)| multiplier);
                enemy.attack_cooldown = attack.cooldown_s * phase_multiplier;
                sounds.push(SoundCue::EnemyTelegraph(audio::EnemySoundStyle::from(
                    &enemy.type_id,
                )));
                let action = match attack.kind.as_str() {
                    "telegraphed_lunge" => "attack_2",
                    "jump_slam" => "jump",
                    _ if enemy.type_id == "evil_wizard" => "attack_1",
                    _ => "attack",
                };
                set_enemy_animation(&mut sprite, &mut animation, &sheets, &enemy.type_id, action);
            } else {
                enemy.movement_sound_timer = 0.0;
                if animation.sheet != enemy.type_id.idle_sheet() {
                    sounds.push(SoundCue::enemy_idle(&enemy.type_id));
                }
                set_enemy_animation(&mut sprite, &mut animation, &sheets, &enemy.type_id, "idle");
            }
        }
    }
}

pub(in crate::game) fn update_enemy_platform_physics(
    time: Res<Time>,
    data: Res<GameData>,
    session: Res<RunSession>,
    mut enemies: Query<(&mut EnemyBody, &mut Transform), Without<PlayerBody>>,
    platforms: Query<&PlatformSurface>,
) {
    let dt = time.delta_secs().min(0.05);
    let gravity = data.gameplay.player.gravity_h_s2 * H_WORLD_UNITS;
    let half_width = PLAYER_COLLISION_WIDTH_H * H_WORLD_UNITS / 2.0;
    let room_width = data.rooms[&session.room_id].width_h(&data.gameplay) * H_WORLD_UNITS;

    for (mut enemy, mut transform) in &mut enemies {
        if enemy.health <= 0.0 {
            // Corpses drop straight down instead of finishing a leap.
            enemy.jump = None;
        }
        if let Some(jump) = enemy.jump.filter(|jump| !jump.landed) {
            transform.translation.x =
                (transform.translation.x + jump.velocity_x * dt).clamp(24.0, room_width - 24.0);
        }
        let gait_offset = transform.translation.y - enemy.home_y;
        let previous_foot_y = enemy.home_y - enemy.feet_offset;
        let overlaps_platform = |platform: &PlatformSurface| {
            transform.translation.x + half_width > platform.left
                && transform.translation.x - half_width < platform.right
        };
        let standing_platform = platforms.iter().find(|platform| {
            enemy.vertical_velocity <= 0.0
                && overlaps_platform(platform)
                && (platform.top - previous_foot_y).abs() <= 1.0
        });

        let (next_foot_y, grounded) = if let Some(platform) = standing_platform {
            enemy.vertical_velocity = 0.0;
            (platform.top, true)
        } else {
            // Average velocity over the frame keeps jump arcs exact at any frame rate.
            let launch_velocity = enemy.vertical_velocity;
            enemy.vertical_velocity -= gravity * dt;
            let next_foot_y =
                previous_foot_y + (launch_velocity + enemy.vertical_velocity) / 2.0 * dt;
            if enemy.vertical_velocity <= 0.0
                && let Some(platform) = platforms.iter().find(|platform| {
                    overlaps_platform(platform)
                        && previous_foot_y >= platform.top - 1.0
                        && next_foot_y <= platform.top
                })
            {
                enemy.vertical_velocity = 0.0;
                (platform.top, true)
            } else {
                (next_foot_y, false)
            }
        };

        enemy.grounded = grounded;
        if grounded && let Some(jump) = &mut enemy.jump {
            jump.landed = true;
        }
        enemy.home_y = next_foot_y + enemy.feet_offset;
        transform.translation.y = enemy.home_y + gait_offset;
    }
}
