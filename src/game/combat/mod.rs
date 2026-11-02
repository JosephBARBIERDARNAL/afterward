use super::*;

mod cleanup;
mod damage;
mod enemy_ai;
mod enemy_jumps;
mod feedback;
mod player_attacks;
mod projectiles;

pub(super) use cleanup::clear_room_combat_effects;
use damage::{
    EnemyDamageResources, EnemyDamageTarget, EnemyHit, damage_enemy, set_enemy_animation,
    spawn_damage_number,
};
pub(super) use enemy_ai::{update_enemy_ai, update_enemy_platform_physics};
use enemy_jumps::{JumpArc, JumpPlan, JumpRoute};
pub(super) use feedback::{
    update_damage_numbers, update_dead_enemies, update_health_bars, update_health_pickups,
    update_impact_effects,
};
pub(super) use player_attacks::player_attacks;
pub(super) use projectiles::update_projectiles;

fn has_bonus(session: &RunSession, id: &str) -> bool {
    session.owned_bonuses.iter().any(|owned| owned == id)
}

fn bonus_multiplier(session: &RunSession, data: &GameData, id: &str, effect: &str) -> f32 {
    if has_bonus(session, id) {
        data.bonus_effect(id, effect)
    } else {
        1.0
    }
}

fn attack_speed_multiplier(
    session: &RunSession,
    data: &GameData,
    player: &PlayerBody,
    is_sword: bool,
) -> f32 {
    (if is_sword {
        bonus_multiplier(session, data, "quick_cut", "sword_attack_speed_multiplier")
    } else {
        1.0
    }) * if player.sated_fury_timer > 0.0 {
        data.bonus_effect("sated_fury", "healing_pickup_attack_speed_multiplier")
    } else {
        1.0
    } * (1.0
        + f32::from(player.rhythm_stacks)
            * data.bonus_effect("rhythm_training", "attack_speed_step"))
}

fn player_damage_multiplier(
    session: &RunSession,
    data: &GameData,
    player: &PlayerBody,
    is_gun: bool,
) -> f32 {
    let weapon_bonus = if is_gun {
        bonus_multiplier(session, data, "heavy_caliber", "gun_damage_multiplier")
    } else {
        bonus_multiplier(session, data, "honed_edge", "sword_damage_multiplier")
    };
    let chain = if player.chain_reaction_timer > 0.0 && has_bonus(session, "chain_reaction") {
        1.0 + data.bonus_effect("chain_reaction", "kill_damage_bonus")
    } else {
        1.0
    };
    let movement_bonus = if is_gun
        && player.stationary_timer >= data.bonus_effect("patient_aim", "stationary_time_s")
    {
        bonus_multiplier(
            session,
            data,
            "patient_aim",
            "stationary_gun_damage_multiplier",
        )
    } else {
        1.0
    };
    let air_bonus = if is_gun && !player.grounded {
        bonus_multiplier(
            session,
            data,
            "aerial_hunter",
            "airborne_gun_damage_multiplier",
        )
    } else {
        1.0
    };
    let dash_bonus = if !is_gun && player.dash_strike_timer > 0.0 {
        bonus_multiplier(
            session,
            data,
            "dash_strike",
            "post_dash_sword_damage_multiplier",
        )
    } else {
        1.0
    };
    weapon_bonus * chain * movement_bonus * air_bonus * dash_bonus
}

fn register_hit(player: &mut PlayerBody, session: &RunSession, data: &GameData) {
    if !has_bonus(session, "rhythm_training") {
        return;
    }
    if player.rhythm_timer > 0.0 {
        let maximum = data.bonus_effect("rhythm_training", "attack_speed_max_bonus");
        let step = data.bonus_effect("rhythm_training", "attack_speed_step");
        player.rhythm_stacks = player
            .rhythm_stacks
            .saturating_add(1)
            .min((maximum / step) as u8);
    }
    player.rhythm_timer = data.bonus_effect("rhythm_training", "hit_window_s");
}

struct PlayerDamageResources<'a, 'w, 's> {
    commands: &'a mut Commands<'w, 's>,
    data: &'a GameData,
    session: &'a mut RunSession,
    next_screen: &'a mut NextState<GameScreen>,
    sounds: &'a mut SoundQueue,
}

struct PlayerDamageTarget<'a> {
    player: &'a mut PlayerBody,
    transform: &'a Transform,
}

fn apply_player_damage(
    resources: PlayerDamageResources<'_, '_, '_>,
    target: PlayerDamageTarget<'_>,
    damage: f32,
) {
    let PlayerDamageResources {
        commands,
        data,
        session,
        next_screen,
        sounds,
    } = resources;
    let PlayerDamageTarget { player, transform } = target;
    if player.invulnerability_timer > 0.0 || player.dash_invulnerability_timer > 0.0 {
        return;
    }
    if has_bonus(session, "room_ward") && !session.room_ward_used {
        session.room_ward_used = true;
        return;
    }
    let reduced_damage =
        damage * bonus_multiplier(session, data, "thick_skin", "damage_taken_multiplier");
    session.health = (session.health - reduced_damage).max(0.0);
    player.invulnerability_timer = data.gameplay.player.damage_invulnerability_s;
    spawn_damage_number(commands, transform.translation, reduced_damage, true);
    sounds.push(SoundCue::PlayerHurt);
    if session.health == 0.0 {
        sounds.push(SoundCue::RunEnded);
        next_screen.set(GameScreen::Dead);
    }
}
