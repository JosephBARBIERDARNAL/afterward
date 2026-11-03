use super::*;

#[derive(SystemParam)]
pub(in crate::game) struct PlayerAttackResources<'w> {
    time: Res<'w, Time>,
    keys: Res<'w, ButtonInput<KeyCode>>,
    controls: Res<'w, ControlBindings>,
    data: Res<'w, GameData>,
    session: ResMut<'w, RunSession>,
    random: ResMut<'w, RandomState>,
    sheets: Res<'w, SpriteSheets>,
    sounds: ResMut<'w, SoundQueue>,
}

/// Sword combo: spamming the sword chains swings; each hit plays its own combo
/// animation and later hits multiply the damage further. There is no timing
/// window: the chain only resets after its finisher or once the player stops
/// attacking.
pub(in crate::game) fn player_attacks(
    mut commands: Commands,
    resources: PlayerAttackResources,
    mut player_query: Query<
        (
            &mut PlayerBody,
            &Transform,
            &mut Sprite,
            &mut AnimatedSprite,
        ),
        Without<EnemyBody>,
    >,
    mut enemy_query: Query<
        (
            Entity,
            &mut EnemyBody,
            &mut Transform,
            &mut Sprite,
            &mut AnimatedSprite,
        ),
        Without<PlayerBody>,
    >,
) {
    let PlayerAttackResources {
        time,
        keys,
        controls,
        data,
        mut session,
        mut random,
        sheets,
        mut sounds,
    } = resources;
    let Ok((mut player, player_transform, mut player_sprite, mut player_animation)) =
        player_query.single_mut()
    else {
        return;
    };
    let dt = time.delta_secs().min(0.05);
    let sword = &data.gameplay.player.sword;
    player.sword_cooldown = (player.sword_cooldown - dt).max(0.0);
    player.chain_reaction_timer = (player.chain_reaction_timer - dt).max(0.0);
    player.sated_fury_timer = (player.sated_fury_timer - dt).max(0.0);
    player.rhythm_timer = (player.rhythm_timer - dt).max(0.0);
    if player.rhythm_timer == 0.0 {
        player.rhythm_stacks = 0;
    }

    // A press during the previous swing is kept and fires as soon as the sword
    // is ready, so spamming J always continues the chain.
    if controls.just_pressed(GameAction::Sword, &keys) {
        player.sword_buffered = true;
    }
    let swing = player.sword_buffered && player.sword_cooldown == 0.0;
    // The sword is ready and nothing is queued: the player stopped attacking.
    if player.sword_cooldown == 0.0 && player.combo_step > 0 && !swing {
        player.combo_step = 0;
    }

    if swing {
        player.sword_buffered = false;
        let multipliers = &sword.combo.damage_multipliers;
        let step = player.combo_step.min(multipliers.len() - 1);
        let finisher = step + 1 == multipliers.len();
        let sated_fury = if player.sated_fury_timer > 0.0 {
            data.bonus_effect("sated_fury", "healing_pickup_attack_speed_multiplier")
        } else {
            1.0
        };
        let rhythm = 1.0
            + f32::from(player.rhythm_stacks)
                * data.bonus_effect("rhythm_training", "attack_speed_step");
        let attack_speed = bonus_multiplier(
            &session,
            &data,
            "quick_cut",
            "sword_attack_speed_multiplier",
        ) * sated_fury
            * rhythm;
        player.combo_step = if finisher { 0 } else { step + 1 };
        // Combo Flow speeds up the wait before each follow-up hit.
        let follow_up_speed = if finisher {
            1.0
        } else {
            bonus_multiplier(
                &session,
                &data,
                "combo_flow",
                "follow_up_attack_speed_multiplier",
            )
        };
        player.sword_cooldown = sword.attack_interval_s / (attack_speed * follow_up_speed);

        let finisher_bonus = if finisher {
            bonus_multiplier(
                &session,
                &data,
                "keen_finisher",
                "finisher_damage_multiplier",
            )
        } else {
            1.0
        };
        let chain_reaction =
            if player.chain_reaction_timer > 0.0 && has_bonus(&session, "chain_reaction") {
                1.0 + data.bonus_effect("chain_reaction", "kill_damage_bonus")
            } else {
                1.0
            };
        let airborne = if player.grounded {
            1.0
        } else {
            bonus_multiplier(
                &session,
                &data,
                "aerial_edge",
                "airborne_sword_damage_multiplier",
            )
        };
        let dash_strike = if player.dash_strike_timer > 0.0 {
            bonus_multiplier(
                &session,
                &data,
                "dash_strike",
                "post_dash_sword_damage_multiplier",
            )
        } else {
            1.0
        };
        let swing_damage = sword.damage
            * multipliers[step]
            * finisher_bonus
            * bonus_multiplier(&session, &data, "honed_edge", "sword_damage_multiplier")
            * chain_reaction
            * airborne
            * dash_strike;
        let range = sword.range_h
            * bonus_multiplier(&session, &data, "long_reach", "sword_range_multiplier")
            * H_WORLD_UNITS;

        let mut total_healing = 0.0;
        let mut hit_any = false;
        for (entity, mut enemy, mut transform, mut sprite, mut animation) in &mut enemy_query {
            if enemy.health <= 0.0 {
                continue;
            }
            let dx = transform.translation.x - player_transform.translation.x;
            let dy = (transform.translation.y - player_transform.translation.y).abs();
            if dx * player.facing < 0.0 || dx.abs() > range || dy > H_WORLD_UNITS * 1.25 {
                continue;
            }
            let marked = if enemy.marked_timer > 0.0 {
                bonus_multiplier(
                    &session,
                    &data,
                    "marked_prey",
                    "marked_sword_damage_multiplier",
                )
            } else {
                1.0
            };
            let damage = swing_damage * marked;
            enemy.health = (enemy.health - damage).max(0.0);
            spawn_damage_number(&mut commands, transform.translation, damage, false);
            if enemy.type_id == "evil_wizard" {
                enemy.knockback_velocity = player.facing * sword.boss_knockback_h_s;
                transform.translation.y = enemy.home_y;
            }
            enemy.marked_timer = data.bonus_effect("marked_prey", "mark_duration_s");
            if data.enemies.damage_triggers_combat {
                enemy.engaged = true;
            }
            enemy.windup_remaining = 0.0;
            enemy.combo_hit_pending = false;
            let style = audio::EnemySoundStyle::from(&enemy.type_id);
            if enemy.health == 0.0 {
                sounds.push(SoundCue::EnemyDeath(style));
                set_enemy_animation(&mut sprite, &mut animation, &sheets, &enemy.type_id, "dead");
                commands.entity(entity).insert(DeadEnemy {
                    timer: Timer::from_seconds(0.9, TimerMode::Once),
                });
                let drop_chance = data.gameplay.health_pickup.base_drop_chance
                    * bonus_multiplier(
                        &session,
                        &data,
                        "scavenger",
                        "health_drop_chance_multiplier",
                    );
                if random.chance(drop_chance.min(1.0)) {
                    commands.spawn((
                        Sprite::from_color(Color::srgb(0.28, 0.95, 0.48), Vec2::splat(18.0)),
                        Transform::from_translation(
                            transform.translation + Vec3::new(0.0, 20.0, 0.8),
                        ),
                        HealthPickup {
                            healing: data.gameplay.health_pickup.healing,
                        },
                        RoomEntity,
                    ));
                }
                player.chain_reaction_timer =
                    data.bonus_effect("chain_reaction", "kill_damage_duration_s");
            } else {
                sounds.push(SoundCue::EnemyHurt(style));
                enemy.hurt_timer = 0.24;
                set_enemy_animation(&mut sprite, &mut animation, &sheets, &enemy.type_id, "hurt");
            }
            hit_any = true;
            if has_bonus(&session, "vampiric_edge") {
                total_healing += data.bonus_effect("vampiric_edge", "sword_hit_healing");
            }
        }
        if has_bonus(&session, "dash_strike") {
            player.dash_strike_timer = 0.0;
        }
        if hit_any {
            sounds.push(if finisher {
                SoundCue::SwordFinisherHit
            } else {
                SoundCue::SwordHit
            });
            if has_bonus(&session, "rhythm_training") {
                if player.rhythm_timer > 0.0 {
                    let maximum = data.bonus_effect("rhythm_training", "attack_speed_max_bonus");
                    let step = data.bonus_effect("rhythm_training", "attack_speed_step");
                    player.rhythm_stacks = player
                        .rhythm_stacks
                        .saturating_add(1)
                        .min((maximum / step) as u8);
                }
                player.rhythm_timer = data.bonus_effect("rhythm_training", "hit_window_s");
            }
            let cap = data.bonus_effect("vampiric_edge", "sword_swing_healing_cap");
            session.health = (session.health + total_healing.min(cap)).min(session.max_health);
        } else {
            sounds.push(SoundCue::SwordSwing);
        }
        // Each combo hit plays its own clip from the first frame, and holds it
        // over the movement animations until it finishes.
        set_animation(
            &mut player_sprite,
            &mut player_animation,
            &sheets,
            &format!("player_sword_combo_{}", step + 1),
            false,
        );
        player_animation.frame = 0;
        if let Some(atlas) = player_sprite.texture_atlas.as_mut() {
            atlas.index = 0;
        }
        player_animation.timer.reset();
        player.action_animation_timer =
            player_animation.frames as f32 * player_animation.timer.duration().as_secs_f32();
    }
}
