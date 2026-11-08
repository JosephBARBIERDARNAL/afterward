//! Enemy types from `enemies/enemy_types.yaml`.

use std::{
    borrow::Borrow,
    collections::BTreeMap,
    fmt::{self, Display, Formatter},
    ops::Deref,
    str::FromStr,
};

use serde::{Deserialize, Deserializer, de::Error as DeserializeError};

use super::{RangeConfig, RectH, SizeH};

#[derive(Debug, Deserialize)]
pub struct EnemyTypes {
    pub detection: EnemyDetectionConfig,
    pub damage_triggers_combat: bool,
    pub jumping: EnemyJumpingConfig,
    pub types: BTreeMap<EnemyTypeId, EnemyTypeConfig>,
}

/// What brings the player into an idle enemy's sight.
#[derive(Debug, Deserialize)]
pub struct EnemyDetectionConfig {
    /// How far the enemy sees, between the two hurtbox centers.
    pub range_h: f32,
    /// How far above or below its own eyes it still sees the player.
    pub height_h: f32,
    /// Whether an enemy that has seen the player keeps chasing out of sight.
    pub retain_target: bool,
}

#[derive(Debug, Deserialize)]
pub struct EnemyJumpingConfig {
    pub jump_height_h: f32,
    pub air_speed_h_s: f32,
    pub cooldown_s: f32,
    pub level_change_threshold_h: f32,
}

#[derive(Debug, Clone, Eq, Ord, PartialEq, PartialOrd)]
pub struct EnemyTypeId(String);

impl FromStr for EnemyTypeId {
    type Err = &'static str;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let mut bytes = value.bytes();
        if !bytes.next().is_some_and(|first| first.is_ascii_lowercase())
            || !bytes.all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
        {
            return Err("enemy type IDs must use lowercase letters, digits, and underscores");
        }
        Ok(Self(value.to_string()))
    }
}

impl<'de> Deserialize<'de> for EnemyTypeId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer)?
            .parse()
            .map_err(D::Error::custom)
    }
}

impl Borrow<str> for EnemyTypeId {
    fn borrow(&self) -> &str {
        &self.0
    }
}

impl Deref for EnemyTypeId {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for EnemyTypeId {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl PartialEq<&str> for EnemyTypeId {
    fn eq(&self, other: &&str) -> bool {
        &**self == *other
    }
}

/// What an enemy is made of: it picks the sound a hit on it makes, and metal
/// enemies carry a shield that soaks their first hit.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EnemyMaterial {
    Polymer,
    Metal,
}

impl EnemyMaterial {
    pub fn is_shielded(self) -> bool {
        self == Self::Metal
    }
}

#[derive(Debug, Deserialize)]
pub struct EnemyTypeConfig {
    pub health: f32,
    pub material: EnemyMaterial,
    pub movement_speed_h_s: f32,
    pub jump_height_h: Option<f32>,
    pub behavior: String,
    /// Where bullets hit the enemy: centered on the body, from the feet up.
    pub hurtbox_h: SizeH,
    pub procedural_movement: Option<ProceduralMovementConfig>,
    pub preferred_distance_h: Option<RangeConfig>,
    pub attack: Option<EnemyAttackConfig>,
    #[serde(default)]
    pub attacks: Vec<EnemyAttackConfig>,
    pub phase_two: Option<BossPhaseConfig>,
}

#[cfg(debug_assertions)]
impl EnemyTypeConfig {
    /// Damage of the type's first attack, which a spawn's `attack_damage`
    /// override replaces on every attack.
    pub(crate) fn attack_damage(&self) -> Option<f32> {
        self.attacks
            .first()
            .or(self.attack.as_ref())
            .and_then(|attack| attack.damage.or(attack.damage_per_hit))
    }
}

#[derive(Debug, Deserialize)]
pub struct ProceduralMovementConfig {
    pub bob_height_h: f32,
    pub cycles_per_h: f32,
    pub lean_degrees: f32,
}

#[derive(Debug, Deserialize)]
pub struct EnemyAttackConfig {
    pub kind: String,
    pub damage: Option<f32>,
    pub damage_per_hit: Option<f32>,
    pub hits: Option<u8>,
    /// Where a strike (or a slam's landing) hurts the player, from the
    /// enemy's feet: `x` points the way it faces, `y` up. The enemy starts
    /// the attack once the player's hurtbox reaches its far edge.
    pub hitbox_h: Option<RectH>,
    pub projectile_speed_h_s: Option<f32>,
    pub projectile_size_h: Option<SizeH>,
    pub cooldown_s: f32,
    pub windup_s: f32,
}

#[derive(Debug, Deserialize)]
pub struct BossPhaseConfig {
    pub health_fraction_at_or_below: f32,
    pub cooldown_multiplier: f32,
}
