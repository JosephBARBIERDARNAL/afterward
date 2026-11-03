use super::*;

mod cleanup;
mod damage;
mod enemy_ai;
mod enemy_jumps;
mod feedback;
mod player_attacks;
mod projectiles;

pub(super) use cleanup::clear_room_combat_effects;
use damage::{set_enemy_animation, spawn_damage_number};
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
