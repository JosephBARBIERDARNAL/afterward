use super::*;

mod animation;
mod body;

use animation::{AnimationState, PlayerSprite};
pub(super) use body::{LedgeClimb, PlayerAction, PlayerBody, Strike, Thrown};

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
    tells: MessageWriter<'w, BonusTell>,
}

pub(super) fn update_player(
    resources: PlayerUpdateResources,
    room_query: Query<&PlatformSurface>,
    ladders: Query<&devices::Ladder>,
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
        mut tells,
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
    // The dash runs on its own clock for `dash.duration_s`, and the next one
    // waits out the cooldown from the moment it ends.
    let dash = &tuning.dash;
    let was_dashing = player.dash_s.is_some();
    player.dash_s = player
        .dash_s
        .filter(|_| matches!(player.action, Some(PlayerAction::Dash { .. })))
        .map(|elapsed_s| elapsed_s + dt)
        .filter(|elapsed_s| *elapsed_s < dash.duration_s);
    player.dash_ended = was_dashing && player.dash_s.is_none();
    player.dash_cooldown_s = if player.dash_ended {
        dash.cooldown_s
    } else {
        (player.dash_cooldown_s - dt).max(0.0)
    };
    if player.dash_ended && matches!(player.action, Some(PlayerAction::Dash { .. })) {
        player.action = None;
    }
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
    player.crouching = player.grounded && player.dash_s.is_none() && held(GameAction::MoveDown);

    let movement_multiplier =
        session.bonus_multiplier(&data, "fleet_feet", "movement_speed_multiplier")
            * (1.0 + session.keystone(&data, "tailwind"));
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
    let animation_state = |moving| AnimationState {
        moving,
        jog_speed,
        dt,
        enemies_alive: enemies.iter().any(|enemy| enemy.health > 0.0),
        health_fraction: session.health / session.max_health.max(1.0),
        climb_pace: movement_multiplier,
        ledge_climb_s: tuning.ledge.climb_s,
    };
    // Climbing over a ledge: the body is carried up beside the edge, then
    // over onto the top, and nothing else moves it meanwhile.
    if let Some(mut ledge) = player.ledge_climb {
        ledge.elapsed_s += dt;
        let progress = (ledge.elapsed_s / tuning.ledge.climb_s).min(1.0);
        let ease = |from: f32, to: f32| {
            let t = ((progress - from) / (to - from)).clamp(0.0, 1.0);
            t * t * (3.0 - 2.0 * t)
        };
        let feet = ledge.from
            + Vec2::new(
                (ledge.to.x - ledge.from.x) * ease(0.4, 1.0),
                (ledge.to.y - ledge.from.y) * ease(0.0, 0.6),
            );
        transform.translation.x = feet.x;
        transform.translation.y = feet.y + H_WORLD_UNITS / 2.0;
        player.velocity = Vec2::ZERO;
        player.ledge_climb = (progress < 1.0 && alive).then_some(ledge);
        if player.ledge_climb.is_none() {
            // On the top: the ordinary movement takes over next frame.
            player.grounded = alive;
            player.jumps_used = 0;
            player.air_dashes_used = 0;
            sounds.push(SoundCue::Landing);
        }
        sprite_view.update(
            &mut player,
            &data.player_animations,
            &animation_state(false),
        );
        sprite_view.sprite.flip_x = player.facing < 0.0;
        return;
    }

    // A ladder: Up or Down inside its column grabs it, and while the player
    // climbs it nothing but the ladder holds them up. What speeds the run
    // speeds the climb.
    let climb = &tuning.climb;
    let climb_speed = climb.speed_h_s * movement_multiplier * H_WORLD_UNITS;
    let feet_now = transform.translation.y - H_WORLD_UNITS / 2.0;
    if player.climbing.is_none()
        && alive
        && player.dash_s.is_none()
        && !player.slamming
        && player.strike.is_none()
        && let Some(ladder) = ladders.iter().map(|ladder| ladder.area).find(|area| {
            (transform.translation.x - area.center().x).abs() <= climb.reach_h * H_WORLD_UNITS
                && (area.min.y - 1.0..=area.max.y + 1.0).contains(&feet_now)
        })
        && ((held(GameAction::MoveUp) && feet_now < ladder.max.y - 1.0)
            || (held(GameAction::MoveDown) && feet_now > ladder.min.y + 1.0))
    {
        player.climbing = Some(ladder);
        player.crouching = false;
    }
    if let Some(ladder) = player.climbing {
        let up_down = f32::from(u8::from(held(GameAction::MoveUp)))
            - f32::from(u8::from(held(GameAction::MoveDown)));
        let feet = (feet_now + up_down * climb_speed * dt).clamp(ladder.min.y, ladder.max.y);
        transform.translation.x = ladder.center().x;
        transform.translation.y = feet + H_WORLD_UNITS / 2.0;
        player.velocity = Vec2::new(0.0, up_down * climb_speed);
        player.grounded = false;
        player.wall_cling = None;
        player.jumps_used = 0;
        player.air_dashes_used = 0;
        player.jump_buffer_s = 0.0;
        if direction != 0.0 {
            player.facing = direction.signum();
        }
        let at_top = feet >= ladder.max.y - 1.0;
        let at_bottom = feet <= ladder.min.y + 1.0;
        if tapped(GameAction::Jump) || !alive {
            // Letting go with a hop, the way the player steers.
            player.climbing = None;
            player.velocity = Vec2::new(
                direction * tuning.air_max_speed_h_s * H_WORLD_UNITS,
                climb.jump_off_impulse_h_s * H_WORLD_UNITS,
            );
            player.jumps_used = 1;
            sounds.push(SoundCue::Jump);
        } else if direction != 0.0 && !at_top && up_down == 0.0 {
            // Left or Right alone lets go of the ladder that way: the player
            // drops off it already moving.
            player.climbing = None;
            player.velocity = Vec2::new(
                direction * tuning.air_max_speed_h_s * movement_multiplier * H_WORLD_UNITS,
                0.0,
            );
        } else if (at_top && direction != 0.0)
            || (at_bottom && held(GameAction::MoveDown))
            || !ladders.iter().any(|other| other.area == ladder)
        {
            // Stepping off its top onto the ledge, or onto the floor at its
            // foot: the ordinary movement takes over next frame.
            player.climbing = None;
            player.velocity = Vec2::ZERO;
        }
        sprite_view.update(
            &mut player,
            &data.player_animations,
            &animation_state(false),
        );
        sprite_view.sprite.flip_x = player.facing < 0.0;
        return;
    }

    // Dash: the way the player steers, else away from the wall they cling
    // to, else the way they face. In mid-air only `air_dashes` of them (Air
    // Step adds one) until something holds the player up again. It cuts a
    // blow in hand short: `combat/player_melee.rs` lets the blow go once its
    // clip is no longer the one playing.
    let air_dashes = dash.air_dashes
        + if session.has_bonus("air_step") {
            session.bonus_effect(&data, "air_step", "extra_air_dashes") as u8
        } else {
            0
        };
    if tapped(GameAction::Dash)
        && player.dash_s.is_none()
        && player.dash_cooldown_s == 0.0
        && !player.slamming
        && (player.grounded || player.air_dashes_used < air_dashes)
    {
        if !player.grounded {
            player.air_dashes_used += 1;
            // The dash Air Step adds shows the air it pushed off.
            if player.air_dashes_used > dash.air_dashes {
                tells.write(
                    BonusTell::new("air_step", TellAt::Spot(transform.translation))
                        .with(TellEffect::new("air_burst", 0.8)),
                );
            }
        }
        let way = if direction != 0.0 {
            direction.signum()
        } else {
            player.wall_cling.map_or(player.facing, |wall| -wall)
        };
        player.facing = way;
        player.dash_direction = way;
        player.dash_s = Some(0.0);
        // Decoy leaves a double of the player here, if they stand:
        // `combat::plant_decoy` takes the spot once the dash has set off.
        player.dash_origin = player.grounded.then_some(transform.translation);
        player.wall_cling = None;
        player.wall_control_lock_s = 0.0;
        player.crouching = false;
        player.start_action(
            PlayerAction::Dash {
                speed: clips.dash.duration_s() / dash.duration_s,
            },
            dash.duration_s,
        );
    }

    // The push off a wall owns the player until its lock runs out: no
    // steering then, and no drag either, so the shove carries them clear
    // instead of being rubbed out the frame it is given.
    if player.slamming {
        // The slam is a straight drop: nothing steers it.
        player.velocity.x = 0.0;
    } else if player.dash_s.is_some() {
        // The dash cannot be steered once it is under way.
        player.velocity.x = player.dash_direction * dash.speed_h_s * H_WORLD_UNITS;
    } else if player
        .strike
        .is_some_and(|strike| strike.contact_in_s > 0.0)
    {
        // A blow on its way in carries the player with it: the lunge
        // `combat/player_melee.rs` gave them holds until it connects.
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
    // pressed just before landing, or just before a dash ends, still counts.
    player.jump_buffer_s = if tapped(GameAction::Jump) {
        tuning.jump_buffer_s
    } else {
        (player.jump_buffer_s - dt).max(0.0)
    };
    let jump_asked = controllable && player.jump_buffer_s > 0.0;
    if jump_asked && player.dash_s.is_none() {
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
                session.bonus_effect(&data, "double_jump", "extra_midair_jumps") as u8
            } else {
                0
            };
            let jumps_allowed = tuning.jumps_per_takeoff + extra_jump;
            if player.grounded || player.jumps_used < jumps_allowed {
                let jump_factor = if player.jumps_used == 0 {
                    session
                        .bonus_multiplier(&data, "high_jump", "jump_height_multiplier")
                        .sqrt()
                } else if session.has_bonus("kick_off") {
                    session.bonus_effect(&data, "kick_off", "extra_jump_impulse_factor")
                } else {
                    session.bonus_effect(&data, "double_jump", "extra_jump_impulse_factor")
                };
                // A jump a bonus lifts or gives shows the air it pushed off,
                // where it pushed off rather than where the player has risen
                // to by the time it is shown.
                let lifted_by = if player.jumps_used >= tuning.jumps_per_takeoff {
                    Some(if session.has_bonus("kick_off") {
                        ("kick_off", TellEffect::new("air_burst", 0.95).at_the_feet())
                    } else {
                        (
                            "double_jump",
                            TellEffect::new("air_burst", 0.7).at_the_feet(),
                        )
                    })
                } else {
                    session.has_bonus("high_jump").then(|| {
                        let mut dust = TellEffect::new("jump_dust", 0.75).on_the_floor();
                        // The dust's frame leaves its bottom quarter empty:
                        // sunk by that much, the cloud sits on the floor.
                        dust.rise_h -= dust.size_h / 4.0;
                        ("high_jump", dust)
                    })
                };
                if let Some((bonus, effect)) = lifted_by {
                    tells.write(
                        BonusTell::new(bonus, TellAt::Spot(transform.translation))
                            .quiet()
                            .with(effect),
                    );
                }
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
    // An air dash holds the player level for its length.
    if player.slamming {
        player.velocity.y = -tuning.melee.slam.drop_speed_h_s * H_WORLD_UNITS;
    } else if player.dash_s.is_some() && !player.grounded {
        player.velocity.y = 0.0;
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

    // Pressed into a platform's side in mid-air by the player's own
    // steering: a ledge within reach is climbed over, and only a true wall,
    // too tall or roofed over, is clung to on the way down.
    let feet = transform.translation.y - H_WORLD_UNITS / 2.0;
    let pressed_into_side = !player.grounded
        && player.dash_s.is_none()
        && player.wall_control_lock_s == 0.0
        && direction != 0.0
        && platforms.wall_beside(transform.translation.x, feet, direction);
    let ledge = (pressed_into_side && alive && !player.slamming && player.strike.is_none())
        .then(|| {
            platforms.ledge(
                transform.translation.x,
                feet,
                direction,
                tuning.ledge.reach_h * H_WORLD_UNITS,
            )
        })
        .flatten();
    if let Some(top) = ledge {
        player.ledge_climb = Some(LedgeClimb {
            from: Vec2::new(transform.translation.x, feet),
            to: top,
            elapsed_s: 0.0,
        });
        player.facing = direction;
        player.velocity = Vec2::ZERO;
        player.jump_held = false;
        if matches!(
            player.action,
            Some(PlayerAction::Land | PlayerAction::WallJump)
        ) {
            player.action = None;
        }
    }
    player.wall_cling =
        (pressed_into_side && ledge.is_none() && player.velocity.y <= 0.0).then_some(direction);

    if player.grounded || player.wall_cling.is_some() {
        player.air_dashes_used = 0;
    }
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

    sprite_view.update(
        &mut player,
        &data.player_animations,
        &animation_state(moving),
    );
    sprite_view.sprite.flip_x = player.facing < 0.0;
}
