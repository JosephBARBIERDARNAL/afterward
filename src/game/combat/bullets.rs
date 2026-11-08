use super::*;

/// An axis-aligned box a bullet's path can run into.
struct HitBox {
    min: Vec2,
    max: Vec2,
}

impl HitBox {
    /// How far along a path of `length` from `start` toward unit `direction`
    /// the path enters the box, if it does.
    fn entry(&self, start: Vec2, direction: Dir2, length: f32) -> Option<f32> {
        let inverse = direction.recip();
        let to_min = (self.min - start) * inverse;
        let to_max = (self.max - start) * inverse;
        let near = to_min.min(to_max).max_element();
        let far = to_min.max(to_max).min_element();
        (near <= far && far >= 0.0 && near <= length).then_some(near.max(0.0))
    }
}

type BulletPlayerQuery<'w, 's> =
    Query<'w, 's, &'static mut PlayerBody, (Without<EnemyBody>, Without<PlayerBullet>)>;
type BulletEnemiesQuery<'w, 's> = Query<
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
        Without<PlayerBullet>,
        Without<DeadEnemy>,
    ),
>;

#[derive(SystemParam)]
pub(in crate::game) struct BulletResources<'w> {
    time: Res<'w, Time>,
    data: Res<'w, GameData>,
    session: ResMut<'w, RunSession>,
    random: ResMut<'w, RandomState>,
    sheets: Res<'w, SpriteSheets>,
    sounds: ResMut<'w, SoundQueue>,
}

/// Player bullets fly straight until they hit the first enemy or platform in
/// their path, leave the room, or run out of range. A hit damages the enemy
/// and bursts into flame.
pub(in crate::game) fn update_player_bullets(
    mut commands: Commands,
    resources: BulletResources,
    mut players: BulletPlayerQuery,
    mut bullets: Query<(Entity, &mut Transform, &mut PlayerBullet), Without<EnemyBody>>,
    mut enemies: BulletEnemiesQuery,
    platforms: Query<&PlatformSurface>,
) {
    let BulletResources {
        time,
        data,
        mut session,
        mut random,
        sheets,
        mut sounds,
    } = resources;
    let Ok(mut player) = players.single_mut() else {
        return;
    };
    let dt = time.delta_secs().min(0.05);
    let gun = &data.gameplay.player.gun;
    let room = &data.rooms[&session.room_id];
    let room_size = Vec2::new(room.size.width_h, room.size.height_h) * H_WORLD_UNITS;
    for (bullet_entity, mut transform, mut bullet) in &mut bullets {
        let Ok(direction) = Dir2::new(bullet.velocity) else {
            commands.entity(bullet_entity).despawn();
            continue;
        };
        let muzzle = transform.translation.truncate();
        let step = (bullet.velocity.length() * dt).min(bullet.remaining_range);
        bullet.remaining_range -= step;
        let start = bullet.fired_from.take().unwrap_or(muzzle);
        let travel = (muzzle - start).length() + step;
        let half_size = bullet.half_size;
        let platform_hit = platforms
            .iter()
            .filter_map(|block| {
                HitBox {
                    min: Vec2::new(block.left, block.bottom) - half_size,
                    max: Vec2::new(block.right, block.top) + half_size,
                }
                .entry(start, direction, travel)
            })
            .min_by(f32::total_cmp);
        let enemy_hit = enemies
            .iter()
            .filter(|(_, enemy, ..)| enemy.health > 0.0)
            .filter_map(|(entity, enemy, enemy_transform, ..)| {
                // The bullet's center hits once its edge touches the hurtbox.
                let hurtbox = enemy.hurtbox(enemy_transform);
                let bounds = HitBox {
                    min: hurtbox.min - half_size,
                    max: hurtbox.max + half_size,
                };
                bounds
                    .entry(start, direction, travel)
                    .map(|distance| (distance, entity))
            })
            .min_by(|a, b| a.0.total_cmp(&b.0));

        match (enemy_hit, platform_hit) {
            (Some((distance, enemy_entity)), platform)
                if platform.is_none_or(|blocked| distance <= blocked) =>
            {
                let point = start + *direction * distance;
                if let Ok((entity, mut enemy, mut enemy_transform, mut sprite, mut animation)) =
                    enemies.get_mut(enemy_entity)
                {
                    let marked = if enemy.marked_timer > 0.0 {
                        session.bonus_multiplier(
                            &data,
                            "marked_prey",
                            "marked_bullet_damage_multiplier",
                        )
                    } else {
                        1.0
                    };
                    enemy.marked_timer = data.bonus_effect("marked_prey", "mark_duration_s");
                    if data.enemies.damage_triggers_combat {
                        enemy.engaged = true;
                    }
                    if enemy.shielded {
                        // The shield soaks the hit whole: it shatters, and
                        // nothing else about the enemy changes. `shield.rs`
                        // picks the break up from `shielded` going false.
                        sounds.push(SoundCue::Damage(DamageSound::of(enemy.material, true)));
                        enemy.shielded = false;
                    } else {
                        let damage = bullet.damage * marked;
                        enemy.health = (enemy.health - damage).max(0.0);
                        spawn_damage_number(
                            &mut commands,
                            enemy_transform.translation,
                            damage,
                            false,
                        );
                        if enemy.type_id == "evil_wizard" {
                            enemy.knockback_velocity =
                                bullet.velocity.x.signum() * gun.boss_knockback_h_s;
                            enemy_transform.translation.y = enemy.home_y;
                        }
                        enemy.windup_remaining = 0.0;
                        enemy.combo_hit_pending = false;
                        let style = audio::EnemySoundStyle::from(&enemy.type_id);
                        if enemy.health > 0.0 {
                            sounds.push(SoundCue::Damage(DamageSound::of(enemy.material, false)));
                            enemy.hurt_timer = 0.24;
                            set_enemy_animation(
                                &mut sprite,
                                &mut animation,
                                &sheets,
                                &enemy.type_id,
                                "hurt",
                            );
                        } else {
                            sounds.push(SoundCue::EnemyDeath(style));
                            set_enemy_animation(
                                &mut sprite,
                                &mut animation,
                                &sheets,
                                &enemy.type_id,
                                "dead",
                            );
                            commands.entity(entity).insert(DeadEnemy {
                                timer: Timer::from_seconds(0.9, TimerMode::Once),
                            });
                            let item = &data.gameplay.health_pickup;
                            let drop_chance = item.base_drop_chance
                                * session.bonus_multiplier(
                                    &data,
                                    "scavenger",
                                    "health_drop_chance_multiplier",
                                );
                            if random.chance(drop_chance.min(1.0)) {
                                let mut sprite = Sprite::from_image(
                                    sheets
                                        .0
                                        .get(HealthPickup::SHEET)
                                        .map(|sheet| sheet.image.clone())
                                        .unwrap_or_default(),
                                );
                                sprite.custom_size = Some(Vec2::splat(item.size_h * H_WORLD_UNITS));
                                commands.spawn((
                                    sprite,
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
                }
                sounds.push(if bullet.last_round {
                    SoundCue::LastRoundHit
                } else {
                    SoundCue::BulletHit
                });
                if session.has_bonus("rhythm_training") {
                    if player.rhythm_timer > 0.0 {
                        let maximum = data.bonus_effect("rhythm_training", "fire_rate_max_bonus");
                        let step = data.bonus_effect("rhythm_training", "fire_rate_step");
                        player.rhythm_stacks = player
                            .rhythm_stacks
                            .saturating_add(1)
                            .min((maximum / step) as u8);
                    }
                    player.rhythm_timer = data.bonus_effect("rhythm_training", "hit_window_s");
                }
                if session.has_bonus("vampiric_rounds") {
                    let cap = data.bonus_effect("vampiric_rounds", "magazine_healing_cap");
                    let healing = data
                        .bonus_effect("vampiric_rounds", "bullet_hit_healing")
                        .min(cap - player.magazine_healing)
                        .max(0.0);
                    player.magazine_healing += healing;
                    session.health = (session.health + healing).min(session.max_health);
                }
                spawn_fire_burst(
                    &mut commands,
                    &mut random,
                    point,
                    -*direction,
                    &gun.impact_burst,
                );
                commands.entity(bullet_entity).despawn();
            }
            (_, Some(distance)) => {
                let point = start + *direction * distance;
                spawn_fire_burst(
                    &mut commands,
                    &mut random,
                    point,
                    -*direction,
                    &gun.muzzle_flash,
                );
                sounds.push(SoundCue::ProjectileEnvironmentHit);
                commands.entity(bullet_entity).despawn();
            }
            _ => {
                let end = start + *direction * travel;
                transform.translation = end.extend(transform.translation.z);
                let outside = end.cmplt(Vec2::ZERO).any() || end.cmpgt(room_size).any();
                if outside || bullet.remaining_range <= 0.0 {
                    commands.entity(bullet_entity).despawn();
                }
            }
        }
    }
}
