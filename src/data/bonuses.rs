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
    pub synergy: BonusSynergyConfig,
    pub glossary: Vec<GlossaryTerm>,
    pub bonuses: Vec<BonusConfig>,
}

/// A word bonus descriptions lean on, and what it means in the game.
#[derive(Debug, Deserialize)]
pub struct GlossaryTerm {
    pub term: String,
    /// What is looked for in a description: whole words, in lower case.
    pub words: Vec<String>,
    pub definition: String,
}

/// One box of explanation for a bonus: a glossary term its description
/// uses, or another bonus it names.
#[derive(Debug, Clone, Copy)]
pub(crate) enum Explanation<'a> {
    Term(&'a GlossaryTerm),
    Bonus(&'a BonusConfig),
}

/// Where `word` first stands in `text` as a whole word, any case: neither
/// letter nor digit on either side of it.
struct WholeWord<'a>(&'a str);

impl WholeWord<'_> {
    fn first_in(&self, text: &str) -> Option<usize> {
        let text = text.to_lowercase();
        let word = self.0.to_lowercase();
        let alone = |character: Option<char>| character.is_none_or(|c| !c.is_alphanumeric());
        text.match_indices(&word).map(|(at, _)| at).find(|&at| {
            alone(text[..at].chars().next_back()) && alone(text[at + word.len()..].chars().next())
        })
    }
}

impl BonusesConfig {
    /// What `bonus`'s description leans on, in the order it comes up there:
    /// the glossary terms it uses, then the other bonuses it names.
    pub(crate) fn explain(&self, bonus: &BonusConfig) -> Vec<Explanation<'_>> {
        let text = &bonus.description;
        let mut found = self
            .glossary
            .iter()
            .filter_map(|term| {
                term.words
                    .iter()
                    .filter_map(|word| WholeWord(word).first_in(text))
                    .min()
                    .map(|at| (at, Explanation::Term(term)))
            })
            .chain(
                self.bonuses
                    .iter()
                    .filter(|named| named.id != bonus.id)
                    .filter_map(|named| {
                        WholeWord(&named.name)
                            .first_in(text)
                            .map(|at| (at, Explanation::Bonus(named)))
                    }),
            )
            .collect::<Vec<_>>();
        found.sort_by_key(|(at, _)| *at);
        found
            .into_iter()
            .map(|(_, explanation)| explanation)
            .collect()
    }
}

/// How an offered bonus that works with the build is told apart: the colour
/// of the line naming what it works with, and how many times heavier it
/// weighs in the draw.
#[derive(Debug, Deserialize)]
pub struct BonusSynergyConfig {
    pub color: HexColor,
    pub weight_multiplier: u32,
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
    /// The bonuses a run must own before this one is offered: it is what
    /// two of them do together.
    #[serde(default)]
    pub requires: Vec<String>,
    /// The bonuses this one works with, without needing them: owning one of
    /// them (or being offered one of them while owning this) weighs it
    /// heavier in the draw, and the bonus screen names the ones owned.
    #[serde(default)]
    pub synergies: Vec<String>,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::GameData;
    use std::path::Path;

    #[test]
    fn a_bonus_is_explained_by_the_words_and_bonuses_its_description_names() {
        let data = GameData::load(&Path::new(env!("CARGO_MANIFEST_DIR")).join("src/assets"))
            .expect("the game data should be valid");
        let bonus = |id: &str| {
            data.bonuses
                .bonuses
                .iter()
                .find(|bonus| bonus.id == id)
                .expect("the bonus is in the pool")
        };
        let titles = |id: &str| {
            data.bonuses
                .explain(bonus(id))
                .into_iter()
                .map(|explanation| match explanation {
                    Explanation::Term(term) => term.term.clone(),
                    Explanation::Bonus(named) => named.name.clone(),
                })
                .collect::<Vec<_>>()
        };
        assert_eq!(titles("finisher"), ["Chain", "Blow"]);
        assert_eq!(
            titles("riptide"),
            ["Aftershock", "Undertow", "Dash"],
            "the bonuses a combo names come first, where they stand"
        );
        assert_eq!(titles("undertow"), ["Slam"]);
        assert!(
            titles("heavy_hands").contains(&"Blow".to_owned()),
            "a synonym explains the same term"
        );
    }
}
