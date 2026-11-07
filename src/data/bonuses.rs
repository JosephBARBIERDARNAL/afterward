//! The bonus pool from `config/bonuses.yaml`.

use std::{
    collections::BTreeMap,
    fmt::{self, Display, Formatter},
};

use serde::Deserialize;
use yaml_serde::Value;

use super::HexColor;

#[derive(Debug, Deserialize)]
pub struct BonusesConfig {
    pub categories: BTreeMap<String, BonusCategoryConfig>,
    pub bonuses: Vec<BonusConfig>,
}

#[derive(Debug, Deserialize)]
pub struct BonusCategoryConfig {
    pub color: HexColor,
    pub icon: BonusCategoryIcon,
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
