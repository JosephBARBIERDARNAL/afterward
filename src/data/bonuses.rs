//! The bonus pool from `config/bonuses.yaml`.

use std::{
    collections::BTreeMap,
    fmt::{self, Display, Formatter},
    ops::{Index, Range},
};

use serde::Deserialize;

use super::HexColor;

#[derive(Debug, Deserialize)]
pub struct BonusesConfig {
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

/// Where `word` stands in `text` as a whole word, any case: neither letter
/// nor digit on either side of it. Lower-casing only ASCII keeps every byte
/// where it was, so what is found is where it stands in `text` itself.
struct WholeWord<'a>(&'a str);

impl WholeWord<'_> {
    fn all_in(&self, text: &str) -> Vec<Range<usize>> {
        let lowered = text.to_ascii_lowercase();
        let word = self.0.to_ascii_lowercase();
        let alone = |character: Option<char>| character.is_none_or(|c| !c.is_alphanumeric());
        lowered
            .match_indices(&word)
            .map(|(at, _)| at..at + word.len())
            .filter(|at| {
                alone(lowered[..at.start].chars().next_back())
                    && alone(lowered[at.end..].chars().next())
            })
            .collect()
    }
}

impl BonusesConfig {
    pub(crate) fn bonus(&self, id: &str) -> Option<&BonusConfig> {
        self.bonuses.iter().find(|bonus| bonus.id == id)
    }

    /// Every place `text` names another bonus than `except`, in order and
    /// never overlapping: what a description writes in bold.
    pub(crate) fn mentions<'a>(
        &'a self,
        text: &str,
        except: &str,
    ) -> Vec<(Range<usize>, &'a BonusConfig)> {
        let mut found = self
            .bonuses
            .iter()
            .filter(|named| named.id != except)
            .flat_map(|named| {
                WholeWord(&named.name)
                    .all_in(text)
                    .into_iter()
                    .map(move |at| (at, named))
            })
            .collect::<Vec<_>>();
        // The longer name wins where two start together.
        found.sort_by_key(|(at, _)| (at.start, usize::MAX - at.end));
        let mut end = 0;
        found.retain(|(at, _)| {
            let clear = at.start >= end;
            if clear {
                end = at.end;
            }
            clear
        });
        found
    }

    /// What a description (`text`, written for the bonus `except`) leans
    /// on, in the order it comes up there: the glossary terms it uses, then
    /// the other bonuses it names.
    pub(crate) fn explain(&self, text: &str, except: &str) -> Vec<Explanation<'_>> {
        let mut found = self
            .glossary
            .iter()
            .filter_map(|term| {
                term.words
                    .iter()
                    .filter_map(|word| WholeWord(word).all_in(text).into_iter().next())
                    .map(|at| at.start)
                    .min()
                    .map(|at| (at, Explanation::Term(term)))
            })
            .collect::<Vec<_>>();
        let mut named = Vec::new();
        for (at, bonus) in self.mentions(text, except) {
            if !named.contains(&bonus.id) {
                named.push(bonus.id.clone());
                found.push((at.start, Explanation::Bonus(bonus)));
            }
        }
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
    /// How often a bonus at this rarity is drawn, against the weights of the
    /// other bonuses (and rarities) still on offer.
    pub weight: u32,
}

/// How rare a bonus is drawn, which is what its colour says and what its
/// numbers are: a bonus comes at one or several rarities, each its own
/// version of it.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Deserialize)]
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
pub struct BonusConfig {
    pub id: String,
    pub name: String,
    /// What the bonus does, its numbers written in from the version taken
    /// (see `BonusText`).
    pub description: BonusText,
    /// The effects every version of the bonus shares.
    #[serde(default)]
    pub effects: BTreeMap<String, BonusEffect>,
    /// Each rarity the bonus can be drawn at, and the effects that version
    /// has on top of (or in place of) the shared ones.
    pub rarities: BTreeMap<BonusRarity, BTreeMap<String, BonusEffect>>,
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

impl BonusConfig {
    /// An effect of the version at `rarity`: its own, else the shared one.
    pub(crate) fn effect(&self, rarity: BonusRarity, key: &str) -> Option<f32> {
        self.rarities
            .get(&rarity)
            .and_then(|own| own.get(key))
            .or_else(|| self.effects.get(key))
            .map(|effect| effect.0)
    }

    /// Whether any version of the bonus has the effect.
    pub(crate) fn has_effect(&self, key: &str) -> bool {
        self.effects.contains_key(key) || self.rarities.values().any(|own| own.contains_key(key))
    }

    /// The description of the version at `rarity`, its numbers written in.
    pub(crate) fn describe(&self, rarity: BonusRarity) -> String {
        self.description
            .0
            .iter()
            .map(|piece| match piece {
                TextPiece::Words(words) => words.clone(),
                TextPiece::Value { effect, shown } => self
                    .effect(rarity, effect)
                    .map(|value| shown.write(value))
                    .unwrap_or_default(),
            })
            .collect()
    }
}

/// A bonus effect's number: finite, whatever it tunes.
#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(try_from = "f64")]
pub struct BonusEffect(f32);

impl TryFrom<f64> for BonusEffect {
    type Error = &'static str;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        let number = value as f32;
        if !number.is_finite() {
            return Err("bonus effects must be finite numbers");
        }
        Ok(Self(number))
    }
}

/// A description with the numbers of a version left open: `{effect}` writes
/// the effect as it is, `{effect:%}` as a share (0.04 is 4%), `{effect:+%}`
/// as what a multiplier adds (1.2 is 20%), `{effect:-%}` as what it takes
/// away (0.9 is 10%).
#[derive(Debug, Deserialize)]
#[serde(try_from = "String")]
pub struct BonusText(Vec<TextPiece>);

#[derive(Debug)]
enum TextPiece {
    Words(String),
    Value { effect: String, shown: Shown },
}

#[derive(Debug, Clone, Copy)]
enum Shown {
    Plain,
    Share,
    Increase,
    Decrease,
}

impl Shown {
    fn write(self, value: f32) -> String {
        let (number, unit) = match self {
            Self::Plain => (value, ""),
            Self::Share => (value * 100.0, "%"),
            Self::Increase => ((value - 1.0) * 100.0, "%"),
            Self::Decrease => ((1.0 - value) * 100.0, "%"),
        };
        let written = format!("{number:.2}");
        let written = written.trim_end_matches('0').trim_end_matches('.');
        format!("{written}{unit}")
    }
}

impl TryFrom<String> for BonusText {
    type Error = String;

    fn try_from(text: String) -> Result<Self, Self::Error> {
        let mut pieces = Vec::new();
        let mut rest = text.as_str();
        while let Some(open) = rest.find('{') {
            if open > 0 {
                pieces.push(TextPiece::Words(rest[..open].to_owned()));
            }
            let close = rest[open..]
                .find('}')
                .map(|close| open + close)
                .ok_or_else(|| format!("an unclosed {{ in the bonus description {text:?}"))?;
            let inside = &rest[open + 1..close];
            let (effect, shown) = match inside.split_once(':') {
                None => (inside, Shown::Plain),
                Some((effect, "%")) => (effect, Shown::Share),
                Some((effect, "+%")) => (effect, Shown::Increase),
                Some((effect, "-%")) => (effect, Shown::Decrease),
                Some((_, format)) => {
                    return Err(format!(
                        "the bonus description {text:?} writes a number as {format:?}, \
                         which is none of %, +% or -%"
                    ));
                }
            };
            if effect.is_empty() {
                return Err(format!("an empty {{}} in the bonus description {text:?}"));
            }
            pieces.push(TextPiece::Value {
                effect: effect.to_owned(),
                shown,
            });
            rest = &rest[close + 1..];
        }
        if !rest.is_empty() {
            pieces.push(TextPiece::Words(rest.to_owned()));
        }
        Ok(Self(pieces))
    }
}

impl BonusText {
    /// The effects the description writes in.
    pub(crate) fn effects(&self) -> impl Iterator<Item = &str> {
        self.0.iter().filter_map(|piece| match piece {
            TextPiece::Value { effect, .. } => Some(effect.as_str()),
            TextPiece::Words(_) => None,
        })
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
        let titles = |id: &str| {
            let bonus = data.bonuses.bonus(id).expect("the bonus is in the pool");
            let text = bonus.describe(*bonus.rarities.keys().next().expect("a version"));
            data.bonuses
                .explain(&text, id)
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

    #[test]
    fn each_version_writes_its_own_numbers_into_the_description() {
        let data = GameData::load(&Path::new(env!("CARGO_MANIFEST_DIR")).join("src/assets"))
            .expect("the game data should be valid");
        let heavy_hands = data.bonuses.bonus("heavy_hands").expect("in the pool");
        assert_eq!(
            heavy_hands.describe(BonusRarity::Common),
            "Increase strike damage by 20%."
        );
        assert_eq!(
            heavy_hands.describe(BonusRarity::Legendary),
            "Increase strike damage by 40%."
        );
        let thick_skin = data.bonuses.bonus("thick_skin").expect("in the pool");
        assert_eq!(
            thick_skin.describe(BonusRarity::Common),
            "Reduce damage taken by 10%."
        );
        let executioner = data.bonuses.bonus("executioner").expect("in the pool");
        assert_eq!(
            executioner.effect(BonusRarity::Epic, "execute_health_fraction"),
            Some(0.3),
            "a shared effect holds for every version"
        );
    }

    #[test]
    fn a_description_marks_every_other_bonus_it_names() {
        let data = GameData::load(&Path::new(env!("CARGO_MANIFEST_DIR")).join("src/assets"))
            .expect("the game data should be valid");
        let text = "Marked Prey and Executioner together: Executioner strikes.";
        let named = data
            .bonuses
            .mentions(text, "open_wound")
            .into_iter()
            .map(|(at, bonus)| (&text[at], bonus.id.as_str()))
            .collect::<Vec<_>>();
        assert_eq!(
            named,
            [
                ("Marked Prey", "marked_prey"),
                ("Executioner", "executioner"),
                ("Executioner", "executioner"),
            ]
        );
        assert!(
            data.bonuses.mentions(text, "marked_prey").len() == 2,
            "a bonus is not marked in its own description"
        );
    }
}
