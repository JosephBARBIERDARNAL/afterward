use super::*;

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

pub(in crate::game) fn update_impact_effects(
    mut commands: Commands,
    time: Res<Time>,
    mut effects: Query<(Entity, &mut ImpactEffect)>,
) {
    for (entity, mut effect) in &mut effects {
        effect.timer.tick(time.delta());
        if effect.timer.is_finished() {
            commands.entity(entity).despawn();
        }
    }
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
        session.health = (session.health + pickup.healing).min(session.max_health);
        sounds.push(SoundCue::Pickup);
        if has_bonus(&session, "sated_fury") {
            player_body.sated_fury_timer =
                data.bonus_effect("sated_fury", "healing_pickup_attack_speed_duration_s");
        }
        commands.entity(entity).despawn();
    }
}
