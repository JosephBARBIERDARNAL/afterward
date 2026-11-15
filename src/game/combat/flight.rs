//! Enemies that fly (`flight` on their type): they keep no floor, see all
//! round them, and fly straight at a spot by their mark, stopped by the
//! platforms as a walk is. A flying mine (`contact_h`) goes off on reaching
//! its mark; a flying gunner hovers over it and drops its shot straight down.

use super::*;

/// One flying enemy's turn, from `update_enemy_ai`.
pub(super) struct Flyer<'a> {
    pub(super) entity: Entity,
    pub(super) enemy: &'a mut EnemyBody,
    pub(super) transform: &'a mut Transform,
    pub(super) sprite: &'a mut Sprite,
    pub(super) animation: &'a mut AnimatedSprite,
}

/// What a flyer's turn reads and writes outside itself.
pub(super) struct FlightWorld<'a, 'w, 's> {
    pub(super) commands: &'a mut Commands<'w, 's>,
    pub(super) data: &'a GameData,
    pub(super) session: &'a mut RunSession,
    pub(super) sounds: &'a mut SoundQueue,
    pub(super) player: &'a mut PlayerBody,
    pub(super) player_transform: &'a Transform,
    pub(super) sheets: &'a SpriteSheets,
    pub(super) surfaces: &'a Platforms<'a>,
    pub(super) dt: f32,
}

/// Whether a flyer at `eyes` sees `target`: as far all round it as a walker
/// sees ahead, unless a platform stands between them.
pub(super) fn sees(data: &GameData, surfaces: &Platforms, eyes: Vec2, target: Vec2) -> bool {
    eyes.distance(target) <= data.enemies.detection.range_h * H_WORLD_UNITS
        && !surfaces.blocks_sight(eyes, target)
}

pub(super) fn fly(world: FlightWorld, flyer: Flyer, mark: &Mark) {
    let FlightWorld {
        commands,
        data,
        session,
        sounds,
        player,
        player_transform,
        sheets,
        surfaces,
        dt,
    } = world;
    let Flyer {
        entity,
        enemy,
        transform,
        sprite,
        animation,
    } = flyer;
    let Some(flight) = enemy.flight else {
        return;
    };
    let origin = transform.translation.truncate();
    // Where its origin should be: a mine on its mark's middle, a gunner
    // `hover_h` above its feet.
    let target = match flight.contact_h {
        Some(_) => mark.hurtbox.center(),
        None => mark.feet + Vec2::new(0.0, flight.hover_h * H_WORLD_UNITS + enemy.feet_offset),
    };
    let wanted =
        origin + (target - origin).clamp_length_max(enemy.movement_speed_h_s * H_WORLD_UNITS * dt);
    // A platform stops it on whichever axis would take it in.
    let size = enemy.hurtbox_size;
    let inside = |at: Vec2| {
        let feet = at.y - enemy.feet_offset;
        let body = Rect::new(
            at.x - size.x / 2.0,
            feet,
            at.x + size.x / 2.0,
            feet + size.y,
        );
        surfaces.0.iter().any(|platform| {
            platform.left < body.max.x
                && platform.right > body.min.x
                && platform.bottom < body.max.y
                && platform.top > body.min.y
        })
    };
    let mut at = origin;
    if !inside(Vec2::new(wanted.x, at.y)) {
        at.x = wanted.x;
    }
    if !inside(Vec2::new(at.x, wanted.y)) {
        at.y = wanted.y;
    }
    transform.translation.x = at.x;
    transform.translation.y = at.y;
    enemy.home_y = at.y;
    let middle = enemy.hurtbox(transform).center();
    let profile = &data.enemies.types[&enemy.type_id];

    // A mine that reaches its mark goes off there.
    if let Some(contact_h) = flight.contact_h
        && middle.distance(mark.hurtbox.center()) <= contact_h * H_WORLD_UNITS
    {
        enemy.health = 0.0;
        commands.entity(entity).insert(DeadEnemy {
            timer: Timer::from_seconds(0.15, TimerMode::Once),
        });
        if let Some(blast) = &profile.death_blast {
            // The player it reached is hit at once, through the same path as
            // any blow: a blast left to burn would never go off if the mine
            // was the room's last enemy, since the room clears as it dies.
            // The blast still burns for a double it reached instead.
            if mark.decoy.is_none() {
                apply_player_damage(
                    PlayerDamageResources {
                        commands,
                        data,
                        session,
                        sounds,
                    },
                    PlayerDamageTarget {
                        player,
                        transform: player_transform,
                    },
                    blast.damage,
                );
            }
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
        sprite.color = Color::NONE;
        return;
    }

    // A gunner right over its mark winds up, warming toward the danger
    // colour as every enemy does, then drops its shot.
    let attack = enemy
        .attacks
        .first()
        .cloned()
        .filter(|attack| attack.kind == "projectile");
    if let Some(attack) = attack {
        if enemy.windup_remaining > 0.0 {
            enemy.windup_remaining = (enemy.windup_remaining - dt).max(0.0);
            let warning = &data.gameplay.effects.telegraph;
            let warm = Color::from(warning.color).to_srgba() * warning.brightness;
            let nearness = (1.0 - enemy.windup_remaining / attack.windup_s).clamp(0.0, 1.0);
            sprite.color = Color::srgb(
                1.0 + (warm.red - 1.0) * nearness,
                1.0 + (warm.green - 1.0) * nearness,
                1.0 + (warm.blue - 1.0) * nearness,
            );
            if enemy.windup_remaining == 0.0 {
                sprite.color = Color::WHITE;
                enemy.attack_cooldown = attack.cooldown_s;
                drop_shot(commands, data, sheets, enemy, &attack, middle);
            }
        } else if enemy.attack_cooldown == 0.0
            && (mark.feet.x - at.x).abs() < 0.75 * H_WORLD_UNITS
            && middle.y > mark.hurtbox.max.y
        {
            enemy.windup_remaining = attack.windup_s;
            set_enemy_animation(sprite, animation, sheets, &enemy.type_id, "attack");
        }
    }
    if enemy.windup_remaining == 0.0 {
        set_enemy_animation(sprite, animation, sheets, &enemy.type_id, "walk");
    }
}

/// Drops a flying gunner's shot straight down from under it.
fn drop_shot(
    commands: &mut Commands,
    data: &GameData,
    sheets: &SpriteSheets,
    enemy: &EnemyBody,
    attack: &EnemyAttack,
    middle: Vec2,
) {
    let speed = attack.projectile_speed_h_s * H_WORLD_UNITS;
    let range = speed * 3.0;
    let muzzle = middle - Vec2::new(0.0, enemy.hurtbox_size.y / 2.0);
    let key = format!("{}_shot", enemy.type_id);
    let scale = data.enemies.types[&enemy.type_id]
        .sprite
        .map_or(1.0, |sprite| sprite.scale);
    let mut shot = commands.spawn((
        Transform::from_translation(muzzle.extend(3.4)).with_scale(Vec3::splat(scale)),
        CombatProjectile {
            velocity_x: 0.0,
            velocity_y: -speed,
            damage: attack.damage,
            life_s: 3.0,
            remaining_range: range,
            half_size: attack.projectile_size / 2.0,
        },
        RoomEntity,
    ));
    if let Some(sheet) = sheets.0.get(&key) {
        shot.insert((
            Sprite::from_atlas_image(
                sheet.image.clone(),
                TextureAtlas {
                    layout: sheet.layout.clone(),
                    index: 0,
                },
            ),
            AnimatedSprite {
                sheet: key,
                frame: 0,
                frames: sheet.frames,
                timer: Timer::from_seconds(1.0 / sheet.fps, TimerMode::Repeating),
                looping: true,
            },
        ));
    }
}
