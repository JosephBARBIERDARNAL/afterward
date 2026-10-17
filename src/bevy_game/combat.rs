use bevy::ecs::system::SystemParam;

mod enemies;
use bevy::prelude::*;

use super::arena::{
    ArenaGeometry, ArenaObstacle, line_clear, point_hits_obstacle, resolve_circle_obstacles,
};
use super::config::ROOM_COUNT;
use super::model::Scene;
use super::model::*;
use super::progression::{bonus_count, return_home, start_run, update_overlay_input};
use crate::presentation::audio::{GameSounds, SoundCue, play_sound};
use crate::presentation::visual::{
    ActionVisual, Art, WeaponQuery, spawn_action_effect, spawn_damage, update_player_weapon,
};

pub(crate) const PLAYER_MOVE_SPEED: f32 = 340.0;

pub(crate) const HEAVY_MOVE_SPEED: f32 = 155.0;

pub(crate) const DASH_MOVE_SPEED: f32 = 610.0;

pub(crate) fn update_game(mut commands: Commands, params: GameParams) {
    let GameParams {
        keys,
        time,
        windows,
        cameras,
        mut game,
        art,
        sounds,
        geometry,
        mut player,
        mut weapon,
        mut enemies,
        mut projectiles,
        doors,
        caches,
    } = params;
    let dt = time.delta_secs().min(0.05);
    if game.scene == Scene::Home {
        if let Ok((mut transform, _)) = player.single_mut() {
            move_player(
                &keys,
                &mut transform,
                &mut game.facing,
                PLAYER_MOVE_SPEED,
                dt,
                &geometry.obstacles,
            );
            update_player_weapon(&mut weapon, transform.translation.truncate(), game.facing);
            let near_door = doors.iter().any(|door| {
                door.translation
                    .truncate()
                    .distance(transform.translation.truncate())
                    < 82.0
            });
            if keys.just_pressed(KeyCode::KeyE) && near_door {
                start_run(&mut game, time.elapsed_secs_f64().to_bits());
            }
        }
        return;
    }

    if !matches!(game.overlay, Overlay::None) {
        update_overlay_input(&keys, &mut game, time.elapsed_secs_f64().to_bits());
        if matches!(game.overlay, Overlay::Ended(_)) && keys.just_pressed(KeyCode::Enter) {
            return_home(&mut game);
        }
        return;
    }
    if keys.just_pressed(KeyCode::Escape) {
        game.overlay = Overlay::Pause(0);
        game.last_overlay_key.clear();
        return;
    }

    let Ok((mut player_transform, _)) = player.single_mut() else {
        return;
    };
    game.primary_cooldown = (game.primary_cooldown - dt).max(0.0);
    game.heavy_recovery = (game.heavy_recovery - dt).max(0.0);
    game.heavy_windup = (game.heavy_windup - dt).max(0.0);
    game.combo_timer = (game.combo_timer - dt).max(0.0);
    game.dash_cooldown = (game.dash_cooldown - dt).max(0.0);
    let previous_dash_time = game.dash_time;
    game.dash_time = (game.dash_time - dt).max(0.0);
    let dash_finished = previous_dash_time > 0.0 && game.dash_time == 0.0;
    game.invulnerable = (game.invulnerable - dt).max(0.0);

    if let (Ok(window), Ok((camera, camera_transform))) = (windows.single(), cameras.single())
        && let Some(cursor) = window.cursor_position()
        && let Ok(world) = camera.viewport_to_world_2d(camera_transform, cursor)
    {
        let delta = world - player_transform.translation.truncate();
        if delta.length_squared() > 4.0 {
            game.facing = delta.normalize();
        }
    }

    if keys.just_pressed(KeyCode::Space) && game.dash_cooldown <= 0.0 {
        play_sound(&mut commands, &sounds, SoundCue::Dash);
        spawn_action_effect(
            &mut commands,
            ActionVisual {
                position: player_transform.translation.truncate() - game.facing * 16.0,
                angle: game.facing.to_angle(),
                size: Vec2::new(50.0, 7.0),
                rgba: [0.96, 0.82, 0.54, 0.64],
                lifetime: 0.20,
                start_scale: 0.45,
                end_scale: 1.18,
                rise_speed: 0.0,
                z: 2.4,
            },
        );
        game.dash_time = 0.18;
        game.dash_cooldown = (0.78 - bonus_count(&game, Bonus::QuickStep) as f32 * 0.1).max(0.25);
        game.invulnerable = 0.24;
    }
    let speed = if game.dash_time > 0.0 {
        DASH_MOVE_SPEED
    } else if game.heavy_windup > 0.0 {
        HEAVY_MOVE_SPEED
    } else {
        PLAYER_MOVE_SPEED
    };
    move_player(
        &keys,
        &mut player_transform,
        &mut game.facing,
        speed,
        dt,
        &geometry.obstacles,
    );
    let player_pos = player_transform.translation.truncate();
    update_player_weapon(&mut weapon, player_pos, game.facing);

    if dash_finished && game.bonuses.contains(&Bonus::DashNova) {
        play_sound(&mut commands, &sounds, SoundCue::DashBurst);
        spawn_action_effect(
            &mut commands,
            ActionVisual {
                position: player_pos,
                angle: 0.0,
                size: Vec2::splat(88.0),
                rgba: [1.0, 0.61, 0.24, 0.72],
                lifetime: 0.22,
                start_scale: 0.20,
                end_scale: 1.15,
                rise_speed: 0.0,
                z: 4.2,
            },
        );
        hit_enemies(
            &mut commands,
            &sounds,
            &mut enemies,
            Attack {
                origin: player_pos,
                facing: game.facing,
                range: 88.0,
                damage: 18.0,
                shape: AttackShape::Arc { half_arc: -1.0 },
            },
            &geometry.obstacles,
        );
    }

    if keys.just_pressed(KeyCode::KeyJ)
        && game.primary_cooldown <= 0.0
        && game.heavy_recovery <= 0.0
    {
        play_sound(&mut commands, &sounds, SoundCue::Primary);
        game.primary_cooldown = 0.31;
        spawn_action_effect(
            &mut commands,
            ActionVisual {
                position: player_pos + game.facing * 38.0,
                angle: game.facing.to_angle() + std::f32::consts::FRAC_PI_2,
                size: Vec2::new(78.0, 24.0),
                rgba: [1.0, 0.93, 0.72, 0.96],
                lifetime: 0.14,
                start_scale: 0.38,
                end_scale: 1.12,
                rise_speed: 0.0,
                z: 4.4,
            },
        );
        if game.combo_timer <= 0.0 {
            game.combo_index = 0;
        }
        let final_hit = game.combo_index == 2;
        let combo_damage = [13.0, 13.0, 20.0][game.combo_index]
            + game.bonus_damage
            + if final_hit && game.bonuses.contains(&Bonus::CinderCombo) {
                6.0
            } else {
                0.0
            };
        let hit_count = hit_enemies(
            &mut commands,
            &sounds,
            &mut enemies,
            Attack {
                origin: player_pos,
                facing: game.facing,
                range: 74.0 + game.bonus_range,
                damage: combo_damage,
                shape: AttackShape::HorizontalSweep { half_width: 34.0 },
            },
            &geometry.obstacles,
        );
        if final_hit && game.bonuses.contains(&Bonus::ComboBlast) {
            if hit_count > 0 {
                play_sound(&mut commands, &sounds, SoundCue::ComboBurst);
            }
            hit_enemies(
                &mut commands,
                &sounds,
                &mut enemies,
                Attack {
                    origin: player_pos,
                    facing: game.facing,
                    range: 78.0,
                    damage: 14.0,
                    shape: AttackShape::Arc { half_arc: -1.0 },
                },
                &geometry.obstacles,
            );
        }
        game.player_health = (game.player_health
            + hit_count as f32 * bonus_count(&game, Bonus::Siphon) as f32 * 2.0)
            .min(game.max_health);
        game.combo_index = (game.combo_index + 1) % 3;
        game.combo_timer = 0.9;
    }
    if keys.just_pressed(KeyCode::KeyK) && game.heavy_recovery <= 0.0 && game.heavy_windup <= 0.0 {
        game.heavy_windup = 0.43;
        game.heavy_pending = true;
        spawn_action_effect(
            &mut commands,
            ActionVisual {
                position: player_pos,
                angle: 0.0,
                size: Vec2::splat(54.0),
                rgba: [0.94, 0.55, 0.22, 0.58],
                lifetime: game.heavy_windup,
                start_scale: 0.28,
                end_scale: 0.95,
                rise_speed: 0.0,
                z: 2.7,
            },
        );
    }
    if game.heavy_pending && game.heavy_windup <= 0.0 && game.heavy_recovery <= 0.0 {
        play_sound(&mut commands, &sounds, SoundCue::Heavy);
        game.heavy_recovery = 0.56;
        game.heavy_pending = false;
        spawn_action_effect(
            &mut commands,
            ActionVisual {
                position: player_pos + game.facing * 48.0,
                angle: game.facing.to_angle() + std::f32::consts::FRAC_PI_2,
                size: Vec2::new(80.0, 14.0),
                rgba: [1.0, 0.67, 0.31, 0.96],
                lifetime: 0.20,
                start_scale: 0.45,
                end_scale: 1.12,
                rise_speed: 0.0,
                z: 4.5,
            },
        );
        let wide = game.bonuses.contains(&Bonus::WideSwing);
        let hit_count = hit_enemies(
            &mut commands,
            &sounds,
            &mut enemies,
            Attack {
                origin: player_pos,
                facing: game.facing,
                range: 100.0 + game.bonus_range,
                damage: 44.0 + game.bonus_damage,
                shape: AttackShape::Arc {
                    half_arc: if wide { -0.1 } else { 0.52 },
                },
            },
            &geometry.obstacles,
        );
        game.player_health = (game.player_health
            + hit_count as f32 * bonus_count(&game, Bonus::Siphon) as f32 * 2.0)
            .min(game.max_health);
    }

    let enemy_context = enemies::EnemyUpdateContext::new(&art, &sounds, &geometry.obstacles);
    enemies::update_enemies(
        &mut commands,
        enemy_context,
        &mut game,
        &mut enemies,
        player_pos,
        dt,
    );
    update_projectiles(
        &mut commands,
        &sounds,
        &mut game,
        &mut projectiles,
        player_pos,
        dt,
        &geometry.obstacles,
    );
    let near_cache = geometry
        .cache
        .is_some_and(|position| position.distance(player_pos) < 48.0);
    if keys.just_pressed(KeyCode::KeyE) && near_cache && game.room_clear && !game.cache_claimed {
        game.cache_claimed = true;
        for entity in &caches {
            commands.entity(entity).despawn();
        }
        game.overlay = Overlay::BonusChoice {
            selected: 0,
            source: BonusSource::Cache,
        };
        game.last_overlay_key.clear();
        return;
    }
    let near_door = doors
        .iter()
        .any(|door| door.translation.truncate().distance(player_pos) < 82.0);
    if keys.just_pressed(KeyCode::KeyE) && near_door && game.room_clear {
        if game.room + 1 >= ROOM_COUNT {
            game.overlay = Overlay::Ended(true);
        } else {
            game.overlay = Overlay::BonusChoice {
                selected: 0,
                source: BonusSource::Door,
            };
        }
        game.last_overlay_key.clear();
    }
}

pub(crate) fn move_player(
    keys: &ButtonInput<KeyCode>,
    transform: &mut Transform,
    facing: &mut Vec2,
    speed: f32,
    dt: f32,
    obstacles: &[ArenaObstacle],
) {
    let mut direction = Vec2::ZERO;
    // KeyCode is physical: on AZERTY, the physical W/A positions are labeled Z/Q.
    if keys.pressed(KeyCode::ArrowUp) || keys.pressed(KeyCode::KeyW) {
        direction.y += 1.0;
    }
    if keys.pressed(KeyCode::ArrowDown) || keys.pressed(KeyCode::KeyS) {
        direction.y -= 1.0;
    }
    if keys.pressed(KeyCode::ArrowLeft) || keys.pressed(KeyCode::KeyA) {
        direction.x -= 1.0;
    }
    if keys.pressed(KeyCode::ArrowRight) || keys.pressed(KeyCode::KeyD) {
        direction.x += 1.0;
    }
    if direction != Vec2::ZERO {
        direction = direction.normalize();
        *facing = direction;
        transform.translation += (direction * speed * dt).extend(0.0);
    }
    let mut position = transform.translation.truncate();
    resolve_circle_obstacles(&mut position, 16.0, obstacles);
    transform.translation.x = position.x;
    transform.translation.y = position.y;
    transform.translation.x = transform.translation.x.clamp(-430.0, 430.0);
    transform.translation.y = transform.translation.y.clamp(-220.0, 220.0);
}

pub(crate) type PlayerQuery<'w, 's> =
    Query<'w, 's, (&'static mut Transform, &'static mut Player), (With<Player>, Without<Door>)>;

pub(crate) type EnemyQuery<'w, 's> = Query<
    'w,
    's,
    (Entity, &'static mut Transform, &'static mut Enemy),
    (With<Enemy>, Without<Player>, Without<Door>),
>;

pub(crate) type ProjectileQuery<'w, 's> = Query<
    'w,
    's,
    (Entity, &'static mut Transform, &'static mut Projectile),
    (
        With<Projectile>,
        Without<Player>,
        Without<Enemy>,
        Without<Door>,
    ),
>;

#[derive(SystemParam)]
pub(crate) struct GameParams<'w, 's> {
    pub(crate) keys: Res<'w, ButtonInput<KeyCode>>,
    pub(crate) time: Res<'w, Time>,
    pub(crate) windows: Query<'w, 's, &'static Window>,
    pub(crate) cameras: Query<'w, 's, (&'static Camera, &'static GlobalTransform)>,
    pub(crate) game: ResMut<'w, Game>,
    pub(crate) art: Res<'w, Art>,
    pub(crate) sounds: Res<'w, GameSounds>,
    pub(crate) geometry: Res<'w, ArenaGeometry>,
    pub(crate) player: PlayerQuery<'w, 's>,
    pub(crate) weapon: WeaponQuery<'w, 's>,
    pub(crate) enemies: EnemyQuery<'w, 's>,
    pub(crate) projectiles: ProjectileQuery<'w, 's>,
    pub(crate) doors: Query<'w, 's, &'static Transform, With<Door>>,
    pub(crate) caches: Query<'w, 's, Entity, With<BonusCache>>,
}

pub(crate) struct Attack {
    pub(crate) origin: Vec2,
    pub(crate) facing: Vec2,
    pub(crate) range: f32,
    pub(crate) damage: f32,
    pub(crate) shape: AttackShape,
}

#[derive(Clone, Copy)]
pub(crate) enum AttackShape {
    Arc { half_arc: f32 },
    HorizontalSweep { half_width: f32 },
}

pub(crate) fn hit_enemies(
    commands: &mut Commands,
    sounds: &GameSounds,
    enemies: &mut EnemyQuery<'_, '_>,
    attack: Attack,
    obstacles: &[ArenaObstacle],
) -> usize {
    let mut hits = 0;
    let mut defeated = 0;
    for (entity, transform, mut enemy) in enemies.iter_mut() {
        let offset = transform.translation.truncate() - attack.origin;
        let enemy_radius = if enemy.elite.is_some() { 17.0 } else { 15.0 };
        let hit = match attack.shape {
            AttackShape::Arc { half_arc } => {
                let distance = offset.length();
                distance <= attack.range
                    && (distance < 1.0 || offset.normalize().dot(attack.facing) >= half_arc)
            }
            AttackShape::HorizontalSweep { half_width } => {
                let forward = offset.dot(attack.facing);
                let lateral = offset.perp_dot(attack.facing).abs();
                forward >= -enemy_radius
                    && forward <= attack.range + enemy_radius
                    && lateral <= half_width + enemy_radius
            }
        };
        if hit && line_clear(attack.origin, transform.translation.truncate(), obstacles) {
            enemy.health -= attack.damage;
            enemy.stun = enemy.stun.max(0.13);
            hits += 1;
            spawn_damage(
                commands,
                transform.translation.truncate(),
                attack.damage,
                false,
            );
            if enemy.health <= 0.0 {
                defeated += 1;
                if let Some(telegraph) = enemy.telegraph.take() {
                    commands.entity(telegraph).despawn();
                }
                commands.entity(entity).despawn();
            }
        }
    }
    if hits > 0 {
        play_sound(commands, sounds, SoundCue::EnemyHit);
    }
    for _ in 0..defeated {
        play_sound(commands, sounds, SoundCue::EnemyDown);
    }
    hits
}

pub(crate) fn hurt_player(
    commands: &mut Commands,
    sounds: &GameSounds,
    game: &mut Game,
    position: Vec2,
    damage: f32,
) {
    if game.invulnerable > 0.0 {
        return;
    }
    game.player_health = (game.player_health - damage).max(0.0);
    game.invulnerable = 0.72;
    play_sound(commands, sounds, SoundCue::PlayerHit);
    if game.player_health == 0.0 {
        play_sound(commands, sounds, SoundCue::Death);
    }
    spawn_damage(commands, position, damage, true);
    spawn_action_effect(
        commands,
        ActionVisual {
            position,
            angle: 0.0,
            size: Vec2::splat(44.0),
            rgba: [0.98, 0.16, 0.12, 0.64],
            lifetime: 0.24,
            start_scale: 0.60,
            end_scale: 1.08,
            rise_speed: 0.0,
            z: 2.8,
        },
    );
}

pub(crate) fn update_projectiles(
    commands: &mut Commands,
    sounds: &GameSounds,
    game: &mut Game,
    projectiles: &mut ProjectileQuery<'_, '_>,
    player: Vec2,
    dt: f32,
    obstacles: &[ArenaObstacle],
) {
    for (entity, mut transform, mut projectile) in projectiles.iter_mut() {
        projectile.lifetime -= dt;
        transform.translation += (projectile.velocity * dt).extend(0.0);
        let position = transform.translation.truncate();
        let blocked = obstacles
            .iter()
            .copied()
            .any(|obstacle| point_hits_obstacle(position, 7.0, obstacle));
        if blocked {
            commands.entity(entity).despawn();
        } else if position.distance(player) < 22.0 {
            hurt_player(commands, sounds, game, player, projectile.damage);
            commands.entity(entity).despawn();
        } else if projectile.lifetime <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}
