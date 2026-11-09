use super::*;

type MeleeEnemiesQuery<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static mut EnemyBody,
        &'static mut Transform,
        &'static mut Sprite,
        &'static mut AnimatedSprite,
    ),
    (Without<PlayerBody>, Without<DeadEnemy>),
>;

#[derive(SystemParam)]
pub(in crate::game) struct MeleeResources<'w> {
    time: Res<'w, Time>,
    keys: Res<'w, ButtonInput<KeyCode>>,
    controls: Res<'w, ControlBindings>,
    data: Res<'w, GameData>,
    session: ResMut<'w, RunSession>,
    random: ResMut<'w, RandomState>,
    sheets: Res<'w, SpriteSheets>,
    sounds: ResMut<'w, SoundQueue>,
}

/// The player's only weapon: a chain of blows. Each press of Attack lands the
/// next step of `player.melee` of `config/gameplay.yaml`, shoving the player
/// forward as it goes; the blow itself connects partway through its clip, and
/// once it ends there is `combo_window_s` to ask for the next before the chain
/// drops back to its first step. A step naming a bonus is only in the chain
/// while that bonus is owned, which is how Fire Kick lengthens it.
pub(in crate::game) fn player_melee(
    mut commands: Commands,
    resources: MeleeResources,
    mut players: Query<(&mut PlayerBody, &Transform), Without<EnemyBody>>,
    mut enemies: MeleeEnemiesQuery,
) {
    let MeleeResources {
        time,
        keys,
        controls,
        data,
        mut session,
        mut random,
        sheets,
        mut sounds,
    } = resources;
    let Ok((mut player, transform)) = players.single_mut() else {
        return;
    };
    let dt = time.delta_secs().min(0.05);
    let melee = &data.gameplay.player.melee;
    player.chain_reaction_timer = (player.chain_reaction_timer - dt).max(0.0);
    player.sated_fury_timer = (player.sated_fury_timer - dt).max(0.0);
    player.rhythm_timer = (player.rhythm_timer - dt).max(0.0);
    if player.rhythm_timer == 0.0 {
        player.rhythm_stacks = 0;
    }
    if player.dying_s.is_some() {
        player.strike = None;
        return;
    }

    // The chain this run can actually throw: every step whose bonus is owned.
    let chain = melee
        .steps
        .iter()
        .filter(|step| {
            step.requires_bonus
                .as_ref()
                .is_none_or(|bonus| session.has_bonus(bonus))
        })
        .collect::<Vec<_>>();
    if chain.is_empty() {
        return;
    }

    // No blow under way: the window to chain closes on its own, and Attack
    // throws the next step.
    let Some(strike) = player.strike.as_mut() else {
        player.combo_window_s = (player.combo_window_s - dt).max(0.0);
        if player.combo_window_s == 0.0 {
            player.combo_step = 0;
        }
        let attacking = controls.just_pressed(GameAction::Attack, &keys)
            && !player.covering
            && !player.swimming
            && player.wall_cling.is_none();
        if !attacking {
            return;
        }
        let step_index = player.combo_step.min(chain.len() - 1);
        let step = chain[step_index];
        // Rhythm, and the fury of a fresh pickup, speed the whole chain up.
        let rhythm = 1.0
            + f32::from(player.rhythm_stacks)
                * data.bonus_effect("rhythm_training", "combo_speed_step");
        let sated_fury = if player.sated_fury_timer > 0.0 {
            data.bonus_effect("sated_fury", "healing_pickup_combo_speed_multiplier")
        } else {
            1.0
        };
        let speed = session.bonus_multiplier(&data, "quick_hands", "combo_speed_multiplier")
            * rhythm
            * sated_fury;
        let duration_s = step.strike.clip(&data.player_animations.clips).duration_s() / speed;
        player.strike = Some(Strike {
            step: step_index,
            contact_in_s: step.contact_s / speed,
        });
        player.combo_step = (step_index + 1) % chain.len();
        player.combo_window_s = 0.0;
        player.velocity.x = player.facing * step.lunge_h_s * H_WORLD_UNITS;
        player.start_action(
            PlayerAction::Strike {
                strike: step.strike,
                speed,
            },
            duration_s,
        );
        sounds.push(SoundCue::MeleeSwing);
        return;
    };

    // A blow is under way. It plays itself out, and the window to chain the
    // next one opens as its clip ends.
    let pending = strike.contact_in_s > 0.0;
    strike.contact_in_s -= dt;
    let connects = pending && strike.contact_in_s <= 0.0;
    let step = chain[strike.step.min(chain.len() - 1)];
    if !matches!(player.action, Some(PlayerAction::Strike { .. })) {
        player.strike = None;
        player.combo_window_s = melee.combo_window_s
            * session.bonus_multiplier(&data, "deep_breath", "combo_window_multiplier");
        return;
    }
    if !connects {
        return;
    }

    // The blow lands: everything its hitbox reaches is hurt, so one kick can
    // catch two bodies standing together.
    let feet = Vec2::new(
        transform.translation.x,
        transform.translation.y - 0.5 * H_WORLD_UNITS,
    );
    let reach = session.bonus_multiplier(&data, "long_reach", "strike_reach_multiplier");
    let blow = step.hitbox_at(feet, player.facing, reach);
    // The chain has already stepped on, so a chain that wrapped back to its
    // first step means this blow was its last.
    let finisher = player.combo_step == 0;
    let mut connected = false;

    for (entity, mut enemy, mut enemy_transform, mut sprite, mut animation) in enemies.iter_mut() {
        if enemy.health <= 0.0 || enemy.hurtbox(&enemy_transform).intersect(blow).is_empty() {
            continue;
        }
        connected = true;
        if data.enemies.damage_triggers_combat {
            enemy.engaged = true;
        }
        enemy.marked_timer = data.bonus_effect("marked_prey", "mark_duration_s");
        if enemy.shielded {
            // The shield soaks the blow whole: it shatters, and nothing else
            // about the enemy changes. `shield.rs` picks the break up from
            // `shielded` going false.
            sounds.push(SoundCue::Damage(DamageSound::of(enemy.material, true)));
            enemy.shielded = false;
            continue;
        }
        let marked = if enemy.marked_timer > 0.0 {
            session.bonus_multiplier(&data, "marked_prey", "marked_strike_damage_multiplier")
        } else {
            1.0
        };
        let finisher_bonus = if finisher {
            session.bonus_multiplier(&data, "finisher", "finisher_damage_multiplier")
        } else {
            1.0
        };
        let chain_reaction =
            if player.chain_reaction_timer > 0.0 && session.has_bonus("chain_reaction") {
                1.0 + data.bonus_effect("chain_reaction", "kill_damage_bonus")
            } else {
                1.0
            };
        let airborne = if player.grounded {
            1.0
        } else {
            session.bonus_multiplier(&data, "aerial_assault", "airborne_strike_damage_multiplier")
        };
        let damage = step.damage
            * session.bonus_multiplier(&data, "heavy_hands", "strike_damage_multiplier")
            * marked
            * finisher_bonus
            * chain_reaction
            * airborne;
        enemy.health = (enemy.health - damage).max(0.0);
        spawn_damage_number(&mut commands, enemy_transform.translation, damage, false);
        if enemy.type_id == "emitter_unit" {
            enemy.knockback_velocity = player.facing * melee.boss_knockback_h_s;
            enemy_transform.translation.y = enemy.home_y;
        }
        enemy.windup_remaining = 0.0;
        enemy.combo_hit_pending = false;
        let style = audio::EnemySoundStyle::from(&enemy.type_id);
        if enemy.health > 0.0 {
            sounds.push(SoundCue::Damage(DamageSound::of(enemy.material, false)));
            enemy.hurt_timer = 0.24;
            set_enemy_animation(&mut sprite, &mut animation, &sheets, &enemy.type_id, "hurt");
        } else {
            sounds.push(SoundCue::EnemyDeath(style));
            set_enemy_animation(&mut sprite, &mut animation, &sheets, &enemy.type_id, "dead");
            commands.entity(entity).insert(DeadEnemy {
                timer: Timer::from_seconds(0.9, TimerMode::Once),
            });
            let item = &data.gameplay.health_pickup;
            let drop_chance = item.base_drop_chance
                * session.bonus_multiplier(&data, "scavenger", "health_drop_chance_multiplier");
            if random.chance(drop_chance.min(1.0)) {
                let mut pickup = Sprite::from_image(
                    sheets
                        .0
                        .get(HealthPickup::SHEET)
                        .map(|sheet| sheet.image.clone())
                        .unwrap_or_default(),
                );
                pickup.custom_size = Some(Vec2::splat(item.size_h * H_WORLD_UNITS));
                commands.spawn((
                    pickup,
                    Transform::from_translation(
                        enemy_transform.translation + Vec3::new(0.0, 20.0, 0.8),
                    ),
                    HealthPickup {
                        healing: item.healing,
                    },
                    Glowing {
                        color: Color::from(data.gameplay.effects.glow.health_color),
                        radius_h: data.gameplay.effects.glow.health_radius_h,
                    },
                    RoomEntity,
                ));
            }
            player.chain_reaction_timer =
                data.bonus_effect("chain_reaction", "kill_damage_duration_s");
        }
    }

    if !connected {
        return;
    }
    spawn_fire_burst(
        &mut commands,
        &mut random,
        blow.center(),
        Vec2::new(-player.facing, 0.0),
        &melee.impact_burst,
    );
    sounds.push(SoundCue::MeleeHit);
    if session.has_bonus("rhythm_training") {
        if player.rhythm_timer > 0.0 {
            let maximum = data.bonus_effect("rhythm_training", "combo_speed_max_bonus");
            let step = data.bonus_effect("rhythm_training", "combo_speed_step");
            player.rhythm_stacks = player
                .rhythm_stacks
                .saturating_add(1)
                .min((maximum / step) as u8);
        }
        player.rhythm_timer = data.bonus_effect("rhythm_training", "hit_window_s");
    }
    if session.has_bonus("lifesteal") {
        let cap = data.bonus_effect("lifesteal", "room_healing_cap");
        let healing = data
            .bonus_effect("lifesteal", "strike_hit_healing")
            .min(cap - player.lifesteal_healing)
            .max(0.0);
        player.lifesteal_healing += healing;
        session.health = (session.health + healing).min(session.max_health);
    }
}
