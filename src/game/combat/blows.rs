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
}

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

    for (entity, mut enemy, mut enemy_transform, mut sprite, mut animation) in enemies.iter_mut() {
        if enemy.health <= 0.0
            || enemy
                .hurtbox(&enemy_transform)
                .intersect(blow.area)
                .is_empty()
        {
            continue;
        }
        connected = true;
        if data.enemies.damage_triggers_combat {
            enemy.engaged = true;
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
            if !blow.pierces_shields {
                continue;
            }
        }
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
            * airborne;
        enemy.health = (enemy.health - damage).max(0.0);
        spawn_damage_number(commands, enemy_transform.translation, damage, false);
        if !armored {
            // Shoved away from the player, whichever side of them it stood.
            let away = if enemy_transform.translation.x == blow.from_x {
                player.facing
            } else {
                (enemy_transform.translation.x - blow.from_x).signum()
            };
            if enemy.type_id == "emitter_unit" {
                enemy.knockback_velocity = away * melee.boss_knockback_h_s;
                enemy_transform.translation.y = enemy.home_y;
            } else {
                enemy.knockback_velocity = away * melee.knockback_h_s;
            }
            enemy.windup_remaining = 0.0;
            enemy.combo_hit_pending = false;
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

    if !connected {
        if blow.whiffs {
            sounds.push(SoundCue::MeleeMiss);
        }
        return;
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
    if let Some(sheet) = sheets.0.get(key) {
        let mut sprite = Sprite::from_atlas_image(
            sheet.image.clone(),
            TextureAtlas {
                layout: sheet.layout.clone(),
                index: 0,
            },
        );
        sprite.custom_size = Some(sheet.frame_size * size_h * H_WORLD_UNITS / sheet.frame_size.y);
        sprite.flip_x = player.facing < 0.0;
        commands.spawn((
            sprite,
            Transform::from_translation(blow.area.center().extend(6.0)),
            AnimatedSprite {
                sheet: key.to_string(),
                frame: 0,
                frames: sheet.frames,
                timer: Timer::from_seconds(1.0 / sheet.fps, TimerMode::Repeating),
                looping: false,
            },
            HitEffect {
                timer: Timer::from_seconds(sheet.frames as f32 / sheet.fps, TimerMode::Once),
            },
            RoomEntity,
        ));
    }
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
