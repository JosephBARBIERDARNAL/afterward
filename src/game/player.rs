use super::*;

#[derive(SystemParam)]
pub(super) struct PlayerUpdateResources<'w> {
    time: Res<'w, Time>,
    keys: Res<'w, ButtonInput<KeyCode>>,
    controls: Res<'w, ControlBindings>,
    data: Res<'w, GameData>,
    session: Res<'w, RunSession>,
    sheets: Res<'w, SpriteSheets>,
    sounds: ResMut<'w, SoundQueue>,
}

pub(super) fn update_player(
    resources: PlayerUpdateResources,
    room_query: Query<&PlatformSurface>,
    mut player_query: Query<(
        &mut PlayerBody,
        &mut Transform,
        &mut Sprite,
        &mut AnimatedSprite,
    )>,
) {
    let PlayerUpdateResources {
        time,
        keys,
        controls,
        data,
        session,
        sheets,
        mut sounds,
    } = resources;
    let Ok((mut player, mut transform, mut sprite, mut animation)) = player_query.single_mut()
    else {
        return;
    };
    let dt = time.delta_secs().min(0.05);
    let tuning = &data.gameplay.player;
    player.action_animation_timer = (player.action_animation_timer - dt).max(0.0);
    player.dash_strike_timer = (player.dash_strike_timer - dt).max(0.0);
    let movement_multiplier =
        bonus_multiplier(&session, &data, "fleet_feet", "movement_speed_multiplier");
    let was_dashing = player.dash_timer > 0.0;
    let mut direction: f32 = 0.0;
    if controls.pressed(GameAction::MoveLeft, &keys) {
        direction -= 1.0;
    }
    if controls.pressed(GameAction::MoveRight, &keys) {
        direction += 1.0;
    }
    if direction != 0.0 {
        if !was_dashing {
            player.facing = direction.signum();
        }
        let max_speed_h_s = if player.grounded {
            tuning.max_speed_h_s
        } else {
            tuning.air_max_speed_h_s
        };
        let target_speed = direction * max_speed_h_s * movement_multiplier * H_WORLD_UNITS;
        player.velocity.x = move_towards(
            player.velocity.x,
            target_speed,
            tuning.acceleration_h_s2 * H_WORLD_UNITS * dt,
        );
    } else {
        player.velocity.x = move_towards(
            player.velocity.x,
            0.0,
            tuning.deceleration_h_s2 * H_WORLD_UNITS * dt,
        );
    }

    if controls.just_pressed(GameAction::Jump, &keys) {
        let extra_jump = if session.owned_bonuses.iter().any(|id| id == "double_jump") {
            data.bonus_effect("double_jump", "extra_midair_jumps") as u8
        } else {
            0
        };
        let jumps_allowed = tuning.jumps_per_takeoff + extra_jump;
        if player.grounded || player.jumps_used < jumps_allowed {
            let jump_factor = if player.jumps_used == 0 {
                bonus_multiplier(&session, &data, "high_jump", "jump_height_multiplier").sqrt()
            } else {
                data.bonus_effect("double_jump", "extra_jump_impulse_factor")
            };
            player.velocity.y = tuning.jump_impulse_h_s * jump_factor * H_WORLD_UNITS;
            player.jumps_used += 1;
            player.grounded = false;
            sounds.push(SoundCue::Jump);
        }
    }

    player.dash_cooldown = (player.dash_cooldown - dt).max(0.0);
    if controls.just_pressed(GameAction::Dash, &keys) && player.dash_cooldown == 0.0 {
        player.dash_timer = tuning.dash_duration_s;
        player.dash_cooldown = tuning.dash_cooldown_s
            * bonus_multiplier(&session, &data, "dash_recharge", "dash_cooldown_multiplier");
        player.dash_invulnerability_timer = tuning.dash_invulnerability_s;
        player.dash_direction = player.facing;
        sounds.push(SoundCue::Dash);
    }

    let dash_step = player.dash_timer.min(dt);
    player.dash_timer = (player.dash_timer - dash_step).max(0.0);
    let normal_step = dt - dash_step;
    if was_dashing && player.dash_timer == 0.0 {
        player.dash_strike_timer = if session.owned_bonuses.iter().any(|id| id == "dash_strike") {
            data.bonus_effect("dash_strike", "post_dash_window_s")
        } else {
            0.0
        };
    }

    let previous_bottom = transform.translation.y - H_WORLD_UNITS / 2.0;
    if dash_step > 0.0 {
        transform.translation.x += player.dash_direction * tuning.dash_distance_h
            / tuning.dash_duration_s
            * H_WORLD_UNITS
            * dash_step;
    }
    if normal_step > 0.0 {
        transform.translation.x += player.velocity.x * normal_step;
        player.velocity.y -= tuning.gravity_h_s2 * H_WORLD_UNITS * normal_step;
        transform.translation.y += player.velocity.y * normal_step;
    } else if dash_step > 0.0 {
        transform.translation.y += player.velocity.y * dash_step;
    }

    let room = &data.rooms[&session.room_id];
    let room_width_world = room.width_h(&data.gameplay) * H_WORLD_UNITS;
    let half_width = PLAYER_COLLISION_WIDTH_H * H_WORLD_UNITS / 2.0;
    transform.translation.x = transform
        .translation
        .x
        .clamp(half_width, room_width_world - half_width);

    let was_grounded = player.grounded;
    player.grounded = false;
    if player.velocity.y <= 0.0 {
        let current_bottom = transform.translation.y - H_WORLD_UNITS / 2.0;
        for platform in &room_query {
            let overlaps_x = transform.translation.x + half_width > platform.left
                && transform.translation.x - half_width < platform.right;
            if overlaps_x && previous_bottom >= platform.top - 1.0 && current_bottom <= platform.top
            {
                transform.translation.y = platform.top + H_WORLD_UNITS / 2.0;
                player.velocity.y = 0.0;
                player.grounded = true;
            }
        }
    }
    if player.grounded {
        player.jumps_used = 0;
        if !was_grounded {
            sounds.push(SoundCue::Landing);
        }
    }
    if player.grounded && player.velocity.x.abs() > 20.0 {
        player.footstep_timer = if was_grounded {
            (player.footstep_timer - dt).max(0.0)
        } else {
            data.gameplay.audio.footstep_interval_s
        };
        if player.footstep_timer == 0.0 {
            sounds.push(SoundCue::Footstep);
            player.footstep_timer = data.gameplay.audio.footstep_interval_s;
        }
    } else {
        player.footstep_timer = 0.0;
    }

    player.stationary_timer = if player.velocity.x.abs() < 5.0 {
        player.stationary_timer + dt
    } else {
        0.0
    };
    if player.action_animation_timer <= 0.0 {
        let animation_key = if player.dash_timer > 0.0 {
            "player_dash"
        } else if !player.grounded {
            "player_jump"
        } else if player.velocity.x.abs() > 20.0 {
            match player.weapon_mode {
                PlayerWeapon::Sword => "player_running_sword",
                PlayerWeapon::Gun => "player_running_gun",
            }
        } else {
            match player.weapon_mode {
                PlayerWeapon::Sword => "player_idle_sword",
                PlayerWeapon::Gun => "player_idle_gun",
            }
        };
        set_animation(&mut sprite, &mut animation, &sheets, animation_key, true);
    }
    sprite.flip_x = player.facing < 0.0;
}

fn bonus_multiplier(session: &RunSession, data: &GameData, id: &str, effect: &str) -> f32 {
    if session.owned_bonuses.iter().any(|owned| owned == id) {
        data.bonus_effect(id, effect)
    } else {
        1.0
    }
}
