use super::*;

mod animation;
mod body;

use animation::{AnimationState, PlayerSprite};
pub(super) use body::{Mantle, PlayerAction, PlayerBody, Reload};

#[derive(SystemParam)]
pub(super) struct PlayerUpdateResources<'w, 's> {
    time: Res<'w, Time>,
    screen: Res<'w, State<GameScreen>>,
    keys: Res<'w, ButtonInput<KeyCode>>,
    controls: Res<'w, ControlBindings>,
    data: Res<'w, GameData>,
    session: Res<'w, RunSession>,
    sheets: Res<'w, SpriteSheets>,
    sounds: ResMut<'w, SoundQueue>,
    enemies: Query<'w, 's, &'static EnemyBody>,
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
        screen,
        keys,
        controls,
        data,
        session,
        sheets,
        mut sounds,
        enemies,
    } = resources;
    let Ok((mut player, mut transform, mut sprite, mut animation)) = player_query.single_mut()
    else {
        return;
    };
    let dt = time.delta_secs().min(0.05);
    let tuning = &data.gameplay.player;
    let walls = &tuning.walls;
    let clips = &data.player_animations.clips;
    let terrain = Terrain(room_query.iter().collect());
    player.action_animation_timer = (player.action_animation_timer - dt).max(0.0);
    if player.action_animation_timer == 0.0 {
        player.action = None;
    }
    player.dash_shot_timer = (player.dash_shot_timer - dt).max(0.0);
    player.dash_cooldown = (player.dash_cooldown - dt).max(0.0);
    player.wall_jump_lock_timer = (player.wall_jump_lock_timer - dt).max(0.0);
    player.drop_through_timer = (player.drop_through_timer - dt).max(0.0);
    let mut sprite_view = PlayerSprite {
        sprite: &mut sprite,
        animation: &mut animation,
        sheets: &sheets,
    };

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
        sprite_view.play("player_ledge_grab", false);
        sprite_view.sprite.flip_x = player.facing < 0.0;
        return;
    }

    // A dying player only falls; every control is ignored. The player also
    // keeps living, without input, behind the room-clear flash and the bonus
    // screen: an airborne player lands, then idles.
    let alive = player.dying_s.is_none();
    let controllable = alive && *screen.get() == GameScreen::Playing;
    let held = |action| controllable && controls.pressed(action, &keys);
    let tapped = |action| controllable && controls.just_pressed(action, &keys);
    player.covering = player.grounded && player.dash_timer == 0.0 && held(GameAction::Cover);
    player.crouching = player.grounded && !player.covering && held(GameAction::MoveDown);

    let movement_multiplier =
        session.bonus_multiplier(&data, "fleet_feet", "movement_speed_multiplier");
    let jog_speed = data.player_animations.jog_speed_fraction
        * tuning.max_speed_h_s
        * movement_multiplier
        * H_WORLD_UNITS;
    let was_dashing = player.dash_timer > 0.0;
    let mut direction: f32 = 0.0;
    if held(GameAction::MoveLeft) {
        direction -= 1.0;
    }
    if held(GameAction::MoveRight) {
        direction += 1.0;
    }
    if player.covering {
        direction = 0.0;
    }
    // Sprinting lasts until the player stops, turns, crouches, or covers.
    if direction != player.facing || player.crouching || player.covering {
        player.sprinting = false;
    }
    if player.wall_jump_lock_timer > 0.0 {
        // The wall-jump kick carries the player away from the wall.
    } else if direction != 0.0 {
        // The wall jump keeps the player facing away from the wall it left.
        if !was_dashing && player.action != Some(PlayerAction::WallJump) {
            // Reversing on the ground plays a turn-around step.
            if direction != player.facing
                && player.grounded
                && player.velocity.x * direction < -MOVING_SPEED_THRESHOLD
                && !matches!(
                    player.action,
                    Some(PlayerAction::Shot { .. } | PlayerAction::Hit)
                )
            {
                let running = player.velocity.x.abs() >= jog_speed;
                let clip = if running {
                    &clips.turn_run
                } else {
                    &clips.turn
                };
                player.start_action(PlayerAction::Turn { running }, clip.duration_s());
            }
            player.facing = direction.signum();
        }
        let max_speed_h_s = if player.sprinting {
            data.bonus_effect("sprinter", "sprint_speed_h_s")
        } else if player.crouching {
            tuning.crouch_speed_h_s
        } else if player.grounded {
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

    if tapped(GameAction::Jump) && !player.covering {
        let extra_jump = if session.has_bonus("double_jump") {
            data.bonus_effect("double_jump", "extra_midair_jumps") as u8
        } else {
            0
        };
        let jumps_allowed = tuning.jumps_per_takeoff + extra_jump;
        if player.grounded && player.on_one_way && held(GameAction::MoveDown) {
            player.drop_through_timer = tuning.drop_through_s;
            player.grounded = false;
        } else if let (false, Some(side)) = (player.grounded, player.wall_side) {
            player.velocity = Vec2::new(-side * walls.jump_kick_speed_h_s, walls.jump_impulse_h_s)
                * H_WORLD_UNITS;
            player.facing = -side;
            player.jumps_used = player.jumps_used.max(tuning.jumps_per_takeoff);
            player.jump_held = true;
            player.wall_jump_lock_timer = walls.jump_control_lock_s;
            player.wall_contact_s = 0.0;
            player.start_action(PlayerAction::WallJump, clips.wall_jump.duration_s());
            sounds.push(SoundCue::Jump);
        } else if player.grounded || player.jumps_used < jumps_allowed {
            let jump_factor = if player.jumps_used == 0 {
                session
                    .bonus_multiplier(&data, "high_jump", "jump_height_multiplier")
                    .sqrt()
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
    if player.jump_held && !held(GameAction::Jump) {
        player.jump_held = false;
        if player.velocity.y > 0.0 {
            player.velocity.y *= tuning.jump_release_velocity_multiplier;
        }
    }

    if tapped(GameAction::Dash) && player.dash_cooldown == 0.0 && !player.covering {
        player.dash_timer = tuning.dash_duration_s;
        player.dash_cooldown = tuning.dash_cooldown_s
            * session.bonus_multiplier(&data, "dash_recharge", "dash_cooldown_multiplier");
        player.dash_invulnerability_timer = tuning.dash_invulnerability_s;
        player.dash_direction = player.facing;
        sounds.push(SoundCue::Dash);
    }

    let dash_step = player.dash_timer.min(dt);
    player.dash_timer = (player.dash_timer - dash_step).max(0.0);
    let normal_step = dt - dash_step;
    if was_dashing && player.dash_timer == 0.0 {
        player.dash_shot_timer = if session.has_bonus("dash_shot") {
            data.bonus_effect("dash_shot", "post_dash_window_s")
        } else {
            0.0
        };
        // Sprinter: a grounded dash still pushed forward turns into a sprint.
        if session.has_bonus("sprinter") && player.grounded && direction == player.dash_direction {
            player.sprinting = true;
            player.velocity.x = direction
                * data.bonus_effect("sprinter", "sprint_speed_h_s")
                * movement_multiplier
                * H_WORLD_UNITS;
        }
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
        player.sprinting = false;
    }
    let room = &data.rooms[&session.room_id];
    let room_width_world = room.width_h(&data.gameplay) * H_WORLD_UNITS;
    let half_width = PLAYER_COLLISION_WIDTH_H * H_WORLD_UNITS / 2.0;
    x = x.clamp(half_width, room_width_world - half_width);
    transform.translation.x = x;

    if normal_step > 0.0 {
        player.velocity.y -= tuning.gravity_h_s2 * H_WORLD_UNITS * normal_step;
    }
    // Pressing into a wall while falling first clings to it (the latch), then
    // slides down it slowly.
    let wall_sliding = !player.grounded
        && player.velocity.y < 0.0
        && player.wall_side.is_some_and(|side| side == direction);
    if wall_sliding {
        // On a wall the player's back is against it, facing (and aiming) away.
        player.facing = -direction;
        player.wall_contact_s += dt;
        player.velocity.y = if player.wall_contact_s <= walls.latch_s {
            0.0
        } else {
            player
                .velocity
                .y
                .max(-walls.slide_max_fall_speed_h_s * H_WORLD_UNITS)
        };
    } else {
        player.wall_contact_s = 0.0;
    }
    transform.translation.y += player.velocity.y * dt;
    if player.velocity.y <= 0.0 {
        player.jump_held = false;
    }

    let was_grounded = player.grounded;
    player.grounded = false;
    let mut grounded_footstep = None;
    let feet = transform.translation.y - H_WORLD_UNITS / 2.0;
    if player.velocity.y <= 0.0 {
        if let Some(block) =
            terrain.landing(x, previous_feet, feet, player.drop_through_timer > 0.0)
        {
            transform.translation.y = block.top + H_WORLD_UNITS / 2.0;
            player.velocity.y = 0.0;
            player.grounded = true;
            player.on_one_way = block.one_way;
            grounded_footstep = Some(block.footstep);
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
        player.wall_contact_s = 0.0;
    }

    if player.grounded {
        player.jumps_used = 0;
        if !was_grounded {
            sounds.push(SoundCue::Landing);
            // Touching down in place plays the landing crouch.
            if direction == 0.0 && player.action.is_none() && alive {
                player.start_action(PlayerAction::Land, clips.land.duration_s());
            }
        }
    } else if matches!(
        player.action,
        Some(PlayerAction::Land | PlayerAction::Turn { .. })
    ) {
        player.action = None;
    }
    let moving = player.velocity.x.abs() > MOVING_SPEED_THRESHOLD;
    if player.grounded && moving {
        player.footstep_timer = if was_grounded {
            (player.footstep_timer - dt).max(0.0)
        } else {
            data.gameplay.audio.footstep_interval_s
        };
        if player.footstep_timer == 0.0 {
            if let Some(footstep) = grounded_footstep {
                sounds.push(SoundCue::Footstep(footstep));
            }
            player.footstep_timer = data.gameplay.audio.footstep_interval_s;
        }
    } else {
        player.footstep_timer = 0.0;
    }

    let state = AnimationState {
        moving,
        wall_sliding,
        latch_s: walls.latch_s,
        jog_speed,
        dt,
        enemies_alive: enemies.iter().any(|enemy| enemy.health > 0.0),
        health_fraction: session.health / session.max_health.max(1.0),
    };
    sprite_view.update(&mut player, &data.player_animations, &state);
    sprite_view.sprite.flip_x = player.facing < 0.0;
}
