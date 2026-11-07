use super::*;

mod bullets;
mod cleanup;
mod components;
mod damage;
mod enemy_ai;
mod enemy_body;
mod enemy_jumps;
mod feedback;
mod hitboxes;
mod player_gun;
mod projectiles;
mod shield;

pub(super) use bullets::update_player_bullets;
pub(super) use cleanup::clear_room_combat_effects;
pub(super) use components::{
    CombatProjectile, DamageNumber, HealthPickup, ImpactEffect, PlayerBullet,
};
use damage::{set_enemy_animation, spawn_damage_number};
pub(super) use enemy_ai::{update_enemy_ai, update_enemy_platform_physics};
pub(super) use enemy_body::{
    DeadEnemy, EnemyAttack, EnemyBody, EnemyHealthBar, EnemyHealthBarBackground, EnemyJump,
    EnemyShieldIcon, ProceduralMovementState,
};
use enemy_jumps::{JumpArc, JumpPlan, JumpRoute};
use feedback::spawn_fire_burst;
pub(super) use feedback::{HEALTH_BAR, SHIELDED_HEALTH_BAR};
pub(super) use feedback::{
    update_damage_numbers, update_dead_enemies, update_health_bars, update_health_pickups,
    update_impact_effects,
};
pub(super) use hitboxes::player_hurtbox;
pub(super) use player_gun::player_gun;
pub(super) use projectiles::update_projectiles;
pub(super) use shield::{SHIELD_SHEET, update_enemy_shields};

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
    if player.dying_s.is_some()
        || player.invulnerability_timer > 0.0
        || player.dash_invulnerability_timer > 0.0
    {
        return;
    }
    if session.has_bonus("room_ward") && !session.room_ward_used {
        session.room_ward_used = true;
        return;
    }
    let tuning = &data.gameplay.player;
    let covered = player.covering && (attacker_x - transform.translation.x) * player.facing >= 0.0;
    let cover = if covered {
        sounds.push(SoundCue::CoverBlock);
        tuning.cover_damage_taken_multiplier
    } else {
        1.0
    };
    let reduced_damage =
        damage * session.bonus_multiplier(data, "thick_skin", "damage_taken_multiplier") * cover;
    session.health = (session.health - reduced_damage).max(0.0);
    player.invulnerability_timer = tuning.damage_invulnerability_s;
    spawn_damage_number(commands, transform.translation, reduced_damage, true);
    sounds.push(SoundCue::Damage(DamageSound::Neutral));
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
