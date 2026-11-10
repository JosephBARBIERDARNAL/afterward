use super::{
    blows::{Blow, LandingResources, MeleeEnemiesQuery, land_the_blow},
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
/// Tapping Attack throws the next blow of the chain in `player.melee` of
/// `config/gameplay.yaml`; the blow connects partway through its clip, and
/// once it ends there is `combo_window_s` to ask for the next before the chain
/// drops back to its first step. The chain's last blow is whichever finisher
/// the run's bonuses bought, if any.
///
/// Holding Attack instead, once the blow in hand has ended, gathers for
/// `charge.ready_s`; letting go then throws one heavy blow, which goes through
/// a shield rather than being soaked by it.
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
    if player.dying_s.is_some() {
        player.strike = None;
        player.slamming = false;
        player.slam_landed = false;
        player.charge_s = 0.0;
        return;
    }
    let feet = Vec2::new(
        transform.translation.x,
        transform.translation.y - 0.5 * H_WORLD_UNITS,
    );
    let reach = session.bonus_multiplier(&data, "long_reach", "strike_reach_multiplier");
    let facing = player.facing;
    let chain = melee.chain(&|bonus| session.has_bonus(bonus));
    if chain.is_empty() {
        return;
    }
    // The slam has reached the floor: it breaks to both sides of the player.
    if player.slam_landed {
        player.slam_landed = false;
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
                damage: melee.slam.damage,
                pierces_shields: false,
                flinch_s: 0.32,
                finisher: false,
            },
            &mut enemies,
        );
    }

    // A blow is in the air: it connects at its contact time, then plays itself
    // out, and the window to chain the next one opens as its clip ends.
    if let Some(mut strike) = player.strike {
        if !matches!(
            player.action,
            Some(PlayerAction::Strike { .. } | PlayerAction::ChargedBlow)
        ) {
            player.strike = None;
            player.combo_window_s = melee.combo_window_s
                * session.bonus_multiplier(&data, "deep_breath", "combo_window_multiplier");
            return;
        }
        let pending = strike.contact_in_s > 0.0;
        strike.contact_in_s -= dt;
        player.strike = Some(strike);
        if !pending || strike.contact_in_s > 0.0 {
            return;
        }
        let blow = match strike.thrown {
            Thrown::Step(index) => {
                let step = chain[index.min(chain.len() - 1)];
                Blow {
                    area: step.hitbox_at(feet, player.facing, reach),
                    damage: step.damage,
                    pierces_shields: false,
                    flinch_s: 0.24,
                    // The chain has already stepped on, so one that wrapped
                    // back to its first step means this blow was its last.
                    finisher: player.combo_step == 0,
                }
            }
            Thrown::Charged => Blow {
                area: blow_area(melee.charge.hitbox_h, feet, player.facing, reach, false),
                damage: melee.charge.damage,
                pierces_shields: true,
                flinch_s: 0.5,
                finisher: false,
            },
        };
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
    let free_to_strike = !player.covering && player.wall_cling.is_none() && player.roll_s.is_none();
    let attack_held = controls.pressed(GameAction::Attack, &keys) && free_to_strike;
    let attack_tapped = controls.just_pressed(GameAction::Attack, &keys) && free_to_strike;

    if attack_tapped && !player.grounded && controls.pressed(GameAction::MoveDown, &keys) {
        player.slamming = true;
        player.charge_s = 0.0;
        player.velocity = Vec2::new(0.0, -melee.slam.drop_speed_h_s * H_WORLD_UNITS);
        player.start_action(
            PlayerAction::Slam,
            data.player_animations.clips.ground_slam.duration_s(),
        );
        sounds.push(SoundCue::MeleeSwing);
        return;
    }

    if attack_tapped {
        let step_index = player.combo_step.min(chain.len() - 1);
        let step = chain[step_index];
        // Rhythm, and the fury of a fresh pickup, speed the whole chain up.
        let rhythm = 1.0
            + f32::from(player.rhythm_stacks)
                * data.bonus_effect("rhythm_training", "combo_speed_step");
        let sated_fury = if player.sated_fury_timer > 0.0 {
            data.bonus_effect("sated_fury", "healing_pickup_combo_speed_multiplier")
        } else {
            1.0
        };
        let speed = session.bonus_multiplier(&data, "quick_hands", "combo_speed_multiplier")
            * rhythm
            * sated_fury;
        player.strike = Some(Strike {
            thrown: Thrown::Step(step_index),
            contact_in_s: step.contact_s / speed,
        });
        player.combo_step = (step_index + 1) % chain.len();
        player.combo_window_s = 0.0;
        player.charge_s = 0.0;
        player.velocity.x = player.facing * step.lunge_h_s * H_WORLD_UNITS;
        player.start_action(
            PlayerAction::Strike {
                strike: step.strike,
                speed,
            },
            step.strike.clip(&data.player_animations.clips).duration_s() / speed,
        );
        sounds.push(SoundCue::MeleeSwing);
        return;
    }

    // Attack still held with nothing in the air: the player gathers, and the
    // gather holds its loop for as long as the key does.
    if attack_held {
        player.charge_s += dt;
        if player.charge_s >= melee.charge.ready_s {
            if player.action == Some(PlayerAction::Charging) {
                player.action_animation_timer = melee.charge.ready_s;
            } else {
                player.start_action(PlayerAction::Charging, melee.charge.ready_s);
            }
        }
        return;
    }

    // Let go of a finished gather, and the heavy blow is thrown.
    if player.charge_s >= melee.charge.ready_s {
        player.combo_step = 0;
        player.combo_window_s = 0.0;
        player.strike = Some(Strike {
            thrown: Thrown::Charged,
            contact_in_s: melee.charge.contact_s,
        });
        player.velocity.x = player.facing * melee.charge.lunge_h_s * H_WORLD_UNITS;
        player.start_action(
            PlayerAction::ChargedBlow,
            data.player_animations.clips.charge_brace.duration_s(),
        );
        sounds.push(SoundCue::MeleeSwing);
    }
    player.charge_s = 0.0;
}
