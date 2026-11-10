use super::*;

mod animation;
mod body;

use animation::{AnimationState, PlayerSprite};
pub(super) use body::{BonusAura, PlayerAction, PlayerBody, Strike, Thrown};

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
    let wall_tuning = &tuning.wall;
    let clips = &data.player_animations.clips;
    let platforms = Platforms(room_query.iter().collect());
    player.action_animation_timer = (player.action_animation_timer - dt).max(0.0);
    if player.action_animation_timer == 0.0 {
        player.action = None;
    }
    player.wall_control_lock_s = (player.wall_control_lock_s - dt).max(0.0);
    // The roll runs on its own clock and ends with its clip.
    player.roll_s = player
        .roll_s
        .filter(|_| matches!(player.action, Some(PlayerAction::Roll)))
        .map(|elapsed_s| elapsed_s + dt);
    let mut sprite_view = PlayerSprite {
        sprite: &mut sprite,
        animation: &mut animation,
        sheets: &sheets,
    };

    // A dying player only falls; every control is ignored. The player also
    // keeps living, without input, behind the room-clear flash and the bonus
    // screen: an airborne player lands, then idles.
    let alive = player.dying_s.is_none();
    // Getting up off the floor takes as long as it takes: Last Stand gives
    // the player their life back, not the frames.
    let rising = matches!(player.action, Some(PlayerAction::Rise));
    let controllable = alive && !rising && *screen.get() == GameScreen::Playing;
    let held = |action| controllable && controls.pressed(action, &keys);
    let tapped = |action| controllable && controls.just_pressed(action, &keys);

    let previous_feet = transform.translation.y - H_WORLD_UNITS / 2.0;
    let previous_x = transform.translation.x;
    let rolling = player.roll_s.is_some();
    player.covering = player.grounded && !rolling && held(GameAction::Cover);
    player.crouching =
        player.grounded && !player.covering && !rolling && held(GameAction::MoveDown);

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
    // Cover plants the player, and the push off a wall owns them until its
    // lock runs out: no steering then, and no drag either, so the shove
    // carries them clear instead of being rubbed out the frame it is given.
    if player.covering {
        direction = 0.0;
    }
    if player.slamming {
        // The slam is a straight drop: nothing steers it.
        player.velocity.x = 0.0;
    } else if player
        .strike
        .is_some_and(|strike| strike.contact_in_s > 0.0)
    {
        // A blow on its way in carries the player with it: the lunge
        // `combat/player_melee.rs` gave them holds until it connects.
    } else if let Some(elapsed_s) = player.roll_s {
        // The roll carries the player the way they already faced, and cannot
        // be steered once it is under way.
        let _ = elapsed_s;
        player.velocity.x = player.facing * tuning.roll.speed_h_s * H_WORLD_UNITS;
    } else if player.wall_control_lock_s == 0.0 {
        if direction != 0.0 {
            player.facing = direction.signum();
        }
        let max_speed_h_s = if player.crouching {
            tuning.crouch_speed_h_s
        } else if player.grounded {
            tuning.max_speed_h_s
        } else {
            tuning.air_max_speed_h_s
        };
        let rate_h_s2 = if direction == 0.0 {
            tuning.deceleration_h_s2
        } else {
            tuning.acceleration_h_s2
        };
        player.velocity.x = move_towards(
            player.velocity.x,
            direction * max_speed_h_s * movement_multiplier * H_WORLD_UNITS,
            rate_h_s2 * H_WORLD_UNITS * dt,
        );
    }

    // A Jump that cannot be obeyed yet is remembered for a moment, so one
    // pressed just before landing, or just before a roll ends, still counts.
    player.jump_buffer_s = if tapped(GameAction::Jump) {
        tuning.jump_buffer_s
    } else {
        (player.jump_buffer_s - dt).max(0.0)
    };
    let jump_asked = controllable && player.jump_buffer_s > 0.0;
    if jump_asked
        && player.grounded
        && held(GameAction::MoveDown)
        && !player.covering
        && player.roll_s.is_none()
    {
        // Down and Jump together: a roll rather than a hop.
        player.jump_buffer_s = 0.0;
        player.roll_s = Some(0.0);
        // Decoy leaves a double of the player here: `combat::plant_decoy`
        // takes the spot once the roll has set off from it.
        player.roll_origin = Some(transform.translation);
        player.velocity.x = player.facing * tuning.roll.speed_h_s * H_WORLD_UNITS;
        player.start_action(PlayerAction::Roll, clips.roll.duration_s());
    } else if jump_asked && !player.covering && player.roll_s.is_none() {
        if let Some(wall) = player.wall_cling {
            player.jump_buffer_s = 0.0;
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
                player.jump_buffer_s = 0.0;
                player.velocity.y = tuning.jump_impulse_h_s * jump_factor * H_WORLD_UNITS;
                player.jumps_used += 1;
                player.jump_held = true;
                player.grounded = false;
                sounds.push(SoundCue::Jump);
            }
        }
    }
    if player.jump_held && !held(GameAction::Jump) {
        player.jump_held = false;
        if player.velocity.y > 0.0 {
            player.velocity.y *= tuning.jump_release_velocity_multiplier;
        }
    }

    // Gravity before the horizontal step, so the frame's whole vertical reach
    // is known: a body rising over a ledge this frame also clears its side.
    // A wall the body clings to slows the fall.
    if player.slamming {
        player.velocity.y = -tuning.melee.slam.drop_speed_h_s * H_WORLD_UNITS;
    } else {
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
    // A slope carries the player up it as far as they stepped, and holds
    // them down on it while they walk the other way.
    if player.velocity.y <= 0.0
        && let Some((platform, floor)) =
            platforms.landing(x, previous_feet, feet, (x - previous_x).abs(), was_grounded)
    {
        transform.translation.y = floor + H_WORLD_UNITS / 2.0;
        player.velocity.y = 0.0;
        player.grounded = true;
        grounded_footstep = Some(platform.footstep);
    }
    let room_top = room.size.height_h * H_WORLD_UNITS;
    if transform.translation.y + H_WORLD_UNITS / 2.0 > room_top {
        transform.translation.y = room_top - H_WORLD_UNITS / 2.0;
        player.velocity.y = player.velocity.y.min(0.0);
    }

    // Clinging to a wall: airborne, falling, and pressed into a platform's
    // side by the player's own steering.
    let feet = transform.translation.y - H_WORLD_UNITS / 2.0;
    player.wall_cling = (!player.grounded
        && player.velocity.y <= 0.0
        && player.wall_control_lock_s == 0.0
        && direction != 0.0
        && platforms.wall_beside(transform.translation.x, feet, direction))
    .then_some(direction);

    if player.grounded {
        player.jumps_used = 0;
        if !was_grounded {
            sounds.push(SoundCue::Landing);
            if player.slamming {
                // The floor is reached: `combat/player_melee.rs` breaks it.
                player.slamming = false;
                player.slam_landed = true;
            } else if direction == 0.0 && player.action.is_none() && alive {
                // Touching down in place plays the landing crouch.
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
