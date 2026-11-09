use super::*;

/// An enemy's health bar, and the gold it wears while its shield still holds.
pub(in crate::game) const HEALTH_BAR: Color = Color::srgb(0.8, 0.12, 0.2);
pub(in crate::game) const SHIELDED_HEALTH_BAR: Color = Color::srgb(0.85, 0.67, 0.13);

type HealthBarEnemiesQuery<'w, 's> = Query<
    'w,
    's,
    (&'static EnemyBody, &'static Transform),
    (Without<EnemyHealthBar>, Without<DeadEnemy>),
>;
type HealthBarBackgroundsQuery<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static mut Transform,
        &'static EnemyHealthBarBackground,
    ),
    (Without<EnemyHealthBar>, Without<EnemyBody>),
>;
type HealthBarFillsQuery<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static mut Sprite,
        &'static mut Transform,
        &'static EnemyHealthBar,
    ),
    Without<EnemyBody>,
>;
type PickupPlayerQuery<'w, 's> = Query<
    'w,
    's,
    (&'static Transform, &'static mut PlayerBody),
    (With<PlayerBody>, Without<HealthPickup>),
>;

pub(in crate::game) fn update_damage_numbers(
    mut commands: Commands,
    time: Res<Time>,
    mut numbers: Query<(Entity, &mut Transform, &mut TextColor, &mut DamageNumber)>,
) {
    for (entity, mut transform, mut color, mut number) in &mut numbers {
        number.timer.tick(time.delta());
        transform.translation.y += 30.0 * time.delta_secs();
        let opacity = 1.0 - number.timer.fraction();
        color.0 = number.color.with_alpha(opacity);
        if number.timer.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}

pub(in crate::game) fn update_dead_enemies(
    mut commands: Commands,
    time: Res<Time>,
    mut dead: Query<(Entity, &mut DeadEnemy)>,
) {
    for (entity, mut dead_enemy) in &mut dead {
        dead_enemy.timer.tick(time.delta());
        if dead_enemy.timer.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}

/// Spawns a fiery burst at `position`: a hot core over a wider orange glow,
/// both swelling as they cool, and embers thrown mostly along `direction`
/// (a zero `direction` throws them evenly all around).
pub(super) fn spawn_fire_burst(
    commands: &mut Commands,
    random: &mut RandomState,
    position: Vec2,
    direction: Vec2,
    burst: &crate::data::BurstConfig,
) {
    let radius = burst.radius_h * H_WORLD_UNITS;
    let timer = Timer::from_seconds(burst.duration_s, TimerMode::Once);
    let diamond = Quat::from_rotation_z(std::f32::consts::FRAC_PI_4);
    for (color, size, growth, z) in [
        (Color::srgb(1.0, 0.55, 0.12), radius * 1.6, 0.7, 5.9),
        (Color::srgb(1.0, 0.96, 0.7), radius, 0.4, 6.0),
    ] {
        commands.spawn((
            Sprite::from_color(color, Vec2::splat(size)),
            Transform::from_translation(position.extend(z)).with_rotation(diamond),
            ImpactEffect {
                timer: timer.clone(),
                velocity: Vec2::ZERO,
                growth,
                color,
            },
            RoomEntity,
        ));
    }
    let ember_color = Color::srgb(1.0, 0.78, 0.25);
    for _ in 0..burst.embers {
        let angle = random.fraction() * std::f32::consts::TAU;
        let heading =
            (Vec2::from_angle(angle) + direction * 1.2).normalize_or(Vec2::from_angle(angle));
        let speed = burst.ember_speed_h_s * H_WORLD_UNITS * (0.5 + 0.5 * random.fraction());
        commands.spawn((
            Sprite::from_color(ember_color, Vec2::splat((radius * 0.3).max(2.0))),
            Transform::from_translation(position.extend(6.1)),
            ImpactEffect {
                timer: timer.clone(),
                velocity: heading * speed,
                growth: -0.6,
                color: ember_color,
            },
            RoomEntity,
        ));
    }
}

pub(in crate::game) fn update_impact_effects(
    mut commands: Commands,
    time: Res<Time>,
    mut effects: Query<(Entity, &mut Transform, &mut Sprite, &mut ImpactEffect)>,
) {
    let dt = time.delta_secs();
    let embers = Color::srgb(0.7, 0.12, 0.04);
    for (entity, mut transform, mut sprite, mut effect) in &mut effects {
        effect.timer.tick(time.delta());
        if effect.timer.is_finished() {
            commands.entity(entity).despawn();
            continue;
        }
        let progress = effect.timer.fraction();
        transform.translation += (effect.velocity * dt).extend(0.0);
        effect.velocity *= (1.0 - 5.0 * dt).max(0.0);
        transform.scale = Vec3::splat((1.0 + effect.growth * progress).max(0.05));
        sprite.color = effect
            .color
            .mix(&embers, progress)
            .with_alpha(1.0 - progress * progress);
    }
}

/// Restores the trickle Recovery pays while nothing is engaged, and takes
/// away the plusses that have finished drifting.
#[derive(SystemParam)]
pub(in crate::game) struct HealingResources<'w> {
    time: Res<'w, Time>,
    data: Res<'w, GameData>,
    sheets: Res<'w, SpriteSheets>,
    session: ResMut<'w, RunSession>,
}

pub(in crate::game) fn update_healing_over_time(
    mut commands: Commands,
    resources: HealingResources,
    enemies: Query<&EnemyBody>,
    players: Query<&Transform, With<PlayerBody>>,
    mut effects: Query<(Entity, &mut HealEffect)>,
) {
    let HealingResources {
        time,
        data,
        sheets,
        mut session,
    } = resources;
    for (entity, mut effect) in &mut effects {
        if let Some(timer) = effect.timer.as_mut() {
            timer.tick(time.delta());
            if timer.is_finished() {
                commands.entity(entity).despawn();
            }
        }
    }
    if !session.has_bonus("recovery") || session.health >= session.max_health {
        return;
    }
    if enemies
        .iter()
        .any(|enemy| enemy.health > 0.0 && enemy.engaged)
    {
        session.idle_healing = 0.0;
        return;
    }
    let per_s = data.bonus_effect("recovery", "idle_regeneration_per_s");
    session.idle_healing += per_s * time.delta_secs().min(0.05);
    if session.idle_healing < 1.0 {
        return;
    }
    let restored = session.idle_healing.floor();
    session.idle_healing -= restored;
    session.health = (session.health + restored).min(session.max_health);
    // One play-through of the plusses over the body for every point back.
    let (Ok(transform), Some(sheet)) = (players.single(), sheets.0.get(HealEffect::SHEET)) else {
        return;
    };
    let mut sprite = Sprite::from_atlas_image(
        sheet.image.clone(),
        TextureAtlas {
            layout: sheet.layout.clone(),
            index: 0,
        },
    );
    sprite.custom_size =
        Some(sheet.frame_size * HealEffect::HEIGHT_H * H_WORLD_UNITS / sheet.frame_size.y);
    commands.spawn((
        sprite,
        Transform::from_translation(transform.translation + Vec3::new(0.0, 12.0, 1.0)),
        AnimatedSprite {
            sheet: HealEffect::SHEET.to_string(),
            frame: 0,
            frames: sheet.frames,
            timer: Timer::from_seconds(1.0 / sheet.fps, TimerMode::Repeating),
            looping: false,
        },
        HealEffect {
            timer: Some(Timer::from_seconds(
                sheet.frames as f32 / sheet.fps,
                TimerMode::Once,
            )),
        },
        RoomEntity,
    ));
}

pub(in crate::game) fn update_health_bars(
    mut commands: Commands,
    enemies: HealthBarEnemiesQuery,
    mut backgrounds: HealthBarBackgroundsQuery,
    mut bars: HealthBarFillsQuery,
) {
    for (bar_entity, mut sprite, mut transform, bar) in &mut bars {
        let Ok((enemy, enemy_transform)) = enemies.get(bar.enemy) else {
            commands.entity(bar_entity).despawn();
            continue;
        };
        if enemy.health <= 0.0 {
            commands.entity(bar_entity).despawn();
            continue;
        }
        let fraction = (enemy.health / bar.max_health).clamp(0.0, 1.0);
        sprite.color = if enemy.shielded {
            SHIELDED_HEALTH_BAR
        } else {
            HEALTH_BAR
        };
        sprite.custom_size = Some(Vec2::new(48.0 * fraction.max(0.02), 4.0));
        transform.translation.x = enemy_transform.translation.x - 24.0 * (1.0 - fraction);
        transform.translation.y = enemy_transform.translation.y + enemy.visible_top_offset + 6.0;
    }
    for (background_entity, mut transform, bar) in &mut backgrounds {
        let Ok((enemy, enemy_transform)) = enemies.get(bar.enemy) else {
            commands.entity(background_entity).despawn();
            continue;
        };
        transform.translation.x = enemy_transform.translation.x;
        transform.translation.y = enemy_transform.translation.y + enemy.visible_top_offset + 6.0;
    }
}

pub(in crate::game) fn update_health_pickups(
    mut commands: Commands,
    data: Res<GameData>,
    mut session: ResMut<RunSession>,
    mut player: PickupPlayerQuery,
    pickups: Query<(Entity, &Transform, &HealthPickup)>,
    mut sounds: ResMut<SoundQueue>,
) {
    let Ok((player_transform, mut player_body)) = player.single_mut() else {
        return;
    };
    for (entity, transform, pickup) in &pickups {
        if (transform.translation.truncate() - player_transform.translation.truncate()).length()
            > H_WORLD_UNITS * 0.7
        {
            continue;
        }
        let healing = pickup.healing
            * session.bonus_multiplier(&data, "field_dressing", "health_pickup_multiplier");
        session.health = (session.health + healing).min(session.max_health);
        sounds.push(SoundCue::Pickup);
        // The flask is raised and drunk where it was found.
        player_body.start_action(
            PlayerAction::Drink,
            data.player_animations.clips.drink.duration_s(),
        );
        if session.has_bonus("sated_fury") {
            player_body.sated_fury_timer =
                data.bonus_effect("sated_fury", "healing_pickup_combo_speed_duration_s");
        }
        commands.entity(entity).despawn();
    }
}
