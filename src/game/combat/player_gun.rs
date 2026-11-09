use super::*;

#[derive(SystemParam)]
pub(in crate::game) struct PlayerGunResources<'w> {
    time: Res<'w, Time>,
    keys: Res<'w, ButtonInput<KeyCode>>,
    controls: Res<'w, ControlBindings>,
    data: Res<'w, GameData>,
    session: Res<'w, RunSession>,
    random: ResMut<'w, RandomState>,
    sounds: ResMut<'w, SoundQueue>,
}

/// The gun: holding Shoot fires a bullet every fire interval while rounds
/// remain, aimed straight ahead or diagonally with Up/Down. An empty magazine
/// reloads automatically, and Reload refills a partial one early.
pub(in crate::game) fn player_gun(
    mut commands: Commands,
    resources: PlayerGunResources,
    mut players: Query<(&mut PlayerBody, &Transform)>,
) {
    let PlayerGunResources {
        time,
        keys,
        controls,
        data,
        session,
        mut random,
        mut sounds,
    } = resources;
    let Ok((mut player, transform)) = players.single_mut() else {
        return;
    };
    let dt = time.delta_secs().min(0.05);
    let gun = &data.gameplay.player.gun;
    player.fire_cooldown = (player.fire_cooldown - dt).max(0.0);
    player.aim_hold_timer = (player.aim_hold_timer - dt).max(0.0);
    player.chain_reaction_timer = (player.chain_reaction_timer - dt).max(0.0);
    player.sated_fury_timer = (player.sated_fury_timer - dt).max(0.0);
    player.rhythm_timer = (player.rhythm_timer - dt).max(0.0);
    if player.rhythm_timer == 0.0 {
        player.rhythm_stacks = 0;
    }
    if player.dying_s.is_some() {
        return;
    }

    let capacity = session.magazine_capacity(&data);
    if let Some(mut reload) = player.reload.take() {
        reload.remaining_s -= dt;
        if reload.remaining_s > 0.0 {
            player.reload = Some(reload);
        } else {
            player.ammo = capacity;
            player.magazine_healing = 0.0;
        }
    }
    let reload_requested = controls.just_pressed(GameAction::Reload, &keys)
        && player.ammo < capacity
        && !player.covering;
    if player.reload.is_none() && (player.ammo == 0 || reload_requested) {
        let duration_s = gun.reload_s
            / session.bonus_multiplier(&data, "speed_loader", "reload_speed_multiplier");
        player.reload = Some(Reload {
            remaining_s: duration_s,
            duration_s,
        });
        sounds.push(SoundCue::Reload);
    }

    let trigger = controls.pressed(GameAction::Shoot, &keys) && !player.covering;
    if !trigger || player.reload.is_some() {
        return;
    }
    // Holding the trigger keeps the gun raised between shots.
    player.aim_hold_timer = player.aim_hold_timer.max(dt);
    if player.fire_cooldown > 0.0 {
        return;
    }

    let aim = if controls.pressed(GameAction::AimUp, &keys) {
        Aim::Up
    } else if controls.pressed(GameAction::MoveDown, &keys) {
        Aim::Down
    } else {
        Aim::Straight
    };
    let sated_fury = if player.sated_fury_timer > 0.0 {
        data.bonus_effect("sated_fury", "healing_pickup_fire_rate_multiplier")
    } else {
        1.0
    };
    let rhythm = 1.0
        + f32::from(player.rhythm_stacks) * data.bonus_effect("rhythm_training", "fire_rate_step");
    let fire_rate = session.bonus_multiplier(&data, "hair_trigger", "fire_rate_multiplier")
        * sated_fury
        * rhythm;
    player.fire_cooldown = gun.fire_interval_s / fire_rate;

    let last_round = player.ammo == 1;
    player.ammo -= 1;
    let last_round_bonus = if last_round {
        session.bonus_multiplier(&data, "last_round", "last_round_damage_multiplier")
    } else {
        1.0
    };
    let chain_reaction = if player.chain_reaction_timer > 0.0 && session.has_bonus("chain_reaction")
    {
        1.0 + data.bonus_effect("chain_reaction", "kill_damage_bonus")
    } else {
        1.0
    };
    let airborne = if player.grounded {
        1.0
    } else {
        session.bonus_multiplier(&data, "aerial_gunner", "airborne_bullet_damage_multiplier")
    };
    let damage = gun.damage
        * session.bonus_multiplier(&data, "hollow_points", "bullet_damage_multiplier")
        * last_round_bonus
        * chain_reaction
        * airborne;

    let angle = match aim {
        Aim::Straight => 0.0,
        Aim::Up => gun.diagonal_angle_degrees.to_radians(),
        Aim::Down => -gun.diagonal_angle_degrees.to_radians(),
    };
    let direction = Vec2::new(angle.cos() * player.facing, angle.sin());
    let offset = gun.muzzle_offset_h.get(aim);
    let muzzle = transform.translation.truncate()
        + Vec2::new(offset.x * player.facing, offset.y) * H_WORLD_UNITS;
    let size = Vec2::new(gun.bullet_size_h.width, gun.bullet_size_h.height) * H_WORLD_UNITS;
    commands.spawn((
        Sprite::from_color(Color::srgb(1.0, 0.86, 0.42), size),
        Transform::from_translation(muzzle.extend(4.5))
            .with_rotation(Quat::from_rotation_z(direction.to_angle())),
        PlayerBullet {
            velocity: direction * gun.bullet_speed_h_s * H_WORLD_UNITS,
            damage,
            remaining_range: gun.bullet_range_h * H_WORLD_UNITS,
            half_size: Vec2::splat(size.y / 2.0),
            last_round,
            fired_from: Some(Vec2::new(transform.translation.x, muzzle.y)),
        },
        RoomEntity,
    ));
    spawn_fire_burst(
        &mut commands,
        &mut random,
        muzzle,
        direction,
        &gun.muzzle_flash,
    );
    sounds.push(SoundCue::GunShot);

    let animations = &data.player_animations;
    let variants = animations.shooting.get(aim).shot.len();
    let variant = player.shot_variant % variants;
    player.shot_variant = (variant + 1) % variants;
    let running = aim == Aim::Straight
        && player.grounded
        && !player.crouching
        && player.velocity.x.abs() > MOVING_SPEED_THRESHOLD;
    let duration_s = if running {
        let clip = &animations.clips.run_shooting;
        (clip.frames - animations.run_shooting_shot_frame) as f32 / clip.fps
    } else {
        animations.shooting.get(aim).shot[variant].duration_s()
    };
    player.start_action(
        PlayerAction::Shot {
            aim,
            variant,
            running,
        },
        duration_s,
    );
    player.aim = aim;
    player.aim_hold_timer = duration_s + animations.aim_hold_s;
}
