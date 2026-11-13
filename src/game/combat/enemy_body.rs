//! Enemy bodies, their attacks, and their health bars.

use super::*;
use crate::data::EnemyGuardConfig;

#[derive(Component)]
pub(crate) struct EnemyBody {
    pub(crate) type_id: EnemyTypeId,
    pub(crate) material: EnemyMaterial,
    /// The layers of armor still up. Each soaks one ordinary blow, and while
    /// any is left a hit does not interrupt the enemy.
    pub(crate) armor: u8,
    pub(crate) health: f32,
    pub(crate) max_health: f32,
    pub(crate) movement_speed_h_s: f32,
    pub(crate) home_y: f32,
    pub(crate) feet_offset: f32,
    pub(crate) visible_top_offset: f32,
    /// The hurtbox's width and height, in world units (see `hitboxes.rs`).
    pub(crate) hurtbox_size: Vec2,
    pub(crate) vertical_velocity: f32,
    pub(crate) grounded: bool,
    pub(crate) jump_height_h: f32,
    pub(crate) jump_cooldown: f32,
    pub(crate) jump: Option<EnemyJump>,
    pub(crate) behavior: String,
    pub(crate) procedural_movement: Option<ProceduralMovementState>,
    pub(crate) preferred_distance_h: Option<(f32, f32)>,
    pub(crate) attacks: Vec<EnemyAttack>,
    /// The way the enemy looks, and so the way it sees: -1 left, 1 right. An
    /// idle enemy keeps the facing it was spawned with; a chasing one turns
    /// toward the player.
    pub(crate) facing: f32,
    pub(crate) engaged: bool,
    pub(crate) attack_cooldown: f32,
    pub(crate) windup_remaining: f32,
    pub(crate) combo_hit_pending: bool,
    pub(crate) active_attack: usize,
    pub(crate) next_attack: usize,
    pub(crate) hurt_timer: f32,
    /// What is left of the flare a blow that wounded it left on its sprite.
    pub(crate) flash_s: f32,
    pub(crate) knockback_velocity: f32,
    pub(crate) phase_two: Option<(f32, f32)>,
    pub(crate) phase_two_announced: bool,
    pub(crate) marked_timer: f32,
    pub(crate) movement_sound_timer: f32,
    /// What it is held in that is neither an attack nor a flinch, and how
    /// long is left of it.
    pub(crate) stance: Option<(Stance, f32)>,
    /// Its wind-up is the answer to a blow its guard turned aside, rather
    /// than the attack in hand.
    pub(crate) countering: bool,
    /// The blows that have wounded it with no armor up since it last rolled.
    pub(crate) wounds: u8,
}

/// What holds an enemy where a boss's extra clips need it held.
#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Stance {
    /// Standing while a clip plays out: the sword drawn, the flare of the
    /// second phase, a slam landed, a strike followed through, a guard
    /// lowered unanswered.
    Clip,
    /// The guard of a `parry`, up and waiting for a blow.
    Guard,
    /// Rolling the way `direction` points.
    Roll { direction: f32 },
}

impl EnemyBody {
    /// Holds the enemy where it stands while `clip` of its own sheets plays
    /// once. A type drawn without that clip carries on as it was.
    pub(super) fn play_out(
        &mut self,
        clip: &str,
        sprite: &mut Sprite,
        animation: &mut AnimatedSprite,
        sheets: &SpriteSheets,
    ) {
        let Some(sheet) = sheets.0.get(&format!("{}_{clip}", self.type_id)) else {
            return;
        };
        self.stance = Some((Stance::Clip, sheet.frames as f32 / sheet.fps));
        set_enemy_animation(sprite, animation, sheets, &self.type_id, clip);
    }
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
    /// The hitbox's far edge, in H: the attack starts once the player's
    /// hurtbox reaches it.
    pub(crate) reach_h: f32,
    /// The strike's hitbox in world units, from the enemy's feet, facing
    /// right; `None` for projectile attacks.
    pub(crate) hitbox: Option<Rect>,
    pub(crate) projectile_speed_h_s: f32,
    /// The projectile's size in world units.
    pub(crate) projectile_size: Vec2,
    pub(crate) cooldown_s: f32,
    pub(crate) windup_s: f32,
    pub(crate) clip: Option<String>,
    pub(crate) strike_frame: Option<usize>,
    pub(crate) guard: Option<EnemyGuardConfig>,
}

impl From<&EnemyAttackConfig> for EnemyAttack {
    fn from(config: &EnemyAttackConfig) -> Self {
        Self {
            kind: config.kind.clone(),
            damage: config.damage.or(config.damage_per_hit).unwrap_or(0.0),
            hits: config.hits.unwrap_or(1),
            reach_h: config.hitbox_h.map_or(1.2, |hitbox| hitbox.right()),
            hitbox: config.hitbox_h.map(|hitbox| {
                Rect::from_corners(
                    Vec2::new(hitbox.x, hitbox.y) * H_WORLD_UNITS,
                    Vec2::new(hitbox.right(), hitbox.top()) * H_WORLD_UNITS,
                )
            }),
            projectile_speed_h_s: config.projectile_speed_h_s.unwrap_or(0.0),
            projectile_size: config.projectile_size_h.map_or(Vec2::ZERO, |size| {
                Vec2::new(size.width, size.height) * H_WORLD_UNITS
            }),
            cooldown_s: config.cooldown_s,
            windup_s: config.windup_s,
            clip: config.clip.clone(),
            strike_frame: config.strike_frame,
            guard: config.guard.clone(),
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

/// One layer of armor above an enemy's health bar: frame 0 while it holds,
/// then the break animation, after which it despawns.
#[derive(Component)]
pub(crate) struct EnemyShieldIcon {
    pub(crate) enemy: Entity,
    /// Which layer it stands for, counted from 0: it holds while the enemy
    /// has more layers than that, so the last one in the row goes first.
    pub(crate) layer: u8,
    /// Where it sits in the row, from the middle of the health bar.
    pub(crate) offset_x: f32,
    /// Runs once that layer is gone, stepping through the break frames; the
    /// icon despawns when it finishes.
    pub(crate) breaking: Option<Timer>,
}

#[derive(Component)]
pub(crate) struct EnemyHealthBarBackground {
    pub(crate) enemy: Entity,
}
