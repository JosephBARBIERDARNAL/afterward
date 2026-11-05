//! Enemy bodies, their attacks, and their health bars.

use super::*;

#[derive(Component)]
pub(crate) struct EnemyBody {
    pub(crate) type_id: EnemyTypeId,
    pub(crate) health: f32,
    pub(crate) max_health: f32,
    pub(crate) movement_speed_h_s: f32,
    pub(crate) home_y: f32,
    pub(crate) feet_offset: f32,
    pub(crate) visible_top_offset: f32,
    pub(crate) vertical_velocity: f32,
    pub(crate) grounded: bool,
    pub(crate) jump_height_h: f32,
    pub(crate) jump_cooldown: f32,
    pub(crate) jump: Option<EnemyJump>,
    pub(crate) drop_through_timer: f32,
    pub(crate) behavior: String,
    pub(crate) procedural_movement: Option<ProceduralMovementState>,
    pub(crate) preferred_distance_h: Option<(f32, f32)>,
    pub(crate) attacks: Vec<EnemyAttack>,
    pub(crate) engaged: bool,
    pub(crate) attack_cooldown: f32,
    pub(crate) windup_remaining: f32,
    pub(crate) combo_hit_pending: bool,
    pub(crate) active_attack: usize,
    pub(crate) next_attack: usize,
    pub(crate) hurt_timer: f32,
    pub(crate) knockback_velocity: f32,
    pub(crate) phase_two: Option<(f32, f32)>,
    pub(crate) phase_two_announced: bool,
    pub(crate) marked_timer: f32,
    pub(crate) movement_sound_timer: f32,
}

#[derive(Clone, Copy)]
pub(crate) struct EnemyJump {
    pub(crate) velocity_x: f32,
    pub(crate) slam: bool,
    pub(crate) landed: bool,
}

pub(crate) struct ProceduralMovementState {
    pub(crate) phase_radians: f32,
    pub(crate) bob_height_h: f32,
    pub(crate) cycles_per_h: f32,
    pub(crate) lean_radians: f32,
}

#[derive(Clone)]
pub(crate) struct EnemyAttack {
    pub(crate) kind: String,
    pub(crate) damage: f32,
    pub(crate) hits: u8,
    pub(crate) reach_h: f32,
    pub(crate) projectile_speed_h_s: f32,
    pub(crate) landing_radius_h: f32,
    pub(crate) cooldown_s: f32,
    pub(crate) windup_s: f32,
}

impl From<&EnemyAttackConfig> for EnemyAttack {
    fn from(config: &EnemyAttackConfig) -> Self {
        Self {
            kind: config.kind.clone(),
            damage: config.damage.or(config.damage_per_hit).unwrap_or(0.0),
            hits: config.hits.unwrap_or(1),
            reach_h: config.reach_h.or(config.landing_radius_h).unwrap_or(1.2),
            projectile_speed_h_s: config.projectile_speed_h_s.unwrap_or(0.0),
            landing_radius_h: config.landing_radius_h.unwrap_or(0.0),
            cooldown_s: config.cooldown_s,
            windup_s: config.windup_s,
        }
    }
}

#[derive(Component)]
pub(crate) struct DeadEnemy {
    pub(crate) timer: Timer,
}

#[derive(Component)]
pub(crate) struct EnemyHealthBar {
    pub(crate) enemy: Entity,
    pub(crate) max_health: f32,
}

#[derive(Component)]
pub(crate) struct EnemyHealthBarBackground {
    pub(crate) enemy: Entity,
}
