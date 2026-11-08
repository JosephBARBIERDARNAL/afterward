use super::*;

mod animation;
mod body;

use animation::{AnimationState, PlayerSprite};
pub(super) use body::{PlayerAction, PlayerBody, Reload};

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
    let clips = &data.player_animations.clips;
    let platforms = Platforms(room_query.iter().collect());
    player.action_animation_timer = (player.action_animation_timer - dt).max(0.0);
    if player.action_animation_timer == 0.0 {
        player.action = None;
    }
    player.dash_shot_timer = (player.dash_shot_timer - dt).max(0.0);
    player.dash_cooldown = (player.dash_cooldown - dt).max(0.0);
    let mut sprite_view = PlayerSprite {
        sprite: &mut sprite,
        animation: &mut animation,
        sheets: &sheets,
    };

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
    if direction != 0.0 {
        if !was_dashing {
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
        if player.grounded || player.jumps_used < jumps_allowed {
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

    // Gravity before the horizontal step, so the frame's whole vertical reach
    // is known: a body rising over a ledge this frame also clears its side.
    if normal_step > 0.0 {
        player.velocity.y -= tuning.gravity_h_s2 * H_WORLD_UNITS * normal_step;
    }
    let previous_feet = transform.translation.y - H_WORLD_UNITS / 2.0;
    let previous_x = transform.translation.x;
    let dash_dx =
        player.dash_direction * tuning.dash_distance_h / tuning.dash_duration_s * H_WORLD_UNITS;
    let dx = dash_dx * dash_step + player.velocity.x * normal_step;
    let room = &data.rooms[&session.room_id];
    let room_width_world = room.size.width_h * H_WORLD_UNITS;
    let wanted_x = (previous_x + dx).clamp(HALF_BODY_WIDTH, room_width_world - HALF_BODY_WIDTH);
    // Platform sides are solid: the player stops against them. The test takes
    // the frame's highest feet, so a body sinking into the floor it lands on
    // is never pushed sideways and a rise over a ledge is never cut short.
    let reach_feet = previous_feet.max(previous_feet + player.velocity.y * dt);
    let x = platforms.blocked_x(previous_x, wanted_x, reach_feet);
    if x != wanted_x {
        player.velocity.x = 0.0;
    }
    transform.translation.x = x;

    transform.translation.y += player.velocity.y * dt;
    // Platform undersides are solid too: a rising player bumps their head.
    if player.velocity.y > 0.0
        && let Some(bottom) = platforms.ceiling(
            x,
            previous_feet + H_WORLD_UNITS,
            transform.translation.y + H_WORLD_UNITS / 2.0,
        )
    {
        transform.translation.y = bottom - H_WORLD_UNITS / 2.0;
        player.velocity.y = 0.0;
    }
    if player.velocity.y <= 0.0 {
        player.jump_held = false;
    }

    let was_grounded = player.grounded;
    player.grounded = false;
    let mut grounded_footstep = None;
    let feet = transform.translation.y - H_WORLD_UNITS / 2.0;
    if player.velocity.y <= 0.0
        && let Some(platform) = platforms.landing(x, previous_feet, feet)
    {
        transform.translation.y = platform.top + H_WORLD_UNITS / 2.0;
        player.velocity.y = 0.0;
        player.grounded = true;
        grounded_footstep = Some(platform.footstep);
    }
    let room_top = room.size.height_h * H_WORLD_UNITS;
    if transform.translation.y + H_WORLD_UNITS / 2.0 > room_top {
        transform.translation.y = room_top - H_WORLD_UNITS / 2.0;
        player.velocity.y = player.velocity.y.min(0.0);
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
        jog_speed,
        dt,
        enemies_alive: enemies.iter().any(|enemy| enemy.health > 0.0),
        health_fraction: session.health / session.max_health.max(1.0),
    };
    sprite_view.update(&mut player, &data.player_animations, &state);
    sprite_view.sprite.flip_x = player.facing < 0.0;
}
