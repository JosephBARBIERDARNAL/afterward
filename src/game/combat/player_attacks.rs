use super::*;

#[derive(SystemParam)]
pub(in crate::game) struct PlayerAttackResources<'w> {
    time: Res<'w, Time>,
    keys: Res<'w, ButtonInput<KeyCode>>,
    controls: Res<'w, ControlBindings>,
    data: Res<'w, GameData>,
    session: ResMut<'w, RunSession>,
    random: ResMut<'w, RandomState>,
    sheets: Res<'w, SpriteSheets>,
    sounds: ResMut<'w, SoundQueue>,
}

pub(in crate::game) fn player_attacks(
    mut commands: Commands,
    resources: PlayerAttackResources,
    mut player_query: Query<
        (
            &mut PlayerBody,
            &Transform,
            &mut Sprite,
            &mut AnimatedSprite,
        ),
        Without<EnemyBody>,
    >,
    mut enemy_query: Query<
        (
            Entity,
            &mut EnemyBody,
            &mut Transform,
            &mut Sprite,
            &mut AnimatedSprite,
        ),
        Without<PlayerBody>,
    >,
) {
    let PlayerAttackResources {
        time,
        keys,
        controls,
        data,
        mut session,
        mut random,
        sheets,
        mut sounds,
    } = resources;
    let Ok((mut player, player_transform, mut player_sprite, mut player_animation)) =
        player_query.single_mut()
    else {
        return;
    };
    let dt = time.delta_secs().min(0.05);
    player.sword_cooldown = (player.sword_cooldown - dt).max(0.0);
    player.gun_cooldown = (player.gun_cooldown - dt).max(0.0);
    player.chain_reaction_timer = (player.chain_reaction_timer - dt).max(0.0);
    player.sated_fury_timer = (player.sated_fury_timer - dt).max(0.0);
    player.rhythm_timer = (player.rhythm_timer - dt).max(0.0);
    if player.rhythm_timer == 0.0 {
        player.rhythm_stacks = 0;
    }

    let sword_speed = attack_speed_multiplier(&session, &data, &player, true);
    let sword_ready = controls.pressed(GameAction::Sword, &keys) && player.sword_cooldown == 0.0;
    if sword_ready {
        player.sword_cooldown = data.gameplay.player.sword.attack_interval_s / sword_speed;
        let mut total_healing = 0.0;
        let mut hit_any = false;
        let range = data.gameplay.player.sword.range_h * H_WORLD_UNITS;
        for (entity, mut enemy, mut transform, mut sprite, mut animation) in &mut enemy_query {
            if enemy.health <= 0.0 {
                continue;
            }
            let dx = transform.translation.x - player_transform.translation.x;
            let dy = (transform.translation.y - player_transform.translation.y).abs();
            if dx * player.facing < 0.0 || dx.abs() > range || dy > H_WORLD_UNITS * 1.25 {
                continue;
            }
            let damage = player_damage_multiplier(&session, &data, &player, false)
                * data.gameplay.player.sword.damage;
            let killed = damage_enemy(
                EnemyDamageResources {
                    commands: &mut commands,
                    data: &data,
                    session: &session,
                    random: &mut random,
                    sheets: &sheets,
                    sounds: &mut sounds,
                },
                EnemyDamageTarget {
                    entity,
                    enemy: &mut enemy,
                    transform: &mut transform,
                    sprite: &mut sprite,
                    animation: &mut animation,
                },
                EnemyHit {
                    damage,
                    knockback_h_s: data.gameplay.player.boss_knockback.sword_h_s,
                    knockback_direction: player.facing,
                    sword_hit: true,
                },
            );
            if killed {
                player.chain_reaction_timer =
                    effect_number(&data, "chain_reaction", "kill_damage_duration_s");
            }
            hit_any = true;
            if has_bonus(&session, "vampiric_edge") {
                total_healing += effect_number(&data, "vampiric_edge", "sword_hit_healing");
            }
        }
        if has_bonus(&session, "dash_strike") {
            player.dash_strike_timer = 0.0;
        }
        if hit_any {
            sounds.push(SoundCue::SwordHit);
            register_hit(&mut player, &session, &data);
            let cap = effect_number(&data, "vampiric_edge", "sword_swing_healing_cap");
            session.health = (session.health + total_healing.min(cap)).min(session.max_health);
        } else {
            sounds.push(SoundCue::SwordSwing);
        }
        set_animation(
            &mut player_sprite,
            &mut player_animation,
            &sheets,
            "player_sword",
            false,
        );
        player.action_animation_timer = 0.35;
    }

    if controls.pressed(GameAction::Gun, &keys) && player.gun_cooldown == 0.0 {
        sounds.push(SoundCue::Gunshot);
        let gun_speed = attack_speed_multiplier(&session, &data, &player, false);
        player.gun_cooldown = data.gameplay.player.gun.fire_interval_s / gun_speed;
        let damage = data.gameplay.player.gun.damage
            * player_damage_multiplier(&session, &data, &player, true);
        let velocity = data.gameplay.player.gun.projectile_speed_h_s
            * bonus_multiplier(
                &session,
                &data,
                "velocity_rounds",
                "projectile_speed_multiplier",
            )
            * H_WORLD_UNITS
            * player.facing;
        commands.spawn((
            Sprite::from_color(Color::srgb(1.0, 0.83, 0.32), Vec2::new(12.0, 5.0)),
            Transform::from_translation(
                player_transform.translation + Vec3::new(player.facing * 22.0, 0.0, 0.5),
            ),
            CombatProjectile {
                velocity_x: velocity,
                damage,
                knockback_h_s: data.gameplay.player.boss_knockback.bullet_h_s,
                life_s: 2.0,
                friendly: true,
            },
            RoomEntity,
        ));
        set_animation(
            &mut player_sprite,
            &mut player_animation,
            &sheets,
            if player.velocity.x.abs() > 20.0 {
                "player_gun_shoot_run"
            } else {
                "player_gun_shoot_idle"
            },
            true,
        );
        player.action_animation_timer = 0.2;
    }
}
