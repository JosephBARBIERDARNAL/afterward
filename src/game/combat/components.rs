//! Bullets, enemy projectiles, bursts, damage numbers, and health pickups.

use super::*;

#[derive(Component)]
pub(crate) struct CombatProjectile {
    pub(crate) velocity_x: f32,
    pub(crate) damage: f32,
    pub(crate) life_s: f32,
    pub(crate) remaining_range: f32,
    pub(crate) half_size: Vec2,
}

/// A piece of a fiery burst (its hot core or a flying ember): it drifts,
/// swells by `growth`, cools from yellow to red, and fades out.
#[derive(Component)]
pub(crate) struct ImpactEffect {
    pub(crate) timer: Timer,
    pub(crate) velocity: Vec2,
    pub(crate) growth: f32,
    pub(crate) color: Color,
}

/// A bullet fired by the player; it stops at the first enemy or solid block.
#[derive(Component)]
pub(crate) struct PlayerBullet {
    pub(crate) velocity: Vec2,
    pub(crate) damage: f32,
    pub(crate) remaining_range: f32,
    pub(crate) half_size: Vec2,
    /// The last round of a magazine, which plays the heavier hit sound.
    pub(crate) last_round: bool,
    /// Where the shooter's body is: the first step traces from here, so an
    /// enemy pressed against the player is hit point-blank.
    pub(crate) fired_from: Option<Vec2>,
}

#[derive(Component)]
pub(crate) struct DamageNumber {
    pub(crate) timer: Timer,
    pub(crate) color: Color,
}

#[derive(Component)]
pub(crate) struct HealthPickup {
    pub(crate) healing: f32,
}

impl HealthPickup {
    /// Its sprite's key in `SpriteSheets`.
    pub(crate) const SHEET: &'static str = "health_pickup";
}
