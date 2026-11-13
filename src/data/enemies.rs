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

/// What brings the player to an idle enemy's notice.
#[derive(Debug, Deserialize)]
pub struct EnemyDetectionConfig {
    /// How far the enemy sees on the side it faces, between the two hurtbox
    /// centers.
    pub range_h: f32,
    /// How far it sees at its back.
    pub behind_range_h: f32,
    /// How far from an enemy that was struck an idle one hears the blow.
    pub hearing_range_h: f32,
    /// How far above or below its own eyes it still sees the player.
    pub height_h: f32,
    /// Whether an enemy that has seen the player keeps chasing out of sight.
    pub retain_target: bool,
}

impl EnemyDetectionConfig {
    /// How far the enemy sees toward something `ahead_h` in front of it:
    /// less when that is negative, and so at its back.
    pub(crate) fn reach_h(&self, ahead_h: f32) -> f32 {
        if ahead_h >= 0.0 {
            self.range_h
        } else {
            self.behind_range_h
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct EnemyJumpingConfig {
    pub jump_height_h: f32,
    pub air_speed_h_s: f32,
    pub cooldown_s: f32,
    pub level_change_threshold_h: f32,
    /// The deepest drop an enemy steps off a ledge into, after a player
    /// standing below.
    pub max_drop_h: f32,
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

/// What an enemy is made of: it picks the sound a hit on its body makes.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EnemyMaterial {
    Polymer,
    Metal,
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
    pub roll: Option<EnemyRollConfig>,
    /// The layers of armor every enemy of the type walks in with, on top of
    /// the `armor_layers` its room spawn gives it.
    #[serde(default)]
    pub armor_layers: u8,
    /// What it leaves where it dies.
    pub death_blast: Option<DeathBlastConfig>,
    pub phase_two: Option<BossPhaseConfig>,
    /// How the type's sheets are drawn; the older types have theirs in
    /// `world::spawn_enemies`.
    pub sprite: Option<EnemySpriteConfig>,
}

/// How a type's sheets are laid over its body, in source pixels.
#[derive(Debug, Clone, Copy, Deserialize)]
pub struct EnemySpriteConfig {
    /// World units a source pixel is drawn at: a whole number for pixel art.
    pub scale: f32,
    /// Empty rows above and below the figure in a frame.
    pub top_padding_px: f32,
    pub bottom_padding_px: f32,
    /// The height of the entity's origin above its feet. Combat and the AI
    /// measure from that origin, so it sits mid-body.
    pub origin_above_feet_px: f32,
}

/// The roll an enemy answers a beating with: wounded `after_wounds` times
/// with no armor up, it rolls through whoever is hitting it and comes up at
/// their back.
#[derive(Debug, Deserialize)]
pub struct EnemyRollConfig {
    pub after_wounds: u8,
    pub distance_h: f32,
    pub duration_s: f32,
    /// No blow reaches it between these two moments of the roll.
    pub invulnerable_from_s: f32,
    pub invulnerable_to_s: f32,
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
    /// The clip the attack plays, named as the type's sheet is
    /// (`<type>_<clip>`); the type's ordinary attack clip when left out.
    pub clip: Option<String>,
    /// The frame of that clip the strike lands on: the frames before it are
    /// spread over `windup_s`, so the telegraph ends exactly as it hits.
    pub strike_frame: Option<usize>,
    /// What a `parry` does once its guard is up.
    pub guard: Option<EnemyGuardConfig>,
    /// How far from its mark the enemy starts the attack, instead of once
    /// the mark reaches its hitbox's far edge: a pounce or a blink thrown
    /// from across the floor.
    pub start_range_h: Option<f32>,
    /// The attack is passed over while the mark stands nearer than this, for
    /// the next one of the type's `attacks`.
    #[serde(default)]
    pub min_range_h: f32,
    /// The wave a strike sends along the floor ahead of it.
    pub shockwave: Option<ShockwaveConfig>,
    /// The patch of floor a `hazard` sets alight under its mark.
    pub hazard: Option<HazardConfig>,
}

/// A low wave a strike sends along the floor the way the enemy faces, hurting
/// like a shot: it is jumped over, or rolled through.
#[derive(Debug, Clone, Deserialize)]
pub struct ShockwaveConfig {
    pub damage: f32,
    pub speed_h_s: f32,
    pub range_h: f32,
    pub size_h: SizeH,
}

/// The patch a `hazard` attack lays under its mark's feet: it warns for
/// `arm_s`, then burns for `burn_s` and hurts whoever still stands in it.
#[derive(Debug, Clone, Deserialize)]
pub struct HazardConfig {
    pub size_h: SizeH,
    pub arm_s: f32,
    pub burn_s: f32,
}

/// The blast a type leaves where it dies: its body warns for `fuse_s`, then
/// everything within `radius_h` of it burns.
#[derive(Debug, Clone, Deserialize)]
pub struct DeathBlastConfig {
    pub radius_h: f32,
    pub damage: f32,
    pub fuse_s: f32,
    pub burn_s: f32,
}

/// The guard a `parry` holds once `windup_s` has raised it. An ordinary blow
/// landing on it is turned aside and answered with the attack's own damage
/// and hitbox, `counter_windup_s` later; a blow that goes through armor
/// breaks the guard instead.
#[derive(Debug, Clone, Deserialize)]
pub struct EnemyGuardConfig {
    pub hold_s: f32,
    pub counter_clip: String,
    pub counter_strike_frame: usize,
    pub counter_windup_s: f32,
}

#[derive(Debug, Deserialize)]
pub struct BossPhaseConfig {
    pub health_fraction_at_or_below: f32,
    pub cooldown_multiplier: f32,
}
