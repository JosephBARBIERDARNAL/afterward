use super::*;
use crate::game::water::{WaterSurface, Waters};

mod animation;
mod body;

use animation::{AnimationState, PlayerSprite};
pub(super) use body::{PlayerAction, PlayerBody, Strike};

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
    water_query: Query<&WaterSurface>,
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
    let water_tuning = &tuning.water;
    let wall_tuning = &tuning.wall;
    let clips = &data.player_animations.clips;
    let platforms = Platforms(room_query.iter().collect());
    let waters = Waters(water_query.iter().collect());
    player.action_animation_timer = (player.action_animation_timer - dt).max(0.0);
    if player.action_animation_timer == 0.0 {
        player.action = None;
    }
    player.wall_control_lock_s = (player.wall_control_lock_s - dt).max(0.0);
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

    let previous_feet = transform.translation.y - H_WORLD_UNITS / 2.0;
    let previous_x = transform.translation.x;
    let pool = waters.around(previous_x, previous_feet);
    player.swimming = pool.is_some();
    player.covering = player.grounded && !player.swimming && held(GameAction::Cover);
    player.crouching =
        player.grounded && !player.swimming && !player.covering && held(GameAction::MoveDown);

    let movement_multiplier =
        session.bonus_multiplier(&data, "fleet_feet", "movement_speed_multiplier");
    let jog_speed = data.player_animations.jog_speed_fraction
        * tuning.max_speed_h_s
        * movement_multiplier
        * H_WORLD_UNITS;
    let mut direction: f32 = 0.0;
    if held(GameAction::MoveLeft) {
        direction -= 1.0;
    }
    if held(GameAction::MoveRight) {
        direction += 1.0;
    }
    // Cover plants the player, and a wall push owns them until its lock runs
    // out, so neither takes steering.
    if player.covering || player.wall_control_lock_s > 0.0 {
        direction = 0.0;
    }
    if direction != 0.0 {
        player.facing = direction.signum();
        let max_speed_h_s = if player.swimming {
            water_tuning.swim_speed_h_s
        } else if player.crouching {
            tuning.crouch_speed_h_s
        } else if player.grounded {
            tuning.max_speed_h_s
        } else {
            tuning.air_max_speed_h_s
        };
        let target_speed = direction * max_speed_h_s * movement_multiplier * H_WORLD_UNITS;
        let acceleration = if player.swimming {
            water_tuning.drag_h_s2
        } else {
            tuning.acceleration_h_s2
        };
        player.velocity.x = move_towards(
            player.velocity.x,
            target_speed,
            acceleration * H_WORLD_UNITS * dt,
        );
    } else {
        let deceleration = if player.swimming {
            water_tuning.drag_h_s2
        } else {
            tuning.deceleration_h_s2
        };
        player.velocity.x = move_towards(player.velocity.x, 0.0, deceleration * H_WORLD_UNITS * dt);
    }

    if let Some(pool) = pool {
        // In the water the stroke replaces the jump: Jump swims up, letting go
        // sinks, and at the surface it hops the player out onto the bank.
        let center = transform.translation.y;
        let at_the_surface = center > pool.top - water_tuning.submerged_depth_h * H_WORLD_UNITS;
        if tapped(GameAction::Jump) && at_the_surface {
            player.velocity.y = water_tuning.exit_impulse_h_s * H_WORLD_UNITS;
            player.jumps_used = 0;
            sounds.push(SoundCue::Jump);
        } else {
            // The stroke carries the body up while Jump is held and lets it
            // sink without, and the surface is as high as it goes.
            let mut target = if held(GameAction::Jump) {
                water_tuning.rise_speed_h_s
            } else {
                -water_tuning.sink_speed_h_s
            };
            if at_the_surface && target > 0.0 {
                target = 0.0;
            }
            player.velocity.y = move_towards(
                player.velocity.y,
                target * H_WORLD_UNITS,
                water_tuning.drag_h_s2 * H_WORLD_UNITS * dt,
            );
        }
        player.jump_held = held(GameAction::Jump);
    } else if tapped(GameAction::Jump) && !player.covering {
        if let Some(wall) = player.wall_cling {
            // Pushing off a wall throws the player up and away from it, and
            // turns them to face the way they are thrown.
            player.velocity.y = wall_tuning.jump_impulse_h_s * H_WORLD_UNITS;
            player.velocity.x = -wall * wall_tuning.jump_push_h_s * H_WORLD_UNITS;
            player.facing = -wall;
            player.wall_cling = None;
            player.wall_control_lock_s = wall_tuning.control_lock_s;
            player.jumps_used = 0;
            player.jump_held = true;
            player.grounded = false;
            player.start_action(PlayerAction::WallJump, clips.wall_jump.duration_s());
            sounds.push(SoundCue::Jump);
        } else {
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
    }
    if player.jump_held && !held(GameAction::Jump) && !player.swimming {
        player.jump_held = false;
        if player.velocity.y > 0.0 {
            player.velocity.y *= tuning.jump_release_velocity_multiplier;
        }
    }

    // Gravity before the horizontal step, so the frame's whole vertical reach
    // is known: a body rising over a ledge this frame also clears its side.
    // Water holds the body up instead, and a wall it clings to slows the fall.
    if !player.swimming {
        player.velocity.y -= tuning.gravity_h_s2 * H_WORLD_UNITS * dt;
        if player.wall_cling.is_some() {
            player.velocity.y = player
                .velocity
                .y
                .max(-wall_tuning.slide_speed_h_s * H_WORLD_UNITS);
        }
    }
    let dx = player.velocity.x * dt;
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
        // A swimmer resting on a pool's floor is still swimming, not standing.
        player.grounded = !player.swimming;
        grounded_footstep = Some(platform.footstep);
    }
    let room_top = room.size.height_h * H_WORLD_UNITS;
    if transform.translation.y + H_WORLD_UNITS / 2.0 > room_top {
        transform.translation.y = room_top - H_WORLD_UNITS / 2.0;
        player.velocity.y = player.velocity.y.min(0.0);
    }

    // Clinging to a wall: airborne, out of the water, falling, and pressed
    // into a platform's side by the player's own steering.
    let feet = transform.translation.y - H_WORLD_UNITS / 2.0;
    player.wall_cling = (!player.grounded
        && !player.swimming
        && player.velocity.y <= 0.0
        && player.wall_control_lock_s == 0.0
        && direction != 0.0
        && platforms.wall_beside(transform.translation.x, feet, direction))
    .then_some(direction);

    // Breaking the surface: falling in fast enough plays the dive, and the
    // water takes the fall out of the body either way.
    let in_water = waters.around(transform.translation.x, feet).is_some();
    let entered_water = in_water && !player.swimming;
    player.swimming = in_water;
    if entered_water {
        player.wall_cling = None;
        player.jumps_used = 0;
        if player.velocity.y < -water_tuning.dive_speed_h_s * H_WORLD_UNITS && alive {
            player.start_action(PlayerAction::Dive, clips.dive.duration_s());
        }
        player.velocity.y = player
            .velocity
            .y
            .max(-water_tuning.sink_speed_h_s * H_WORLD_UNITS);
        sounds.push(SoundCue::Splash);
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
    } else if matches!(player.action, Some(PlayerAction::Land)) {
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
