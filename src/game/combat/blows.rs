//! Landing a blow on whatever it reaches. The melee chain, light and heavy,
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
/// set a heavy blow apart from the chain's light ones.
pub(super) struct Blow {
    pub(super) area: Rect,
    /// Where the player stands, which is what a body it hits is shoved away
    /// from.
    pub(super) from_x: f32,
    /// Shakes the camera harder: the chain's last blow, a heavy blow and
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
    /// The bonus that throws this blow or shapes it (a finisher step,
    /// Aftershock's shock, Meteor's or Breaker's slam, Overcharge's heavy
    /// blow, Battering Ram's throw), called out when it connects.
    pub(super) bonus: Option<String>,
    /// How many times heavier the bonuses that threw it already made
    /// `damage`, so its damage number is drawn as boosted as it is.
    pub(super) boost: f32,
}

/// The drag of a blow on every enemy within `radius_h` of where it was
/// thrown from, toward that spot at `speed_h_s`.
#[derive(Clone, Copy)]
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
    // Riposte and Retaliation leave the next blow empowered, and this is it
    // for every body it reaches; it is spent once it connects.
    let empowered = player.empowered.map_or(1.0, |(_, multiplier)| multiplier);
    let at = |transform: &Transform| TellAt::Spot(transform.translation);
    // The keystones that hit harder: Arsenal for every combat bonus owned,
    // Harmony for every combo.
    let keystones =
        (1.0 + session.keystone(data, "arsenal")) * (1.0 + session.keystone(data, "harmony"));

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
        if unaware > 1.0 {
            commands.write_message(BonusTell::new("opportunist", at(&enemy_transform)));
        }
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
        let was_marked = enemy.marked_timer > 0.0 && session.has_bonus("marked_prey");
        let marked = if enemy.marked_timer > 0.0 {
            session.bonus_multiplier(data, "marked_prey", "marked_strike_damage_multiplier")
        } else {
            1.0
        };
        enemy.marked_timer = session.bonus_effect(data, "marked_prey", "mark_duration_s");
        // Armor that is up when the blow lands costs it a layer, and keeps
        // the enemy at what it was doing: no flinch, no knockback, and a
        // wind-up carries on. An ordinary blow is soaked whole on top of
        // that; a heavy one goes through and wounds the body behind.
        // `shield.rs` picks the break up from `armor` going down.
        let armored = enemy.armor > 0;
        if armored {
            sounds.push(SoundCue::Damage(DamageSound::of(enemy.material, true)));
            enemy.armor -= 1;
            // Shield Breaker lets an ordinary blow through the layer it
            // broke, as a heavy one always goes.
            if !blow.pierces_shields && !session.has_bonus("shield_breaker") {
                continue;
            }
            // A blow a bonus let through the layer shows it punching through.
            let pierced_by = if blow.pierces_shields {
                blow.bonus.as_deref().filter(|bonus| *bonus == "breaker")
            } else {
                Some("shield_breaker")
            };
            if let Some(bonus) = pierced_by {
                commands.write_message(
                    BonusTell::new(bonus, at(&enemy_transform)).with(
                        TellEffect::new("pierce", 0.7)
                            .flipped(enemy_transform.translation.x < blow.from_x),
                    ),
                );
            }
        }
        // Open Wound raises Executioner's line on a marked body.
        let execute_below = if was_marked && session.has_bonus("open_wound") {
            session.bonus_effect(data, "open_wound", "marked_execute_health_fraction")
        } else {
            0.0
        }
        .max(session.bonus_effect(data, "executioner", "execute_health_fraction"));
        let executioner = if enemy.health / enemy.max_health < execute_below {
            session.bonus_multiplier(data, "executioner", "execute_damage_multiplier")
        } else {
            1.0
        };
        if executioner > 1.0 {
            // Called out by the bonus that put the body under the line.
            let by = if enemy.health / enemy.max_health
                < session.bonus_effect(data, "executioner", "execute_health_fraction")
            {
                "executioner"
            } else {
                "open_wound"
            };
            commands.write_message(BonusTell::new(by, at(&enemy_transform)));
        }
        let finisher = if blow.finisher {
            session.bonus_multiplier(data, "finisher", "finisher_damage_multiplier")
        } else {
            1.0
        };
        if finisher > 1.0 {
            commands.write_message(BonusTell::new("finisher", at(&enemy_transform)));
        }
        let chain_reaction =
            if player.chain_reaction_timer > 0.0 && session.has_bonus("chain_reaction") {
                1.0 + session.bonus_effect(data, "chain_reaction", "kill_damage_bonus")
            } else {
                1.0
            };
        let airborne = if player.grounded {
            1.0
        } else {
            session.bonus_multiplier(data, "aerial_assault", "airborne_strike_damage_multiplier")
        };
        if airborne > 1.0 {
            commands.write_message(BonusTell::new("aerial_assault", at(&enemy_transform)));
        }
        let boost = session.bonus_multiplier(data, "heavy_hands", "strike_damage_multiplier")
            * marked
            * finisher
            * chain_reaction
            * airborne
            * unaware
            * executioner
            * empowered
            * keystones;
        let damage = blow.damage * boost;
        enemy.health = (enemy.health - damage).max(0.0);
        spawn_damage_number(
            commands,
            &data.gameplay.effects.tells,
            enemy_transform.translation,
            Number::Dealt {
                damage,
                boost: boost * blow.boost,
            },
        );
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
            let profile = &data.enemies.types[&enemy.type_id];
            let middle = enemy.hurtbox(&enemy_transform).center();
            if let Some(blast) = &profile.death_blast {
                Hazard::death_blast(blast, middle).lay(commands);
            }
            if let Some(effect) = &profile.death_effect {
                play_effect(
                    commands,
                    sheets,
                    &effect.sheet,
                    middle.extend(7.0),
                    effect.size_h,
                    false,
                );
            }
            let item = &data.gameplay.health_pickup;
            let drop_chance = item.base_drop_chance
                * session.bonus_multiplier(data, "scavenger", "health_drop_chance_multiplier");
            let roll = random.roll();
            // A drop only Scavenger's better odds let fall says so.
            if roll < drop_chance.min(1.0) && roll >= item.base_drop_chance {
                commands.write_message(
                    BonusTell::new("scavenger", TellAt::Spot(middle.extend(0.0)))
                        .with(TellEffect::new("glint", 0.6)),
                );
            }
            if roll < drop_chance.min(1.0) {
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
                            enemy_transform.translation + Vec3::new(0.0, 0.0, 0.8),
                        ),
                        HealthPickup {
                            healing: item.healing,
                            fall_speed: 0.0,
                        },
                        Glowing {
                            color: Color::from(data.gameplay.effects.glow.health_color),
                            radius_h: data.gameplay.effects.glow.health_radius_h,
                            brightness: data.gameplay.effects.glow.health_brightness,
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
                session.bonus_effect(data, "chain_reaction", "kill_damage_duration_s");
            if session.has_bonus("chain_reaction") {
                commands.write_message(BonusTell::new("chain_reaction", TellAt::Player));
            }
            if was_marked && session.has_bonus("hunters_feast") {
                let healed = session.bonus_effect(data, "hunters_feast", "marked_kill_healing");
                session.health = (session.health + healed).min(session.max_health);
                commands.write_message(
                    BonusTell::new("hunters_feast", TellAt::Player)
                        .healing(healed)
                        .drawn_from(middle.extend(0.0)),
                );
            }
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
    if let Some((by, _)) = player.empowered.take() {
        commands.write_message(BonusTell::new(
            by,
            TellAt::Spot(blow.area.center().extend(0.0)),
        ));
    }
    if let Some(bonus) = &blow.bonus {
        commands.write_message(BonusTell::new(
            bonus.clone(),
            TellAt::Spot(blow.area.center().extend(0.0)),
        ));
    }
    // Updraft: a blow landed in mid-air hands back the mid-air jumps.
    let takeoff = data.gameplay.player.jumps_per_takeoff;
    if !player.grounded && session.has_bonus("updraft") && player.jumps_used > takeoff {
        player.jumps_used = takeoff;
        commands.write_message(
            BonusTell::new("updraft", TellAt::Player)
                .with(TellEffect::new("air_burst", 0.7).at_the_feet()),
        );
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
                    bonus: Some("battering_ram".to_owned()),
                    boost: 1.0,
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
            let maximum = session.bonus_effect(data, "rhythm_training", "combo_speed_max_bonus");
            let step = session.bonus_effect(data, "rhythm_training", "combo_speed_step");
            player.rhythm_stacks = player
                .rhythm_stacks
                .saturating_add(1)
                .min((maximum / step) as u8);
        }
        player.rhythm_timer = session.bonus_effect(data, "rhythm_training", "hit_window_s");
    }
    if session.has_bonus("lifesteal") {
        // Bloodlust heals harder and lets more of it through each room.
        let (cap, healing) = if session.has_bonus("bloodlust") {
            (
                session.bonus_effect(data, "bloodlust", "room_healing_cap"),
                session.bonus_effect(data, "bloodlust", "strike_hit_healing_multiplier"),
            )
        } else {
            (
                session.bonus_effect(data, "lifesteal", "room_healing_cap"),
                1.0,
            )
        };
        let healing = (session.bonus_effect(data, "lifesteal", "strike_hit_healing") * healing)
            .min(cap - player.lifesteal_healing)
            .max(0.0);
        player.lifesteal_healing += healing;
        session.health = (session.health + healing).min(session.max_health);
        if healing > 0.0 {
            let by = if session.has_bonus("bloodlust") {
                "bloodlust"
            } else {
                "lifesteal"
            };
            commands.write_message(
                BonusTell::new(by, TellAt::Player)
                    .quiet()
                    .healing(healing)
                    .drawn_from(blow.area.center().extend(0.0)),
            );
        }
    }
}
