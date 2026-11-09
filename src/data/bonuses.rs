//! The bonus pool from `config/bonuses.yaml`.

use std::{
    collections::BTreeMap,
    fmt::{self, Display, Formatter},
    ops::Index,
};

use serde::Deserialize;
use yaml_serde::Value;

use super::HexColor;

#[derive(Debug, Deserialize)]
pub struct BonusesConfig {
    pub categories: BTreeMap<String, BonusCategoryConfig>,
    pub rarities: BonusRarities,
    pub bonuses: Vec<BonusConfig>,
}

/// What each rarity is worth: the colour the bonus screen paints a bonus of
/// that rarity, and how heavily it weighs in the draw against the others.
#[derive(Debug, Deserialize)]
pub struct BonusRarities {
    pub common: BonusRarityConfig,
    pub uncommon: BonusRarityConfig,
    pub rare: BonusRarityConfig,
    pub epic: BonusRarityConfig,
    pub legendary: BonusRarityConfig,
}

impl Index<BonusRarity> for BonusRarities {
    type Output = BonusRarityConfig;

    fn index(&self, rarity: BonusRarity) -> &Self::Output {
        match rarity {
            BonusRarity::Common => &self.common,
            BonusRarity::Uncommon => &self.uncommon,
            BonusRarity::Rare => &self.rare,
            BonusRarity::Epic => &self.epic,
            BonusRarity::Legendary => &self.legendary,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct BonusRarityConfig {
    pub color: HexColor,
    /// How often a bonus of this rarity is drawn, against the weights of the
    /// other bonuses still on offer.
    pub weight: u32,
}

/// How rare a bonus is, which is the only thing its colour says.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BonusRarity {
    Common,
    Uncommon,
    Rare,
    Epic,
    Legendary,
}

impl BonusRarity {
    pub(crate) const ALL: [Self; 5] = [
        Self::Common,
        Self::Uncommon,
        Self::Rare,
        Self::Epic,
        Self::Legendary,
    ];
}

impl Display for BonusRarity {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Common => "common",
            Self::Uncommon => "uncommon",
            Self::Rare => "rare",
            Self::Epic => "epic",
            Self::Legendary => "legendary",
        })
    }
}

#[derive(Debug, Deserialize)]
pub struct BonusCategoryConfig {
    pub color: HexColor,
    pub icon: BonusCategoryIcon,
    /// The item the player picks up to be offered this category, under `src/assets/`.
    pub pickup: String,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BonusCategoryIcon {
    CrossedSwords,
    Shield,
    Wing,
    Light,
    Knot,
}

impl Display for BonusCategoryIcon {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::CrossedSwords => "X",
            Self::Shield => "O",
            Self::Wing => "^",
            Self::Light => "*",
            Self::Knot => "&",
        })
    }
}

#[derive(Debug, Deserialize)]
pub struct BonusConfig {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub rarity: BonusRarity,
    pub effects: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct BonusEffect(f32);

impl TryFrom<&Value> for BonusEffect {
    type Error = &'static str;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        let number = value.as_f64().ok_or("bonus effects must be numeric")? as f32;
        if !number.is_finite() {
            return Err("bonus effects must be finite numbers");
        }
        Ok(Self(number))
    }
}

impl From<BonusEffect> for f32 {
    fn from(effect: BonusEffect) -> Self {
        effect.0
    }
}
