use super::*;
use bevy::text::FontSize;

pub(super) struct EnemyDamageResources<'a, 'w, 's> {
    pub commands: &'a mut Commands<'w, 's>,
    pub data: &'a GameData,
    pub session: &'a RunSession,
    pub random: &'a mut RandomState,
    pub sheets: &'a SpriteSheets,
    pub sounds: &'a mut SoundQueue,
}

pub(super) struct EnemyDamageTarget<'a> {
    pub entity: Entity,
    pub enemy: &'a mut EnemyBody,
    pub transform: &'a mut Transform,
    pub sprite: &'a mut Sprite,
    pub animation: &'a mut AnimatedSprite,
}

pub(super) struct EnemyHit {
    pub damage: f32,
    pub knockback_h_s: f32,
    pub knockback_direction: f32,
    pub sword_hit: bool,
}

pub(super) fn damage_enemy<'a, 'w, 's>(
    resources: EnemyDamageResources<'a, 'w, 's>,
    target: EnemyDamageTarget<'a>,
    hit: EnemyHit,
) -> bool {
    let EnemyDamageResources {
        commands,
        data,
        session,
        random,
        sheets,
        sounds,
    } = resources;
    let EnemyDamageTarget {
        entity,
        enemy,
        transform,
        sprite,
        animation,
    } = target;
    let EnemyHit {
        damage,
        knockback_h_s,
        knockback_direction,
        sword_hit,
    } = hit;
    if enemy.health <= 0.0 {
        return false;
    }
    let damage = damage.max(0.0);
    enemy.health = (enemy.health - damage).max(0.0);
    spawn_damage_number(commands, transform.translation, damage, false);
    if enemy.type_id == "warrior" {
        enemy.knockback_velocity = knockback_direction * knockback_h_s;
        transform.translation.y = enemy.home_y;
    }
    if sword_hit {
        enemy.marked_timer = effect_number(data, "marked_prey", "mark_duration_s");
    }
    if data.enemies.damage_triggers_combat {
        enemy.engaged = true;
    }
    if enemy.health == 0.0 {
        sounds.push(SoundCue::EnemyDeath);
        enemy.windup_remaining = 0.0;
        enemy.combo_hit_pending = false;
        set_enemy_animation(sprite, animation, sheets, &enemy.type_id, "dead");
        commands.entity(entity).insert(DeadEnemy {
            timer: Timer::from_seconds(0.9, TimerMode::Once),
        });
        let drop_multiplier =
            bonus_multiplier(session, data, "scavenger", "health_drop_chance_multiplier");
        if random.chance((data.gameplay.health_pickup.base_drop_chance * drop_multiplier).min(1.0))
        {
            commands.spawn((
                Sprite::from_color(Color::srgb(0.28, 0.95, 0.48), Vec2::splat(18.0)),
                Transform::from_translation(transform.translation + Vec3::new(0.0, 20.0, 0.8)),
                HealthPickup {
                    healing: data.gameplay.health_pickup.healing,
                },
                RoomEntity,
            ));
        }
        true
    } else {
        sounds.push(SoundCue::EnemyHurt);
        enemy.hurt_timer = 0.24;
        enemy.windup_remaining = 0.0;
        enemy.combo_hit_pending = false;
        set_enemy_animation(sprite, animation, sheets, &enemy.type_id, "hurt");
        false
    }
}

pub(super) fn spawn_damage_number(
    commands: &mut Commands,
    position: Vec3,
    damage: f32,
    received: bool,
) {
    commands.spawn((
        Text2d::new(format!("{damage:.0}")),
        TextFont {
            font_size: FontSize::Px(if received { 20.0 } else { 16.0 }),
            ..default()
        },
        TextColor(if received {
            Color::srgb(1.0, 0.15, 0.18)
        } else {
            Color::WHITE
        }),
        Transform::from_translation(position + Vec3::new(0.0, 34.0, 20.0)),
        DamageNumber {
            timer: Timer::from_seconds(0.65, TimerMode::Once),
            received,
        },
        RoomEntity,
    ));
}

pub(super) fn set_enemy_animation(
    sprite: &mut Sprite,
    animation: &mut AnimatedSprite,
    sheets: &SpriteSheets,
    type_id: &str,
    action: &str,
) {
    let key = match action {
        "walk" => match type_id {
            "infantryman" => "enemy_infantry_walk",
            "destroyer" => "enemy_destroyer_walk",
            "gorgon_1" => "enemy_gorgon_1_run",
            "gorgon_2" => "enemy_gorgon_2_run",
            "gorgon_3" => "enemy_gorgon_3_run",
            "warrior" => "boss_walk",
            _ => enemy_idle_sheet(type_id),
        },
        "attack" | "attack_1" => match type_id {
            "swordsman" => "enemy_swordsman_attack",
            "infantryman" => "enemy_infantry_attack",
            "destroyer" => "enemy_destroyer_attack",
            "gorgon_1" => "enemy_gorgon_1_attack",
            "gorgon_2" => "enemy_gorgon_2_attack",
            "gorgon_3" => "enemy_gorgon_3_attack",
            "warrior" => "boss_attack",
            _ => enemy_idle_sheet(type_id),
        },
        "attack_2" => {
            if type_id == "warrior" {
                "boss_attack_2"
            } else {
                enemy_idle_sheet(type_id)
            }
        }
        "attack_3" => {
            if type_id == "warrior" {
                "boss_attack_3"
            } else {
                enemy_idle_sheet(type_id)
            }
        }
        "jump" => {
            if type_id == "warrior" {
                "boss_jump"
            } else {
                enemy_idle_sheet(type_id)
            }
        }
        "land" => {
            if type_id == "warrior" {
                "boss_land"
            } else {
                enemy_idle_sheet(type_id)
            }
        }
        "hurt" => match type_id {
            "swordsman" => "enemy_swordsman_hurt",
            "infantryman" => "enemy_infantry_hurt",
            "destroyer" => "enemy_destroyer_hurt",
            "gorgon_1" => "enemy_gorgon_1_hurt",
            "gorgon_2" => "enemy_gorgon_2_hurt",
            "gorgon_3" => "enemy_gorgon_3_hurt",
            "warrior" => "boss_hurt",
            _ => enemy_idle_sheet(type_id),
        },
        "dead" => match type_id {
            "swordsman" => "enemy_swordsman_dead",
            "infantryman" => "enemy_infantry_dead",
            "destroyer" => "enemy_destroyer_dead",
            "gorgon_1" => "enemy_gorgon_1_dead",
            "gorgon_2" => "enemy_gorgon_2_dead",
            "gorgon_3" => "enemy_gorgon_3_dead",
            "warrior" => "boss_dead",
            _ => enemy_idle_sheet(type_id),
        },
        _ => enemy_idle_sheet(type_id),
    };
    let Some(sheet) = sheets.0.get(key) else {
        return;
    };
    if animation.sheet == key {
        return;
    }
    sprite.image = sheet.image.clone();
    sprite.texture_atlas = Some(TextureAtlas {
        layout: sheet.layout.clone(),
        index: 0,
    });
    animation.sheet = key.to_string();
    animation.frame = 0;
    animation.frames = sheet.frames;
    animation.looping = matches!(action, "walk" | "idle");
    animation.timer = Timer::from_seconds(1.0 / sheet.fps, TimerMode::Repeating);
}
