use super::*;

/// How far an enemy may have stepped along a slope between two frames, in
/// world units: its walk and its feet are moved by different systems, so the
/// slope is given this much room to have carried it up or down.
const SLOPE_STRIDE: f32 = HALF_BODY_WIDTH;

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
    sounds: ResMut<'w, SoundQueue>,
}

pub(in crate::game) fn update_enemy_ai(
    mut commands: Commands,
    resources: EnemyAiResources,
    mut player_query: EnemyAiPlayerQuery,
    mut enemies: EnemyAiEnemiesQuery,
    mut decoys: DecoyQuery,
    platforms: Query<&PlatformSurface>,
) {
    let EnemyAiResources {
        time,
        data,
        sheets,
        mut session,
        mut sounds,
    } = resources;
    let Ok((player_transform, mut player)) = player_query.single_mut() else {
        return;
    };
    let dt = time.delta_secs().min(0.05);
    let gravity = data.gameplay.player.gravity_h_s2 * H_WORLD_UNITS;
    let jumping = &data.enemies.jumping;
    let surfaces = Platforms(platforms.iter().collect());
    let level_change_threshold = jumping.level_change_threshold_h * H_WORLD_UNITS;
    // Attacks start once the player's hurtbox edge enters their hitbox.
    let player_half_width_h = data.gameplay.player.hurtbox_h.width / 2.0;
    let player_box = player_hurtbox(player_transform.translation, &data);
    let player_mark = Mark {
        hurtbox: player_box,
        feet: Vec2::new(
            player_transform.translation.x,
            player_transform.translation.y - 0.5 * H_WORLD_UNITS,
        ),
        grounded: player.grounded,
        decoy: None,
    };
    player.invulnerability_timer = (player.invulnerability_timer - dt).max(0.0);

    for (mut enemy, mut transform, mut sprite, mut animation) in &mut enemies {
        // The flare a wound left goes out by itself, on the dead as well.
        if enemy.flash_s > 0.0 {
            enemy.flash_s = (enemy.flash_s - dt).max(0.0);
            if enemy.flash_s == 0.0 {
                sprite.color = Color::WHITE;
            }
        }
        if enemy.health <= 0.0 {
            continue;
        }
        enemy.marked_timer = (enemy.marked_timer - dt).max(0.0);
        enemy.attack_cooldown = (enemy.attack_cooldown - dt).max(0.0);
        enemy.hurt_timer = (enemy.hurt_timer - dt).max(0.0);
        enemy.jump_cooldown = (enemy.jump_cooldown - dt).max(0.0);
        if enemy.knockback_velocity.abs() > 0.01 {
            // A shove stops against a platform's side and the room's edge,
            // as a walk does.
            let wanted_x =
                (transform.translation.x + enemy.knockback_velocity * H_WORLD_UNITS * dt).clamp(
                    HALF_BODY_WIDTH,
                    data.rooms[&session.room_id].size.width_h * H_WORLD_UNITS - HALF_BODY_WIDTH,
                );
            transform.translation.x = surfaces.blocked_x(
                transform.translation.x,
                wanted_x,
                enemy.home_y - enemy.feet_offset,
            );
            enemy.knockback_velocity = move_towards(
                enemy.knockback_velocity,
                0.0,
                blows::KNOCKBACK_DRAG_H_S2 * dt,
            );
        }
        let sees_player =
            enemy.sees_player(&transform, player_box, &data.enemies.detection, &surfaces);
        if !enemy.engaged && sees_player {
            enemy.engaged = true;
            sounds.push(SoundCue::enemy_idle(&enemy.type_id));
        }
        if enemy.engaged && !data.enemies.detection.retain_target && !sees_player {
            enemy.engaged = false;
        }
        if !enemy.engaged {
            if enemy.procedural_movement.is_some() {
                transform.translation.y = enemy.home_y;
                transform.rotation = Quat::IDENTITY;
            }
            continue;
        }
        // Decoy's double stands in for the player: an engaged enemy walks at
        // and swings at whichever of the two is nearer to it.
        let mark = player_mark.nearest_to(transform.translation.x, &decoys, &data);
        let dx = mark.feet.x - transform.translation.x;
        let distance_h = dx.abs() / H_WORLD_UNITS;
        enemy.facing = if dx < 0.0 { -1.0 } else { 1.0 };
        sprite.flip_x = enemy.facing < 0.0;
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
            if !matches!(&*enemy.type_id, "emitter_unit" | "black_knight") {
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
            if attack.hits(feet, dx, mark.hurtbox) {
                mark.strike(
                    PlayerDamageResources {
                        commands: &mut commands,
                        data: &data,
                        session: &mut session,
                        sounds: &mut sounds,
                    },
                    PlayerDamageTarget {
                        player: &mut player,
                        transform: player_transform,
                        attacker_x: transform.translation.x,
                    },
                    &mut decoys,
                    attack.damage,
                );
            }
            enemy.play_out("land", &mut sprite, &mut animation, &sheets);
        }

        // The second phase is announced on its feet, and a boss drawn with a
        // flare for it drops whatever it was doing, a flinch included, to
        // stand in it.
        if !enemy.phase_two_announced
            && enemy
                .phase_two
                .is_some_and(|(threshold, _)| enemy.health / enemy.max_health <= threshold)
        {
            enemy.phase_two_announced = true;
            sounds.push(SoundCue::BossPhase);
            enemy.stance = None;
            enemy.play_out("buff", &mut sprite, &mut animation, &sheets);
            if enemy.stance.is_some() {
                enemy.hurt_timer = 0.0;
                enemy.windup_remaining = 0.0;
                enemy.combo_hit_pending = false;
                enemy.countering = false;
            }
        }

        if let Some((stance, left_s)) = enemy.stance {
            let left_s = left_s - dt;
            if let Stance::Roll { direction } = stance
                && let Some(roll) = &data.enemies.types[&enemy.type_id].roll
            {
                // It rolls along the floor it stands on, and stops at a wall
                // or a ledge as a walk does. Its clip is spread over the roll.
                let next_x = surfaces.blocked_x(
                    feet.x,
                    (feet.x + direction * roll.distance_h / roll.duration_s * H_WORLD_UNITS * dt)
                        .clamp(
                            HALF_BODY_WIDTH,
                            data.rooms[&session.room_id].size.width_h * H_WORLD_UNITS
                                - HALF_BODY_WIDTH,
                        ),
                    feet.y,
                );
                if surfaces.standing_on(next_x, feet.y, SLOPE_STRIDE).is_some() {
                    transform.translation.x = next_x;
                }
                sprite.flip_x = direction < 0.0;
                let frame = (1.0 - left_s / roll.duration_s) * animation.frames as f32;
                animation.hold_frame(&mut sprite, frame as usize);
            }
            enemy.stance = (left_s > 0.0).then_some((stance, left_s));
            if enemy.stance.is_none() && stance == Stance::Guard {
                enemy.play_out("guard_miss", &mut sprite, &mut animation, &sheets);
            }
            continue;
        }

        if enemy.hurt_timer > 0.0 {
            if enemy.procedural_movement.is_some() {
                transform.translation.y = enemy.home_y;
                transform.rotation = Quat::IDENTITY;
            }
            set_enemy_animation(&mut sprite, &mut animation, &sheets, &enemy.type_id, "hurt");
            continue;
        }

        // Still standing as it was before it noticed anyone: a type drawn
        // with a weapon to draw draws it before anything else.
        if animation.sheet == enemy.type_id.idle_sheet() {
            enemy.play_out("draw", &mut sprite, &mut animation, &sheets);
        }
        // Beaten on for long enough, it rolls through whoever is doing it
        // and comes up at their back.
        if let Some(roll) = &data.enemies.types[&enemy.type_id].roll
            && enemy.wounds >= roll.after_wounds
        {
            enemy.wounds = 0;
            enemy.stance = Some((
                Stance::Roll {
                    direction: enemy.facing,
                },
                roll.duration_s,
            ));
            set_enemy_animation(&mut sprite, &mut animation, &sheets, &enemy.type_id, "roll");
        }
        if enemy.stance.is_some() {
            continue;
        }

        if enemy.windup_remaining > 0.0 {
            if enemy.procedural_movement.is_some() {
                transform.translation.y = enemy.home_y;
                transform.rotation = Quat::IDENTITY;
            }
            enemy.windup_remaining = (enemy.windup_remaining - dt).max(0.0);
            let Some(attack) = enemy.attacks.get(enemy.active_attack).cloned() else {
                continue;
            };
            // An attack that names the frame its strike lands on spreads the
            // frames before it over the wind-up, so the telegraph ends
            // exactly as it hits; the clip plays on by itself from there. A
            // combo's second blow comes out of that follow-through.
            let telegraph = match &attack.guard {
                Some(guard) if enemy.countering => {
                    Some((guard.counter_strike_frame, guard.counter_windup_s))
                }
                _ => attack.strike_frame.map(|frame| (frame, attack.windup_s)),
            }
            .filter(|_| !enemy.combo_hit_pending);
            if let Some((strike_frame, windup_s)) = telegraph {
                let frame = (1.0 - enemy.windup_remaining / windup_s) * strike_frame as f32;
                animation.hold_frame(&mut sprite, frame as usize);
            }
            // A blink's wind-up is the body fading out of where it stands.
            if attack.kind == "blink" {
                sprite.color =
                    Color::WHITE.with_alpha(0.15 + 0.85 * enemy.windup_remaining / attack.windup_s);
            }
            if enemy.windup_remaining == 0.0 {
                let kind = attack.kind.as_str();
                // A guard that has been raised is held, until a blow lands
                // on it or it is lowered; `blows.rs` starts the answer.
                if let Some(guard) = attack.guard.as_ref().filter(|_| !enemy.countering) {
                    enemy.stance = Some((Stance::Guard, guard.hold_s));
                    continue;
                }
                enemy.countering = false;
                if kind == "blink" {
                    // It comes back on the floor its mark stands on, past
                    // them by its reach: at their back, or in front of them
                    // where the floor ends behind. Platforms are no cover
                    // from it. A mark in mid-air has no floor to come back on.
                    let past = if dx < 0.0 { -1.0 } else { 1.0 };
                    let landing = [past, -past]
                        .map(|side| mark.feet.x + side * attack.reach_h * H_WORLD_UNITS)
                        .into_iter()
                        .find(|x| {
                            surfaces
                                .standing_on(*x, mark.feet.y, SLOPE_STRIDE)
                                .is_some()
                        });
                    if let Some(x) = landing.filter(|_| mark.grounded) {
                        transform.translation.x = x;
                        enemy.home_y = mark.feet.y + enemy.feet_offset;
                        transform.translation.y = enemy.home_y;
                    }
                    sprite.color = Color::WHITE;
                    continue;
                }
                if let Some(hazard) = attack.hazard.as_ref().filter(|_| kind == "hazard") {
                    // The floor under its mark is set alight: the patch
                    // warns, then burns whoever is still standing in it.
                    sounds.push(SoundCue::EnemyAttack(audio::EnemySoundStyle::from(
                        &enemy.type_id,
                    )));
                    let floor = surfaces
                        .floor_below(mark.feet.x, mark.feet.y + SLOPE_STRIDE)
                        .unwrap_or(mark.feet.y);
                    let half_width = hazard.size_h.width / 2.0 * H_WORLD_UNITS;
                    Hazard {
                        area: Rect::new(
                            mark.feet.x - half_width,
                            floor,
                            mark.feet.x + half_width,
                            floor + hazard.size_h.height * H_WORLD_UNITS,
                        ),
                        damage: attack.damage,
                        arm_s: hazard.arm_s,
                        burn_s: hazard.burn_s,
                        rest_s: None,
                        elapsed_s: 0.0,
                    }
                    .lay(&mut commands, &data);
                    if attack.strike_frame.is_some() {
                        enemy.stance = Some((
                            Stance::Clip,
                            animation.frames.saturating_sub(animation.frame + 1) as f32
                                * animation.timer.duration().as_secs_f32(),
                        ));
                    }
                    continue;
                }
                if kind == "jump_slam" {
                    // Leap at the player; the slam resolves on touchdown.
                    let jump_height = enemy.jump_height_h * H_WORLD_UNITS;
                    let arc = JumpArc::between(
                        feet,
                        Vec2::new(mark.feet.x, mark.feet.y.min(feet.y + jump_height)),
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
                    let projectile_size = attack.projectile_size;
                    // A type drawn with a shot of its own (`<type>_shot`)
                    // fires that, at the size its body is drawn at; the
                    // others fire a plain warm bolt.
                    let shot_key = format!("{}_shot", enemy.type_id);
                    let shot = sheets.0.get(&shot_key);
                    let shot_scale = shot
                        .and(data.enemies.types[&enemy.type_id].sprite)
                        .map_or(1.0, |sprite| sprite.scale);
                    let mut projectile = commands.spawn((
                        Transform::from_translation(
                            transform.translation + Vec3::new(direction * 24.0, 0.0, 0.4),
                        )
                        .with_scale(Vec3::splat(shot_scale)),
                        CombatProjectile {
                            velocity_x: direction * attack.projectile_speed_h_s * H_WORLD_UNITS,
                            damage: attack.damage,
                            life_s: 3.0,
                            remaining_range: attack.projectile_speed_h_s * H_WORLD_UNITS * 3.0,
                            half_size: projectile_size / 2.0,
                        },
                        RoomEntity,
                    ));
                    match shot {
                        Some(sheet) => projectile.insert((
                            Sprite {
                                flip_x: direction < 0.0,
                                ..Sprite::from_atlas_image(
                                    sheet.image.clone(),
                                    TextureAtlas {
                                        layout: sheet.layout.clone(),
                                        index: 0,
                                    },
                                )
                            },
                            AnimatedSprite {
                                sheet: shot_key,
                                frame: 0,
                                frames: sheet.frames,
                                timer: Timer::from_seconds(1.0 / sheet.fps, TimerMode::Repeating),
                                looping: true,
                            },
                        )),
                        None => projectile.insert(Sprite::from_color(
                            Color::srgb(0.95, 0.3, 0.24),
                            projectile_size,
                        )),
                    };
                } else {
                    if (kind == "telegraphed_lunge" || kind == "lunge")
                        && distance_h > (attack.reach_h + player_half_width_h)
                    {
                        transform.translation.x += dx.signum()
                            * (distance_h - (attack.reach_h + player_half_width_h)).min(1.5)
                            * H_WORLD_UNITS;
                    }
                    // Struck from where the enemy stands now, after any lunge.
                    let strike_feet = Vec2::new(transform.translation.x, feet.y);
                    if attack.hits(strike_feet, dx, mark.hurtbox) {
                        mark.strike(
                            PlayerDamageResources {
                                commands: &mut commands,
                                data: &data,
                                session: &mut session,
                                sounds: &mut sounds,
                            },
                            PlayerDamageTarget {
                                player: &mut player,
                                transform: player_transform,
                                attacker_x: transform.translation.x,
                            },
                            &mut decoys,
                            attack.damage,
                        );
                    }
                    // The strike carries on along the floor as a low wave,
                    // which is jumped or rolled through.
                    if let Some(wave) = &attack.shockwave {
                        let size = Vec2::new(wave.size_h.width, wave.size_h.height) * H_WORLD_UNITS;
                        commands.spawn((
                            Sprite::from_color(
                                Color::from(data.gameplay.effects.hazard.shockwave_color),
                                size,
                            ),
                            Transform::from_xyz(
                                strike_feet.x + enemy.facing * attack.reach_h * H_WORLD_UNITS,
                                // Clear of the floor it runs along, which
                                // would stop it as it stops a shot.
                                strike_feet.y + size.y / 2.0 + 2.0,
                                3.4,
                            ),
                            CombatProjectile {
                                velocity_x: enemy.facing * wave.speed_h_s * H_WORLD_UNITS,
                                damage: wave.damage,
                                life_s: wave.range_h / wave.speed_h_s,
                                remaining_range: wave.range_h * H_WORLD_UNITS,
                                half_size: size / 2.0,
                            },
                            RoomEntity,
                        ));
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
                // A clip with frames left after its last blow or its shot
                // plays them out before the enemy moves again: its
                // follow-through is the opening it leaves.
                if attack.strike_frame.is_some() && !enemy.combo_hit_pending {
                    enemy.stance = Some((
                        Stance::Clip,
                        animation.frames.saturating_sub(animation.frame + 1) as f32
                            * animation.timer.duration().as_secs_f32(),
                    ));
                }
            }
            continue;
        }

        // Only a grounded player defines a level; a mid-jump player does not
        // send enemies scrambling for platforms.
        let rise = mark.feet.y - feet.y;
        let on_another_floor = mark.grounded && rise.abs() > level_change_threshold;
        let chosen = enemy.choose_attack(distance_h, on_another_floor);
        let Some(attack) = enemy.attacks.get(chosen).cloned() else {
            continue;
        };
        // A blink is thrown at a mark on another floor from where it stands:
        // it is its way there.
        let other_level = on_another_floor && attack.kind != "blink";
        let opens_at_h = attack.start_range_h.unwrap_or(attack.reach_h) + player_half_width_h;
        let (minimum_distance, maximum_distance) = if enemy.behavior == "ranged_keep_distance" {
            enemy.preferred_distance_h.unwrap_or((0.0, opens_at_h))
        } else {
            (0.0, opens_at_h)
        };
        let standing = surfaces
            .standing_on(feet.x, feet.y, SLOPE_STRIDE)
            .filter(|_| enemy.grounded)
            .map(|(platform, _)| platform);
        let should_approach = distance_h > maximum_distance || other_level;
        // A shot that is ready is fired from wherever the enemy stands, however
        // close the player is: it only backs away while the next one cools.
        let should_retreat =
            !other_level && distance_h < minimum_distance && enemy.attack_cooldown > 0.0;
        let mut direction = if should_approach {
            dx.signum()
        } else {
            -dx.signum()
        };
        let route = JumpRoute {
            feet,
            target_feet: mark.feet,
            direction,
            jump_height: enemy.jump_height_h * H_WORLD_UNITS,
            air_speed: jumping.air_speed_h_s * H_WORLD_UNITS,
            gravity,
            level_change_threshold,
        };
        // Only the way to the player is worth a leap: backing away never is.
        let plan = standing
            .filter(|_| should_approach)
            .and_then(|platform| route.plan(platform, &surfaces));
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
        // Platform sides stop an enemy as they stop the player, and so does a
        // ledge: it keeps to the floor it stands on. It only steps off on its
        // way to a player standing below, onto a floor that is not too far
        // down and no lower than the player's own level.
        let next_x = surfaces.blocked_x(
            feet.x,
            (feet.x + direction * enemy.movement_speed_h_s * H_WORLD_UNITS * dt).clamp(
                HALF_BODY_WIDTH,
                data.rooms[&session.room_id].size.width_h * H_WORLD_UNITS - HALF_BODY_WIDTH,
            ),
            feet.y,
        );
        let keeps_its_floor = surfaces.standing_on(next_x, feet.y, SLOPE_STRIDE).is_some();
        let drops_after_the_player = should_approach
            && mark.grounded
            && surfaces.floor_below(next_x, feet.y).is_some_and(|floor| {
                feet.y - floor <= jumping.max_drop_h * H_WORLD_UNITS
                    && floor >= mark.feet.y - level_change_threshold
            });
        waiting |= !(keeps_its_floor || drops_after_the_player);
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
            transform.translation.x = next_x;
            let movement_speed_h_s = enemy.movement_speed_h_s;
            let home_y = enemy.home_y;
            let facing = enemy.facing;
            if let Some(movement) = &mut enemy.procedural_movement {
                movement.phase_radians = (movement.phase_radians
                    + dt * movement_speed_h_s * movement.cycles_per_h * std::f32::consts::TAU)
                    .rem_euclid(std::f32::consts::TAU);
                let gait = movement.phase_radians.sin();
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
            // Held where it stands with the player out of its reach, only a
            // shot is worth throwing.
            let in_reach = distance_h <= maximum_distance || attack.kind.contains("projectile");
            if enemy.attack_cooldown <= 0.0 && !other_level && in_reach {
                enemy.movement_sound_timer = 0.0;
                enemy.active_attack = chosen;
                enemy.next_attack = (chosen + 1) % enemy.attacks.len();
                enemy.windup_remaining = attack.windup_s;
                let phase_multiplier = enemy
                    .phase_two
                    .filter(|(threshold, _)| enemy.health / enemy.max_health <= *threshold)
                    .map_or(1.0, |(_, multiplier)| multiplier);
                enemy.attack_cooldown = attack.cooldown_s * phase_multiplier;
                sounds.push(SoundCue::EnemyTelegraph(audio::EnemySoundStyle::from(
                    &enemy.type_id,
                )));
                let action = match (&attack.clip, attack.kind.as_str()) {
                    (Some(clip), _) => clip,
                    (None, "telegraphed_lunge") => "attack_2",
                    (None, "jump_slam") => "jump",
                    _ if enemy.type_id == "emitter_unit" => "attack_1",
                    _ => "attack",
                };
                set_enemy_animation(&mut sprite, &mut animation, &sheets, &enemy.type_id, action);
            } else {
                enemy.movement_sound_timer = 0.0;
                // Heard as it comes to rest, not for as long as it stands.
                if set_enemy_animation(&mut sprite, &mut animation, &sheets, &enemy.type_id, "idle")
                {
                    sounds.push(SoundCue::enemy_idle(&enemy.type_id));
                }
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
    let room_width = data.rooms[&session.room_id].size.width_h * H_WORLD_UNITS;
    let surfaces = Platforms(platforms.iter().collect());

    for (mut enemy, mut transform) in &mut enemies {
        if enemy.health <= 0.0 {
            // Corpses drop straight down instead of finishing a leap.
            enemy.jump = None;
        }
        let gait_offset = transform.translation.y - enemy.home_y;
        let previous_foot_y = enemy.home_y - enemy.feet_offset;
        if let Some(jump) = enemy.jump.filter(|jump| !jump.landed) {
            let wanted_x = (transform.translation.x + jump.velocity_x * dt)
                .clamp(HALF_BODY_WIDTH, room_width - HALF_BODY_WIDTH);
            // As for the player, the frame's highest feet decide what its
            // sides block, so a leap over a ledge is never cut short.
            let reach_foot_y = previous_foot_y.max(previous_foot_y + enemy.vertical_velocity * dt);
            transform.translation.x =
                surfaces.blocked_x(transform.translation.x, wanted_x, reach_foot_y);
        }
        let x = transform.translation.x;
        let standing = surfaces
            .standing_on(x, previous_foot_y, SLOPE_STRIDE)
            .filter(|_| enemy.vertical_velocity <= 0.0);

        let (next_foot_y, grounded) = if let Some((_, floor)) = standing {
            enemy.vertical_velocity = 0.0;
            (floor, true)
        } else {
            // Average velocity over the frame keeps jump arcs exact at any frame rate.
            let launch_velocity = enemy.vertical_velocity;
            enemy.vertical_velocity -= gravity * dt;
            let mut next_foot_y =
                previous_foot_y + (launch_velocity + enemy.vertical_velocity) / 2.0 * dt;
            // A rising enemy bumps its head on a platform's underside.
            if launch_velocity > 0.0
                && let Some(bottom) = surfaces.ceiling(
                    x,
                    previous_foot_y + H_WORLD_UNITS,
                    next_foot_y + H_WORLD_UNITS,
                )
            {
                next_foot_y = bottom - H_WORLD_UNITS;
                enemy.vertical_velocity = 0.0;
            }
            if enemy.vertical_velocity <= 0.0
                && let Some((_, floor)) =
                    surfaces.landing(x, previous_foot_y, next_foot_y, SLOPE_STRIDE, false)
            {
                enemy.vertical_velocity = 0.0;
                (floor, true)
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
