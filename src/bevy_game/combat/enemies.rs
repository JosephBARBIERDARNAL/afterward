use bevy::prelude::*;

use super::{EnemyQuery, hurt_player};
use crate::bevy_game::arena::{ArenaObstacle, line_clear, resolve_circle_obstacles};
use crate::bevy_game::model::*;
use crate::bevy_game::ui::spawn_text;
use crate::presentation::audio::{GameSounds, SoundCue, play_sound};
use crate::presentation::visual::{Art, AttackTelegraph};

pub(super) struct EnemyUpdateContext<'a> {
    art: &'a Art,
    sounds: &'a GameSounds,
    obstacles: &'a [ArenaObstacle],
}

impl<'a> EnemyUpdateContext<'a> {
    pub(super) fn new(
        art: &'a Art,
        sounds: &'a GameSounds,
        obstacles: &'a [ArenaObstacle],
    ) -> Self {
        Self {
            art,
            sounds,
            obstacles,
        }
    }
}

pub(super) fn update_enemies(
    commands: &mut Commands,
    context: EnemyUpdateContext<'_>,
    game: &mut Game,
    enemies: &mut EnemyQuery<'_, '_>,
    player: Vec2,
    dt: f32,
) {
    let mut count = 0;
    for (entity, mut transform, mut enemy) in enemies.iter_mut() {
        if enemy.health <= 0.0 {
            continue;
        }
        count += 1;
        enemy.cooldown = (enemy.cooldown - dt).max(0.0);
        enemy.stun = (enemy.stun - dt).max(0.0);
        let enemy_radius = if enemy.elite.is_some() { 17.0 } else { 15.0 };
        let mut position = transform.translation.truncate();
        resolve_circle_obstacles(&mut position, enemy_radius, context.obstacles);
        transform.translation.x = position.x;
        transform.translation.y = position.y;
        let delta = player - position;
        let distance = delta.length().max(0.01);
        let toward = delta / distance;
        let vanguard = enemy.elite == Some(EliteKind::Vanguard);
        let speed_multiplier = if vanguard { 1.12 } else { 1.0 };

        if enemy.windup > 0.0 {
            enemy.windup = (enemy.windup - dt).max(0.0);
            if enemy.windup == 0.0 {
                resolve_enemy_attack(
                    commands,
                    context.art,
                    context.sounds,
                    game,
                    &mut enemy,
                    position,
                    player,
                );
            }
            continue;
        }

        if enemy.stun > 0.0 {
            if enemy.charge_time > 0.0 {
                enemy.charge_time = 0.0;
            }
            continue;
        }

        if enemy.charge_time > 0.0 {
            enemy.charge_time = (enemy.charge_time - dt).max(0.0);
            transform.translation +=
                (enemy.charge_direction * 375.0 * speed_multiplier * dt).extend(0.0);
            let attempted_position = transform.translation.truncate();
            position = attempted_position;
            resolve_circle_obstacles(&mut position, enemy_radius, context.obstacles);
            transform.translation.x = position.x;
            transform.translation.y = position.y;
            if position.distance_squared(attempted_position) > 0.25 {
                enemy.charge_time = 0.0;
            }
            if transform.translation.truncate().distance(player) < 28.0 {
                hurt_player(commands, context.sounds, game, player, enemy.kind.damage());
                enemy.charge_time = 0.0;
            }
            if enemy.charge_time == 0.0 {
                enemy.stun = 0.28;
            }
            continue;
        }

        match enemy.kind {
            EnemyKind::Ranged => {
                let preferred_distance = 218.0;
                if enemy.cooldown <= 0.0
                    && distance < preferred_distance + 95.0
                    && line_clear(position, player, context.obstacles)
                {
                    begin_enemy_attack(commands, context.art, &mut enemy, player);
                    continue;
                }
                let radial = if distance > preferred_distance + 30.0 {
                    1.0
                } else if distance < preferred_distance - 36.0 {
                    -1.0
                } else {
                    0.0
                };
                transform.translation += (toward * enemy.kind.speed() * radial * dt).extend(0.0);
            }
            EnemyKind::Charger => {
                if enemy.cooldown <= 0.0
                    && distance <= enemy.kind.attack_range()
                    && line_clear(position, player, context.obstacles)
                {
                    begin_enemy_attack(commands, context.art, &mut enemy, player);
                    continue;
                }
                transform.translation +=
                    (toward * enemy.kind.speed() * speed_multiplier * dt).extend(0.0);
            }
            EnemyKind::Skirmisher => {
                if enemy.cooldown <= 0.0
                    && distance <= enemy.kind.attack_range()
                    && line_clear(position, player, context.obstacles)
                {
                    begin_enemy_attack(commands, context.art, &mut enemy, player);
                    continue;
                }
                let preferred_distance = 98.0;
                let radial = if distance > preferred_distance {
                    0.8
                } else if distance < preferred_distance - 18.0 {
                    -0.3
                } else {
                    0.0
                };
                let lateral = 0.58 * enemy.orbit_side;
                let orbit = Vec2::new(
                    toward.x * radial - toward.y * lateral,
                    toward.y * radial + toward.x * lateral,
                );
                transform.translation +=
                    (orbit * enemy.kind.speed() * speed_multiplier * dt).extend(0.0);
            }
            EnemyKind::Melee | EnemyKind::Heavy | EnemyKind::Bomber => {
                let attack_range = enemy.kind.attack_range();
                if enemy.cooldown <= 0.0
                    && distance <= attack_range
                    && line_clear(position, player, context.obstacles)
                {
                    begin_enemy_attack(commands, context.art, &mut enemy, player);
                    continue;
                }
                if distance > attack_range - 8.0 {
                    transform.translation +=
                        (toward * enemy.kind.speed() * speed_multiplier * dt).extend(0.0);
                }
            }
        }

        position = transform.translation.truncate();
        resolve_circle_obstacles(&mut position, enemy_radius, context.obstacles);
        transform.translation.x = position.x;
        transform.translation.y = position.y;

        if enemy.health <= 0.0 {
            if let Some(telegraph) = enemy.telegraph.take() {
                commands.entity(telegraph).despawn();
            }
            commands.entity(entity).despawn();
        }
    }
    if count == 0 && !game.room_clear {
        game.room_clear = true;
        play_sound(commands, context.sounds, SoundCue::Clear);
        spawn_text(
            commands,
            "ROOM CLEAR  ·  APPROACH THE EAST DOOR AND PRESS E",
            Vec2::new(0.0, 190.0),
            18.0,
            Color::srgb(0.89, 0.78, 0.56),
        );
    }
    if game.player_health <= 0.0 {
        game.player_health = 0.0;
        game.overlay = Overlay::Ended(false);
        game.last_overlay_key.clear();
    }
}

fn begin_enemy_attack(commands: &mut Commands, art: &Art, enemy: &mut Enemy, target: Vec2) {
    enemy.target = target;
    enemy.windup = enemy.kind.attack_windup();
    let radius = enemy.kind.attack_radius();
    let lifetime = enemy.windup;
    enemy.telegraph = Some(
        commands
            .spawn((
                Mesh2d(art.circle.clone()),
                MeshMaterial2d(art.warning.clone()),
                Transform::from_xyz(target.x, target.y, 0.8).with_scale(Vec3::splat(radius * 0.12)),
                SceneEntity,
                AttackTelegraph {
                    age: 0.0,
                    lifetime,
                    radius,
                },
            ))
            .id(),
    );
}

fn resolve_enemy_attack(
    commands: &mut Commands,
    art: &Art,
    sounds: &GameSounds,
    game: &mut Game,
    enemy: &mut Enemy,
    origin: Vec2,
    player: Vec2,
) {
    if let Some(telegraph) = enemy.telegraph.take() {
        commands.entity(telegraph).despawn();
    }
    let cooldown_multiplier = if enemy.elite == Some(EliteKind::Vanguard) {
        0.82
    } else {
        1.0
    };
    enemy.cooldown = enemy.kind.attack_cooldown() * cooldown_multiplier;

    match enemy.kind {
        EnemyKind::Charger => {
            enemy.charge_direction = (enemy.target - origin).normalize_or_zero();
            if enemy.charge_direction == Vec2::ZERO {
                enemy.charge_direction = Vec2::X;
            }
            enemy.charge_time = 0.34;
        }
        EnemyKind::Ranged => {
            let direction = (enemy.target - origin).normalize_or_zero();
            let spreads: &[f32] = if enemy.elite == Some(EliteKind::Volley) {
                &[-0.16, 0.0, 0.16]
            } else {
                &[0.0]
            };
            for spread in spreads {
                commands.spawn((
                    Mesh2d(art.circle.clone()),
                    MeshMaterial2d(art.ranged.clone()),
                    Transform::from_xyz(origin.x, origin.y, 3.0).with_scale(Vec3::splat(7.0)),
                    SceneEntity,
                    Projectile {
                        velocity: direction.rotate(Vec2::from_angle(*spread)) * 265.0,
                        lifetime: 2.8,
                        damage: enemy.kind.damage(),
                    },
                ));
            }
        }
        _ => {
            if player.distance(enemy.target) < enemy.kind.attack_radius() {
                hurt_player(commands, sounds, game, player, enemy.kind.damage());
            }
        }
    }
}
