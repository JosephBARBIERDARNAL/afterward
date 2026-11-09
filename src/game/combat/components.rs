//! Enemy projectiles, bursts, damage numbers, and health pickups.

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

#[derive(Component)]
pub(crate) struct DamageNumber {
    pub(crate) timer: Timer,
    pub(crate) color: Color,
}

#[derive(Component)]
pub(crate) struct HealthPickup {
    pub(crate) healing: f32,
}

/// The green plusses drifting off something being healed: they hang over a
/// pickup lying on the floor for as long as it lies there, and over the
/// player for one play-through each time Recovery restores a point.
#[derive(Component)]
pub(crate) struct HealEffect {
    pub(crate) timer: Option<Timer>,
}

impl HealEffect {
    /// Its sheet's key in `SpriteSheets`.
    pub(crate) const SHEET: &'static str = "effect_health_regeneration";
    /// How tall it is drawn, in H.
    pub(crate) const HEIGHT_H: f32 = 0.5;
}

impl HealthPickup {
    /// Its sprite's key in `SpriteSheets`.
    pub(crate) const SHEET: &'static str = "health_pickup";
}
