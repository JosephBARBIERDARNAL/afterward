use super::*;

mod blows;
mod cleanup;
mod decoy;
pub(in crate::game) use decoy::Decoy;
pub(super) use decoy::plant_decoy;
use decoy::{DecoyQuery, Mark, hurt_the_decoy};
mod components;
mod damage;
mod enemy_ai;
mod enemy_body;
mod enemy_jumps;
mod falls;
mod feedback;
mod hazards;
mod hitboxes;
mod player_melee;
mod projectiles;
mod shield;
mod vision;
mod waves;

pub(super) use cleanup::clear_room_combat_effects;
pub(super) use components::{
    CombatProjectile, DamageNumber, HealEffect, HealthPickup, HitEffect, ImpactEffect,
};
use damage::{set_enemy_animation, spawn_damage_number};
pub(super) use enemy_ai::{update_enemy_ai, update_enemy_platform_physics};
pub(super) use enemy_body::{
    DeadEnemy, EnemyAttack, EnemyBody, EnemyHealthBar, EnemyHealthBarBackground, EnemyJump,
    EnemyShieldIcon, ProceduralMovementState, Stance,
};
use enemy_jumps::{JumpArc, JumpPlan, JumpRoute};
pub(super) use falls::update_falls;
use feedback::play_effect;
pub(super) use feedback::{HEALTH_BAR, SHIELDED_HEALTH_BAR};
pub(super) use feedback::{
    update_damage_numbers, update_dead_enemies, update_healing_over_time, update_health_bars,
    update_health_pickups, update_hit_effects, update_impact_effects,
};
pub(super) use hazards::{Hazard, update_hazards};
use hitboxes::blow_area;
#[cfg(debug_assertions)]
pub(super) use hitboxes::blow_area as melee_blow_area;
pub(super) use hitboxes::player_hurtbox;
pub(super) use player_melee::player_melee;
pub(super) use projectiles::update_projectiles;
pub(super) use shield::{SHIELD_OFFSET_Y, SHIELD_SHEET, update_enemy_shields};
pub(super) use waves::{Arriving, raise_the_arriving, send_in_the_next_wave};

struct PlayerDamageResources<'a, 'w, 's> {
    commands: &'a mut Commands<'w, 's>,
    data: &'a GameData,
    session: &'a mut RunSession,
    sounds: &'a mut SoundQueue,
}

struct PlayerDamageTarget<'a> {
    player: &'a mut PlayerBody,
    transform: &'a Transform,
    /// Where the hit comes from, so Cover only blocks attacks from the front.
    attacker_x: f32,
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
        sounds,
    } = resources;
    let PlayerDamageTarget {
        player,
        transform,
        attacker_x,
    } = target;
    if player.dying_s.is_some() || player.invulnerability_timer > 0.0 {
        return;
    }
    let tuning = &data.gameplay.player;
    // The middle of a roll is the one stretch nothing can reach the player
    // through, which is what a melee fighter has instead of range.
    if player.roll_s.is_some_and(|elapsed_s| {
        (tuning.roll.invulnerable_from_s..=tuning.roll.invulnerable_to_s).contains(&elapsed_s)
    }) {
        return;
    }
    if session.has_bonus("room_ward") && !session.room_ward_used {
        session.room_ward_used = true;
        return;
    }
    let covered = player.covering && (attacker_x - transform.translation.x) * player.facing >= 0.0;
    let cover = if covered {
        sounds.push(SoundCue::CoverBlock);
        tuning.cover_damage_taken_multiplier
            * session.bonus_multiplier(data, "bulwark", "cover_damage_taken_multiplier")
    } else {
        1.0
    };
    let adrenaline = if session.health / session.max_health
        < data.bonus_effect("adrenaline", "low_health_fraction")
    {
        session.bonus_multiplier(data, "adrenaline", "low_health_damage_taken_multiplier")
    } else {
        1.0
    };
    let reduced_damage = damage
        * session.bonus_multiplier(data, "thick_skin", "damage_taken_multiplier")
        * cover
        * adrenaline;
    if !session.playtest.infinite_health {
        session.health = (session.health - reduced_damage).max(0.0);
    }
    session.untouched = false;
    player.invulnerability_timer = tuning.damage_invulnerability_s;
    spawn_damage_number(commands, transform.translation, reduced_damage, true);
    sounds.push(SoundCue::Damage(DamageSound::Neutral));
    // Last Stand spends itself to leave the player on their feet at 1 health:
    // they go down, lie there, and get up again, untouchable until they have.
    if session.health == 0.0 && session.has_bonus("last_stand") && !session.last_stand_used {
        let rise_s = data.player_animations.clips.rise.duration_s();
        session.last_stand_used = true;
        session.health = 1.0;
        player.invulnerability_timer = rise_s;
        player.velocity = Vec2::ZERO;
        player.start_action(PlayerAction::Rise, rise_s);
        return;
    }
    if session.health == 0.0 {
        // The death animation plays out before the run ends.
        player.dying_s = Some(0.0);
        player.action = None;
        player.action_animation_timer = 0.0;
        sounds.push(SoundCue::RunEnded);
    } else if !covered {
        player.start_action(
            PlayerAction::Hit,
            data.player_animations.clips.hit.duration_s(),
        );
    }
}

/// Ends the run once the death animation has played and held.
pub(super) fn update_player_death(
    time: Res<Time>,
    data: Res<GameData>,
    mut players: Query<&mut PlayerBody>,
    mut next_screen: ResMut<NextState<GameScreen>>,
) {
    let Ok(mut player) = players.single_mut() else {
        return;
    };
    let Some(dying_s) = player.dying_s.as_mut() else {
        return;
    };
    *dying_s += time.delta_secs();
    if *dying_s
        >= data.player_animations.clips.death.duration_s() + data.gameplay.player.death_hold_s
    {
        next_screen.set(GameScreen::Dead);
    }
}
