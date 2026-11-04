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
    let walls = &tuning.walls;
    let terrain = Terrain(room_query.iter().collect());
    player.action_animation_timer = (player.action_animation_timer - dt).max(0.0);
    player.dash_strike_timer = (player.dash_strike_timer - dt).max(0.0);
    player.dash_cooldown = (player.dash_cooldown - dt).max(0.0);
    player.wall_jump_lock_timer = (player.wall_jump_lock_timer - dt).max(0.0);
    player.drop_through_timer = (player.drop_through_timer - dt).max(0.0);

    // Mantling onto a ledge owns the body until the climb ends.
    if let Some(mut mantle) = player.mantle.take() {
        mantle.elapsed_s += dt;
        let progress = (mantle.elapsed_s / walls.ledge_climb_s).min(1.0);
        // Rise beside the wall first, then step over onto the ledge.
        let rise = (progress / 0.6).min(1.0);
        let step = ((progress - 0.6) / 0.4).clamp(0.0, 1.0);
        transform.translation.x = mantle.from.x + (mantle.to.x - mantle.from.x) * step;
        transform.translation.y = mantle.from.y + (mantle.to.y - mantle.from.y) * rise;
        if progress < 1.0 {
            player.mantle = Some(mantle);
        } else {
            player.grounded = true;
            player.jumps_used = 0;
            player.velocity = Vec2::ZERO;
            sounds.push(SoundCue::Landing);
        }
        set_animation(
            &mut sprite,
            &mut animation,
            &sheets,
            "player_ledge_climb",
            false,
        );
        sprite.flip_x = player.facing < 0.0;
        return;
    }

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
    if player.wall_jump_lock_timer > 0.0 {
        // The wall-jump kick carries the player away from the wall.
    } else if direction != 0.0 {
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
        if player.grounded && player.on_one_way && controls.pressed(GameAction::MoveDown, &keys) {
            player.drop_through_timer = tuning.drop_through_s;
            player.grounded = false;
        } else if let (false, Some(side)) = (player.grounded, player.wall_side) {
            player.velocity = Vec2::new(-side * walls.jump_kick_speed_h_s, walls.jump_impulse_h_s)
                * H_WORLD_UNITS;
            player.facing = -side;
            player.jumps_used = player.jumps_used.max(tuning.jumps_per_takeoff);
            player.jump_held = true;
            player.wall_jump_lock_timer = walls.jump_control_lock_s;
            sounds.push(SoundCue::Jump);
        } else if player.grounded || player.jumps_used < jumps_allowed {
            let jump_factor = if player.jumps_used == 0 {
                bonus_multiplier(&session, &data, "high_jump", "jump_height_multiplier").sqrt()
            } else {
                data.bonus_effect("double_jump", "extra_jump_impulse_factor")
            };
            player.velocity.y = tuning.jump_impulse_h_s * jump_factor * H_WORLD_UNITS;
            player.jumps_used += 1;
            player.jump_held = true;
            player.grounded = false;
            sounds.push(SoundCue::Jump);
        }
    }
    if player.jump_held && !controls.pressed(GameAction::Jump, &keys) {
        player.jump_held = false;
        if player.velocity.y > 0.0 {
            player.velocity.y *= tuning.jump_release_velocity_multiplier;
        }
    }

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

    // Horizontal motion first, from the frame's starting height, so a body
    // sinking into a floor it lands on is never pushed sideways by it.
    let previous_feet = transform.translation.y - H_WORLD_UNITS / 2.0;
    let dash_dx =
        player.dash_direction * tuning.dash_distance_h / tuning.dash_duration_s * H_WORLD_UNITS;
    let dx = dash_dx * dash_step + player.velocity.x * normal_step;
    let (mut x, wall_hit) = terrain.move_horizontally(
        transform.translation.x,
        transform.translation.x + dx,
        previous_feet,
    );
    if wall_hit.is_some() {
        player.velocity.x = 0.0;
    }
    let room = &data.rooms[&session.room_id];
    let room_width_world = room.width_h(&data.gameplay) * H_WORLD_UNITS;
    let half_width = PLAYER_COLLISION_WIDTH_H * H_WORLD_UNITS / 2.0;
    x = x.clamp(half_width, room_width_world - half_width);
    transform.translation.x = x;

    if normal_step > 0.0 {
        player.velocity.y -= tuning.gravity_h_s2 * H_WORLD_UNITS * normal_step;
    }
    // Pressing into a wall while falling slides down it slowly.
    let wall_sliding = !player.grounded
        && player.velocity.y < 0.0
        && player.wall_side.is_some_and(|side| side == direction);
    if wall_sliding {
        player.velocity.y = player
            .velocity
            .y
            .max(-walls.slide_max_fall_speed_h_s * H_WORLD_UNITS);
    }
    transform.translation.y += player.velocity.y * dt;
    if player.velocity.y <= 0.0 {
        player.jump_held = false;
    }

    let was_grounded = player.grounded;
    player.grounded = false;
    let mut grounded_material = None;
    let feet = transform.translation.y - H_WORLD_UNITS / 2.0;
    if player.velocity.y <= 0.0 {
        if let Some(block) =
            terrain.landing(x, previous_feet, feet, player.drop_through_timer > 0.0)
        {
            transform.translation.y = block.top + H_WORLD_UNITS / 2.0;
            player.velocity.y = 0.0;
            player.grounded = true;
            player.on_one_way = block.one_way;
            grounded_material = Some(block.material);
        }
    } else if let Some(bottom) =
        terrain.ceiling(x, previous_feet + H_WORLD_UNITS, feet + H_WORLD_UNITS)
    {
        transform.translation.y = bottom - H_WORLD_UNITS / 2.0;
        player.velocity.y = 0.0;
        player.jump_held = false;
    }
    let room_top = room.size.height_h * H_WORLD_UNITS;
    if transform.translation.y + H_WORLD_UNITS / 2.0 > room_top {
        transform.translation.y = room_top - H_WORLD_UNITS / 2.0;
        player.velocity.y = player.velocity.y.min(0.0);
    }

    let feet = transform.translation.y - H_WORLD_UNITS / 2.0;
    player.wall_side = if player.grounded {
        None
    } else {
        terrain.wall_contact(x, feet).or_else(|| {
            if x <= half_width + 1.0 {
                Some(-1.0)
            } else if x >= room_width_world - half_width - 1.0 {
                Some(1.0)
            } else {
                None
            }
        })
    };
    // Reaching a ledge while moving toward its wall climbs onto it.
    if !player.grounded
        && player.wall_jump_lock_timer == 0.0
        && direction != 0.0
        && player.wall_side == Some(direction)
        && let Some(ledge_feet) =
            terrain.ledge(x, feet, direction, walls.ledge_grab_reach_h * H_WORLD_UNITS)
    {
        player.mantle = Some(Mantle {
            from: transform.translation.truncate(),
            to: ledge_feet + Vec2::Y * H_WORLD_UNITS / 2.0,
            elapsed_s: 0.0,
        });
        player.velocity = Vec2::ZERO;
        player.dash_timer = 0.0;
        player.jump_held = false;
        player.wall_side = None;
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
            if let Some(material) = grounded_material {
                sounds.push(SoundCue::Footstep(material));
            }
            player.footstep_timer = data.gameplay.audio.footstep_interval_s;
        }
    } else {
        player.footstep_timer = 0.0;
    }

    let moving = player.velocity.x.abs() > 20.0;
    if player.action_animation_timer <= 0.0 {
        let run_speed = data.player_animations.run_speed_fraction
            * tuning.max_speed_h_s
            * movement_multiplier
            * H_WORLD_UNITS;
        let fall_finished = animation.sheet == "player_fall_loop"
            || (animation.sheet == "player_fall" && animation.frame + 1 == animation.frames);
        let (animation_key, looping) = if player.dash_timer > 0.0 {
            ("player_dash", true)
        } else if wall_sliding {
            ("player_wall_slide", true)
        } else if !player.grounded && player.velocity.y > 0.0 {
            ("player_jump", false)
        } else if !player.grounded && fall_finished {
            ("player_fall_loop", true)
        } else if !player.grounded {
            ("player_fall", false)
        } else if player.velocity.x.abs() >= run_speed {
            ("player_run", true)
        } else if moving {
            ("player_walk", true)
        } else {
            ("player_idle", true)
        };
        set_animation(&mut sprite, &mut animation, &sheets, animation_key, looping);
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
