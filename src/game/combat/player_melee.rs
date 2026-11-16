use super::{
    blows::{Blow, LandingResources, Launch, MeleeEnemiesQuery, Pull, land_the_blow},
    *,
};
use crate::game::player::Thrown;

#[derive(SystemParam)]
pub(in crate::game) struct MeleeResources<'w> {
    time: Res<'w, Time>,
    keys: Res<'w, ButtonInput<KeyCode>>,
    controls: Res<'w, ControlBindings>,
    data: Res<'w, GameData>,
    session: ResMut<'w, RunSession>,
    random: ResMut<'w, RandomState>,
    sheets: Res<'w, SpriteSheets>,
    sounds: ResMut<'w, SoundQueue>,
}

/// The player's only weapon, and its three ways of arriving.
///
/// Attack throws the next blow of the chain in `player.melee` of
/// `config/gameplay.yaml`; the blow connects partway through its clip, and
/// once it ends there is `combo_window_s` to ask for the next before the chain
/// drops back to its first step.
///
/// Heavy Attack throws a blow of `heavy` instead and ends the chain: which
/// one depends on how many blows of the chain came before it, and a run's
/// bonuses may buy a finisher in place of the last. It goes through a shield
/// rather than being soaked by it.
///
/// Tapping Attack in mid-air with Down held drives the player at the floor.
/// `player/mod.rs` owns that drop and reports the landing, and the floor
/// breaks to both sides of them.
pub(in crate::game) fn player_melee(
    mut commands: Commands,
    resources: MeleeResources,
    mut players: Query<(&mut PlayerBody, &Transform), Without<EnemyBody>>,
    mut enemies: MeleeEnemiesQuery,
) {
    let MeleeResources {
        time,
        keys,
        controls,
        data,
        mut session,
        mut random,
        sheets,
        mut sounds,
    } = resources;
    let Ok((mut player, transform)) = players.single_mut() else {
        return;
    };
    let dt = time.delta_secs().min(0.05);
    let melee = &data.gameplay.player.melee;
    player.chain_reaction_timer = (player.chain_reaction_timer - dt).max(0.0);
    player.sated_fury_timer = (player.sated_fury_timer - dt).max(0.0);
    player.rhythm_timer = (player.rhythm_timer - dt).max(0.0);
    if player.rhythm_timer == 0.0 {
        player.rhythm_stacks = 0;
    }
    // Both hands are on the ladder.
    if player.climbing.is_some() {
        player.attack_buffer_s = 0.0;
        player.heavy_buffer_s = 0.0;
        return;
    }
    // An Attack pressed while it cannot be thrown is remembered for a moment
    // and thrown the instant it can, so a press made while the blow in hand
    // is still playing asks for the next rather than being lost. The latest
    // press is the one remembered.
    if controls.just_pressed(GameAction::Attack, &keys) {
        player.attack_buffer_s = melee.buffer_s;
        player.heavy_buffer_s = 0.0;
    } else {
        player.attack_buffer_s = (player.attack_buffer_s - dt).max(0.0);
    }
    if controls.just_pressed(GameAction::HeavyAttack, &keys) {
        player.heavy_buffer_s = melee.buffer_s;
        player.attack_buffer_s = 0.0;
    } else {
        player.heavy_buffer_s = (player.heavy_buffer_s - dt).max(0.0);
    }
    if player.dying_s.is_some() {
        player.attack_buffer_s = 0.0;
        player.heavy_buffer_s = 0.0;
        player.strike = None;
        player.slamming = false;
        player.slam_landed = false;
        return;
    }
    // Both hands are on the edge: a press made now waits in the buffer.
    if player.ledge_climb.is_some() {
        return;
    }
    let feet = Vec2::new(
        transform.translation.x,
        transform.translation.y - 0.5 * H_WORLD_UNITS,
    );
    let reach = session.bonus_multiplier(&data, "long_reach", "strike_reach_multiplier");
    let facing = player.facing;
    let chain = &melee.steps;
    // What Undertow gives the slam, and Riptide the dash's shock with it.
    let pull = || Pull {
        radius_h: data.bonus_effect("undertow", "slam_pull_radius_h"),
        speed_h_s: data.bonus_effect("undertow", "slam_pull_h_s"),
    };
    // A dash has just ended: with Aftershock, the floor it ends on shocks
    // everything standing to either side.
    if player.dash_ended && session.has_bonus("aftershock") {
        let reach = data.bonus_effect("aftershock", "dash_shock_reach_h") * H_WORLD_UNITS;
        // The shock is seen whether or not it reaches anybody.
        play_effect(
            &mut commands,
            &sheets,
            "effect_kill_ring",
            Vec3::new(feet.x, feet.y + 0.4 * H_WORLD_UNITS, 6.0),
            2.0 * reach / H_WORLD_UNITS,
            false,
        );
        let drags = session.has_bonus("riptide");
        // Riptide shows how far its drag reaches.
        if drags {
            commands.write_message(
                BonusTell::new("riptide", TellAt::Player)
                    .with(TellEffect::new("whirlpool", 2.4).at_the_feet()),
            );
        }
        land_the_blow(
            LandingResources {
                commands: &mut commands,
                data: &data,
                session: &mut session,
                random: &mut random,
                sheets: &sheets,
                sounds: &mut sounds,
            },
            &mut player,
            Blow {
                area: Rect::new(
                    feet.x - reach,
                    feet.y,
                    feet.x + reach,
                    feet.y + H_WORLD_UNITS,
                ),
                from_x: feet.x,
                heavy: true,
                damage: data.bonus_effect("aftershock", "dash_shock_damage"),
                pierces_shields: false,
                flinch_s: 0.32,
                finisher: false,
                punch: usize::MAX,
                whiffs: false,
                pull: drags.then(pull),
                launch: None,
                bonus: Some("aftershock".to_owned()),
                boost: 1.0,
            },
            &mut enemies,
        );
    }
    // The slam has reached the floor: it breaks to both sides of the player.
    if player.slam_landed {
        player.slam_landed = false;
        let drags = session.has_bonus("undertow");
        let breaks_shields = session.has_bonus("breaker");
        // Meteor: the slam lands harder, and as an airborne blow.
        let meteor = if session.has_bonus("meteor") {
            data.bonus_effect("meteor", "slam_damage_multiplier")
                * session.bonus_multiplier(
                    &data,
                    "aerial_assault",
                    "airborne_strike_damage_multiplier",
                )
        } else {
            1.0
        };
        // Undertow shows how far its drag reaches, and Meteor lands in a
        // burst as heavy as it hits.
        if drags {
            commands.write_message(
                BonusTell::new("undertow", TellAt::Player)
                    .with(TellEffect::new("whirlpool", 2.4).at_the_feet()),
            );
        }
        if meteor > 1.0 {
            commands.write_message(
                BonusTell::new("meteor", TellAt::Player)
                    .quiet()
                    .with(TellEffect::new("meteor_impact", 1.9).on_the_floor()),
            );
        }
        land_the_blow(
            LandingResources {
                commands: &mut commands,
                data: &data,
                session: &mut session,
                random: &mut random,
                sheets: &sheets,
                sounds: &mut sounds,
            },
            &mut player,
            Blow {
                area: blow_area(melee.slam.hitbox_h, feet, facing, reach, true),
                from_x: feet.x,
                heavy: true,
                damage: melee.slam.damage * meteor,
                pierces_shields: breaks_shields,
                flinch_s: 0.32,
                finisher: false,
                punch: usize::MAX,
                whiffs: false,
                pull: drags.then(pull),
                launch: None,
                bonus: if breaks_shields {
                    Some("breaker".to_owned())
                } else {
                    (meteor > 1.0).then(|| "meteor".to_owned())
                },
                boost: meteor,
            },
            &mut enemies,
        );
    }

    // A blow is in the air: it connects at its contact time, then plays itself
    // out, and the window to chain the next one opens as its clip ends.
    if player.strike.is_some() && !matches!(player.action, Some(PlayerAction::Strike { .. })) {
        player.strike = None;
        player.combo_window_s = melee.combo_window_s
            * session.bonus_multiplier(&data, "deep_breath", "combo_window_multiplier");
    }
    if let Some(mut strike) = player.strike {
        let pending = strike.contact_in_s > 0.0;
        strike.contact_in_s -= dt;
        player.strike = Some(strike);
        if !pending {
            return;
        }
        let step = match strike.thrown {
            Thrown::Step(index) => &chain[index.min(chain.len() - 1)],
            Thrown::Heavy(index) => &melee.heavy[index],
        };
        let area = step.hitbox_at(feet, player.facing, reach);
        if strike.contact_in_s > 0.0 {
            // Still on its way in: it carries the player forward, never
            // slower than they were already running, until it can reach a
            // body — a lunge closes the distance, it does not run through.
            let in_reach = enemies.iter().any(|(_, enemy, enemy_transform, ..)| {
                enemy.health > 0.0 && !enemy.hurtbox(enemy_transform).intersect(area).is_empty()
            });
            player.velocity.x = if in_reach {
                0.0
            } else {
                player.facing
                    * (step.lunge_h_s * H_WORLD_UNITS).max(player.velocity.x * player.facing)
            };
            return;
        }
        let blow = match strike.thrown {
            Thrown::Step(index) => Blow {
                area,
                from_x: feet.x,
                // The chain has already stepped on, so one that wrapped
                // back to its first step means this blow was its last.
                heavy: player.combo_step == 0,
                damage: step.damage,
                pierces_shields: false,
                flinch_s: 0.24,
                finisher: player.combo_step == 0,
                punch: index,
                whiffs: true,
                pull: None,
                launch: None,
                bonus: None,
                boost: 1.0,
            },
            Thrown::Heavy(_) => {
                let overcharge =
                    session.bonus_multiplier(&data, "overcharge", "heavy_damage_multiplier");
                Blow {
                    area,
                    from_x: feet.x,
                    heavy: true,
                    damage: step.damage * overcharge,
                    pierces_shields: true,
                    flinch_s: 0.5,
                    // Thrown after a blow of the chain, it is what ends it.
                    finisher: strike.ends_a_chain,
                    punch: usize::MAX,
                    whiffs: true,
                    pull: None,
                    launch: session.has_bonus("battering_ram").then(|| Launch {
                        damage: data.bonus_effect("battering_ram", "launch_damage"),
                        reach_h: data.bonus_effect("battering_ram", "launch_reach_h"),
                        knockback_multiplier: data
                            .bonus_effect("battering_ram", "launch_knockback_multiplier"),
                    }),
                    // A finisher a bonus adds is called by its name.
                    bonus: step
                        .requires_bonus
                        .clone()
                        .or_else(|| (overcharge > 1.0).then(|| "overcharge".to_owned())),
                    boost: overcharge,
                }
            }
        };
        // Long Reach draws the arc of the blow as far as it reaches.
        if session.has_bonus("long_reach")
            && let Some(arc) = play_effect(
                &mut commands,
                &sheets,
                "vfx_slash",
                area.center().extend(5.5),
                area.height().max(area.width() / 2.0) / H_WORLD_UNITS,
                player.facing < 0.0,
            )
        {
            commands
                .entity(arc)
                .entry::<Sprite>()
                .and_modify(move |mut sprite| sprite.custom_size = Some(area.size()));
        }
        land_the_blow(
            LandingResources {
                commands: &mut commands,
                data: &data,
                session: &mut session,
                random: &mut random,
                sheets: &sheets,
                sounds: &mut sounds,
            },
            &mut player,
            blow,
            &mut enemies,
        );
        return;
    }

    // Nothing in the air: the window to chain closes on its own.
    player.combo_window_s = (player.combo_window_s - dt).max(0.0);
    if player.combo_window_s == 0.0 {
        player.combo_step = 0;
    }
    // A slam is a commitment: nothing more is asked until it lands.
    if player.slamming {
        return;
    }
    let free_to_strike = player.wall_cling.is_none() && player.dash_s.is_none();
    let attack_tapped = player.attack_buffer_s > 0.0 && free_to_strike;
    let heavy_tapped = player.heavy_buffer_s > 0.0 && free_to_strike;
    if attack_tapped {
        player.attack_buffer_s = 0.0;
    }
    if heavy_tapped {
        player.heavy_buffer_s = 0.0;
    }

    if attack_tapped && !player.grounded && controls.pressed(GameAction::MoveDown, &keys) {
        player.slamming = true;
        player.velocity = Vec2::new(0.0, -melee.slam.drop_speed_h_s * H_WORLD_UNITS);
        player.start_action(
            PlayerAction::Slam,
            data.player_animations.clips.ground_slam.duration_s(),
        );
        sounds.push(SoundCue::MeleeSwing);
        return;
    }
    if !(attack_tapped || heavy_tapped) {
        return;
    }

    // Rhythm, and the fury of a fresh pickup, speed the whole chain up,
    // heavy blows included.
    let rhythm = 1.0
        + f32::from(player.rhythm_stacks)
            * data.bonus_effect("rhythm_training", "combo_speed_step");
    let sated_fury = if player.sated_fury_timer > 0.0 {
        data.bonus_effect("sated_fury", "healing_pickup_combo_speed_multiplier")
    } else {
        1.0
    };
    // Bloodrush rides Chain Reaction, and Tailwind grows with the agility
    // bonuses owned.
    let bloodrush = if player.chain_reaction_timer > 0.0 {
        session.bonus_multiplier(&data, "bloodrush", "combo_speed_multiplier")
    } else {
        1.0
    };
    let mut speed = session.bonus_multiplier(&data, "quick_hands", "combo_speed_multiplier")
        * rhythm
        * sated_fury
        * bloodrush
        * (1.0 + session.keystone(&data, "tailwind"));
    let light_blows = player.combo_step;
    let (thrown, step) = if heavy_tapped
        && let Some(index) = melee.heavy_after(light_blows, &|bonus| session.has_bonus(bonus))
    {
        // Overcharge throws every heavy blow sooner, and flares as it does.
        speed *= session.bonus_multiplier(&data, "overcharge", "heavy_speed_multiplier");
        if session.has_bonus("overcharge") {
            commands.write_message(
                BonusTell::new("overcharge", TellAt::Player)
                    .quiet()
                    .with(TellEffect::new("charge_flare", 1.3)),
            );
        }
        player.combo_step = 0;
        (Thrown::Heavy(index), &melee.heavy[index])
    } else {
        let index = light_blows.min(chain.len() - 1);
        player.combo_step = (index + 1) % chain.len();
        (Thrown::Step(index), &chain[index])
    };
    player.strike = Some(Strike {
        thrown,
        contact_in_s: step.contact_s / speed,
        ends_a_chain: light_blows > 0,
    });
    player.combo_window_s = 0.0;
    player.start_action(
        PlayerAction::Strike {
            strike: step.strike,
            speed,
        },
        step.strike.clip(&data.player_animations.clips).duration_s() / speed,
    );
    sounds.push(SoundCue::MeleeSwing);
}
