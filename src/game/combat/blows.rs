//! Landing a blow on whatever it reaches. The melee chain, the charged blow
//! and the airborne slam all come through here, so a hit costs the same and
//! sounds the same however it was thrown.

use super::*;

pub(super) type MeleeEnemiesQuery<'w, 's> = Query<
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

/// A blow arriving: where it reaches, what it costs, and the two things that
/// set the charged blow apart from the chain's own.
pub(super) struct Blow {
    pub(super) area: Rect,
    /// Where the player stands, which is what a body it hits is shoved away
    /// from.
    pub(super) from_x: f32,
    /// Shakes the camera harder: the chain's last blow, the charged blow and
    /// the slam.
    pub(super) heavy: bool,
    pub(super) damage: f32,
    /// Goes through armor rather than being soaked by it, so the hit both
    /// breaks a layer and wounds what was behind it.
    pub(super) pierces_shields: bool,
    /// How long the bodies it hits are held in their flinch.
    pub(super) flinch_s: f32,
    /// The chain's last blow, which the Finisher bonus pays for.
    pub(super) finisher: bool,
    /// Which of `audio.recorded_effects.punch_hits` it lands with; one past
    /// the end of that list lands with the last.
    pub(super) punch: usize,
    /// Heard cutting the air when it reaches nobody. The slam is not: it
    /// lands on the floor either way.
    pub(super) whiffs: bool,
    /// Drags the enemies around it in, whether or not it reached them.
    pub(super) pull: Option<Pull>,
    /// Throws the bodies it wounds into whatever stands behind them.
    pub(super) launch: Option<Launch>,
}

/// The drag of a blow on every enemy within `radius_h` of where it was
/// thrown from, toward that spot at `speed_h_s`.
pub(super) struct Pull {
    pub(super) radius_h: f32,
    pub(super) speed_h_s: f32,
}

/// What Battering Ram adds to a blow: a body it wounds is shoved
/// `knockback_multiplier` times harder, and whatever stands within `reach_h`
/// behind it is hit for `damage`.
pub(super) struct Launch {
    pub(super) damage: f32,
    pub(super) reach_h: f32,
    pub(super) knockback_multiplier: f32,
}

/// How fast a shoved body slows down, in H a second each second.
pub(super) const KNOCKBACK_DRAG_H_S2: f32 = 14.0;

pub(super) struct LandingResources<'a, 'w, 's> {
    pub(super) commands: &'a mut Commands<'w, 's>,
    pub(super) data: &'a GameData,
    pub(super) session: &'a mut RunSession,
    pub(super) random: &'a mut RandomState,
    pub(super) sheets: &'a SpriteSheets,
    pub(super) sounds: &'a mut SoundQueue,
}

/// Hurts every enemy the blow reaches — so one swing can catch two bodies
/// standing together — and pays out everything a landed hit owes the run.
pub(super) fn land_the_blow(
    resources: LandingResources<'_, '_, '_>,
    player: &mut PlayerBody,
    blow: Blow,
    enemies: &mut MeleeEnemiesQuery,
) {
    let LandingResources {
        commands,
        data,
        session,
        random,
        sheets,
        sounds,
    } = resources;
    let melee = &data.gameplay.player.melee;
    let mut connected = false;
    let mut killed = false;
    // Where the blow landed, for the idle enemies standing near enough to hear it.
    let mut struck = Vec::new();
    // Behind each body the blow threw: where the throw lands in its turn.
    let mut thrown = Vec::new();

    for (entity, mut enemy, mut enemy_transform, mut sprite, mut animation) in enemies.iter_mut() {
        if enemy.health <= 0.0
            || enemy
                .hurtbox(&enemy_transform)
                .intersect(blow.area)
                .is_empty()
        {
            continue;
        }
        // Nothing reaches a body through the middle of its roll.
        if let Some((Stance::Roll { .. }, left_s)) = enemy.stance
            && let Some(roll) = &data.enemies.types[&enemy.type_id].roll
            && (roll.invulnerable_from_s..=roll.invulnerable_to_s)
                .contains(&(roll.duration_s - left_s))
        {
            continue;
        }
        connected = true;
        // Opportunist pays for a blow on a body that had not noticed anyone.
        let unaware = if enemy.engaged {
            1.0
        } else {
            session.bonus_multiplier(data, "opportunist", "unaware_damage_multiplier")
        };
        if data.enemies.damage_triggers_combat {
            enemy.engaged = true;
            struck.push(enemy_transform.translation.truncate());
        }
        // A guard turns an ordinary blow aside, at no cost in health or
        // armor, and answers it; the answer is not stopped by a second blow
        // either. Only a blow that goes through armor breaks a guard, and
        // lands as it would on a body that never raised one.
        let guarding = matches!(enemy.stance, Some((Stance::Guard, _)));
        if guarding || enemy.countering {
            if !blow.pierces_shields {
                sounds.push(SoundCue::Damage(DamageSound::of(enemy.material, true)));
                if guarding
                    && let Some(guard) = enemy
                        .attacks
                        .get(enemy.active_attack)
                        .and_then(|attack| attack.guard.clone())
                {
                    enemy.stance = None;
                    enemy.countering = true;
                    enemy.windup_remaining = guard.counter_windup_s;
                    set_enemy_animation(
                        &mut sprite,
                        &mut animation,
                        sheets,
                        &enemy.type_id,
                        &guard.counter_clip,
                    );
                }
                continue;
            }
            enemy.stance = None;
        }
        let marked = if enemy.marked_timer > 0.0 {
            session.bonus_multiplier(data, "marked_prey", "marked_strike_damage_multiplier")
        } else {
            1.0
        };
        enemy.marked_timer = data.bonus_effect("marked_prey", "mark_duration_s");
        // Armor that is up when the blow lands costs it a layer, and keeps
        // the enemy at what it was doing: no flinch, no knockback, and a
        // wind-up carries on. An ordinary blow is soaked whole on top of
        // that; a charged one goes through and wounds the body behind.
        // `shield.rs` picks the break up from `armor` going down.
        let armored = enemy.armor > 0;
        if armored {
            sounds.push(SoundCue::Damage(DamageSound::of(enemy.material, true)));
            enemy.armor -= 1;
            // Shield Breaker lets an ordinary blow through the layer it
            // broke, as the charged one always goes.
            if !blow.pierces_shields && !session.has_bonus("shield_breaker") {
                continue;
            }
        }
        let executioner = if enemy.health / enemy.max_health
            < data.bonus_effect("executioner", "execute_health_fraction")
        {
            session.bonus_multiplier(data, "executioner", "execute_damage_multiplier")
        } else {
            1.0
        };
        let finisher = if blow.finisher {
            session.bonus_multiplier(data, "finisher", "finisher_damage_multiplier")
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
            session.bonus_multiplier(data, "aerial_assault", "airborne_strike_damage_multiplier")
        };
        let damage = blow.damage
            * session.bonus_multiplier(data, "heavy_hands", "strike_damage_multiplier")
            * marked
            * finisher
            * chain_reaction
            * airborne
            * unaware
            * executioner;
        enemy.health = (enemy.health - damage).max(0.0);
        spawn_damage_number(commands, enemy_transform.translation, damage, false);
        if !armored {
            // Shoved away from the player, whichever side of them it stood.
            let away = if enemy_transform.translation.x == blow.from_x {
                player.facing
            } else {
                (enemy_transform.translation.x - blow.from_x).signum()
            };
            if enemy.behavior == "boss" {
                enemy.knockback_velocity = away * melee.boss_knockback_h_s;
                enemy_transform.translation.y = enemy.home_y;
            } else {
                enemy.knockback_velocity = away * melee.knockback_h_s;
                if let Some(launch) = &blow.launch {
                    enemy.knockback_velocity *= launch.knockback_multiplier;
                    let body = enemy.hurtbox(&enemy_transform);
                    let near = if away < 0.0 { body.min.x } else { body.max.x };
                    thrown.push(Rect::new(
                        near,
                        body.min.y,
                        near + away * launch.reach_h * H_WORLD_UNITS,
                        body.max.y,
                    ));
                }
            }
            enemy.windup_remaining = 0.0;
            enemy.combo_hit_pending = false;
            enemy.stance = None;
            enemy.countering = false;
            enemy.wounds = enemy.wounds.saturating_add(1);
        }
        let style = audio::EnemySoundStyle::from(&enemy.type_id);
        if enemy.health > 0.0 {
            sounds.push(SoundCue::Damage(DamageSound::of(enemy.material, false)));
            // A wound flares on the body that survives it; `update_enemy_ai`
            // puts the flare out.
            enemy.flash_s = melee.impact.flash_s;
            sprite.color = Color::srgb(
                melee.impact.flash_brightness,
                melee.impact.flash_brightness,
                melee.impact.flash_brightness,
            );
            if !armored {
                enemy.hurt_timer = blow.flinch_s;
                set_enemy_animation(&mut sprite, &mut animation, sheets, &enemy.type_id, "hurt");
            }
        } else {
            killed = true;
            sounds.push(SoundCue::EnemyDeath(style));
            set_enemy_animation(&mut sprite, &mut animation, sheets, &enemy.type_id, "dead");
            commands.entity(entity).insert(DeadEnemy {
                timer: Timer::from_seconds(0.9, TimerMode::Once),
            });
            // A type that goes off as it dies warns where it fell, then
            // burns everything near it.
            if let Some(blast) = &data.enemies.types[&enemy.type_id].death_blast {
                let middle = enemy.hurtbox(&enemy_transform).center();
                Hazard {
                    area: Rect::from_center_half_size(
                        middle,
                        Vec2::splat(blast.radius_h * H_WORLD_UNITS),
                    ),
                    damage: blast.damage,
                    arm_s: blast.fuse_s,
                    burn_s: blast.burn_s,
                    rest_s: None,
                    elapsed_s: 0.0,
                }
                .lay(commands);
            }
            let item = &data.gameplay.health_pickup;
            let drop_chance = item.base_drop_chance
                * session.bonus_multiplier(data, "scavenger", "health_drop_chance_multiplier");
            if random.chance(drop_chance.min(1.0)) {
                let mut pickup = Sprite::from_image(
                    sheets
                        .0
                        .get(HealthPickup::SHEET)
                        .map(|sheet| sheet.image.clone())
                        .unwrap_or_default(),
                );
                pickup.custom_size = Some(Vec2::splat(item.size_h * H_WORLD_UNITS));
                let lying = commands
                    .spawn((
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
                    ))
                    .id();
                // The plusses drift off it for as long as it lies there, and
                // go with it when it is taken.
                if let Some(sheet) = sheets.0.get(HealEffect::SHEET) {
                    let mut plusses = Sprite::from_atlas_image(
                        sheet.image.clone(),
                        TextureAtlas {
                            layout: sheet.layout.clone(),
                            index: 0,
                        },
                    );
                    plusses.custom_size = Some(
                        sheet.frame_size * HealEffect::HEIGHT_H * H_WORLD_UNITS
                            / sheet.frame_size.y,
                    );
                    commands.entity(lying).with_child((
                        plusses,
                        Transform::from_xyz(0.0, 0.3 * H_WORLD_UNITS, 0.1),
                        AnimatedSprite {
                            sheet: HealEffect::SHEET.to_string(),
                            frame: 0,
                            frames: sheet.frames,
                            timer: Timer::from_seconds(1.0 / sheet.fps, TimerMode::Repeating),
                            looping: true,
                        },
                        HealEffect { timer: None },
                    ));
                }
            }
            player.chain_reaction_timer =
                data.bonus_effect("chain_reaction", "kill_damage_duration_s");
        }
    }

    // A blow that drags pulls in everything near it, struck or not, and
    // sets each down just short of where it was thrown from. A boss is not
    // moved.
    if let Some(pull) = &blow.pull {
        for (_, mut enemy, transform, _, _) in enemies.iter_mut() {
            let offset_h = (transform.translation.x - blow.from_x) / H_WORLD_UNITS;
            let level = (enemy.home_y - enemy.feet_offset - blow.area.min.y).abs();
            if enemy.health <= 0.0
                || enemy.behavior == "boss"
                || offset_h.abs() > pull.radius_h
                || level > 1.5 * H_WORLD_UNITS
            {
                continue;
            }
            let carry_h = (offset_h.abs() - 0.9).max(0.0);
            enemy.knockback_velocity = -offset_h.signum()
                * (2.0 * KNOCKBACK_DRAG_H_S2 * carry_h)
                    .sqrt()
                    .min(pull.speed_h_s);
        }
    }
    if !connected {
        if blow.whiffs {
            sounds.push(SoundCue::MeleeMiss);
        }
        return;
    }
    // A thrown body lands on whatever stood behind it, as a blow of its own.
    if let Some(launch) = &blow.launch {
        for area in thrown {
            land_the_blow(
                LandingResources {
                    commands: &mut *commands,
                    data,
                    session: &mut *session,
                    random: &mut *random,
                    sheets,
                    sounds: &mut *sounds,
                },
                player,
                Blow {
                    area,
                    from_x: blow.from_x,
                    heavy: false,
                    damage: launch.damage,
                    pierces_shields: false,
                    flinch_s: blow.flinch_s,
                    finisher: false,
                    punch: usize::MAX,
                    whiffs: false,
                    pull: None,
                    launch: None,
                },
                enemies,
            );
        }
    }
    // A blow is heard: every idle enemy near one it landed on turns on the
    // player, whether or not it could have seen them.
    let hearing_range = data.enemies.detection.hearing_range_h * H_WORLD_UNITS;
    for (_, mut enemy, transform, _, _) in enemies.iter_mut() {
        if !enemy.engaged
            && enemy.health > 0.0
            && struck
                .iter()
                .any(|spot| spot.distance(transform.translation.truncate()) <= hearing_range)
        {
            enemy.engaged = true;
        }
    }
    // The moment holds and the camera is thrown: longer for a kill, harder
    // for a heavy blow or a kill. A blow soaked by armor is felt too.
    commands.spawn(impact::Impact {
        freeze_s: if killed {
            melee.impact.kill_freeze_s
        } else {
            melee.impact.freeze_s
        },
        shake_h: if killed || blow.heavy {
            melee.impact.heavy_shake_h
        } else {
            melee.impact.shake_h
        },
    });
    // What is seen where it landed: sparks thrown the way the blow went, a
    // burst for a heavy one, a ring opening out of a flash for a kill.
    let effect = &melee.hit_effect;
    let (key, size_h) = if killed {
        ("effect_kill_ring", effect.kill_size_h)
    } else if blow.heavy {
        ("effect_hit_burst", effect.burst_size_h)
    } else {
        ("effect_hit_spark", effect.spark_size_h)
    };
    play_effect(
        commands,
        sheets,
        key,
        blow.area.center().extend(6.0),
        size_h,
        player.facing < 0.0,
    );
    let punches = data.gameplay.audio.recorded_effects.punch_hits.len();
    sounds.push(SoundCue::MeleeHit(
        blow.punch.min(punches.saturating_sub(1)),
    ));
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
