//! The current run: room, health, and the bonus build.

use super::*;

/// How hard a run was asked to be. Nothing reads it yet: the choice is made
/// before the run opens, remembered for the next one, and changes nothing.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) enum Difficulty {
    Easy,
    #[default]
    Normal,
    Hard,
}

impl Difficulty {
    /// The choices, in the order the difficulty screen draws them.
    pub(crate) const ALL: [Self; 3] = [Self::Easy, Self::Normal, Self::Hard];

    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Easy => "Easy",
            Self::Normal => "Normal",
            Self::Hard => "Hard",
        }
    }

    /// The colour the choice's box is edged in, from `difficulty:` of
    /// `config/gameplay.yaml`.
    pub(crate) fn edge(self, colors: &DifficultyColors) -> HexColor {
        match self {
            Self::Easy => colors.easy,
            Self::Normal => colors.normal,
            Self::Hard => colors.hard,
        }
    }
}

/// What the debug editor's playtest asked of the runs that follow it. Like
/// `StartRoom`, it stays until the editor changes it, so a restart from the
/// pause menu replays the room the same way.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct Playtest {
    /// A hit still lands and is still felt, but costs no health. A fall out
    /// of the room still ends the run: there is no floor to carry on from.
    pub(crate) infinite_health: bool,
    /// Rooms open empty, as one authored without enemies does.
    pub(crate) no_enemies: bool,
    /// How many bonuses, drawn at random, a run opens with: every one of
    /// them when there are no more than that.
    pub(crate) starting_bonuses: usize,
}

#[derive(Resource, Debug)]
pub(crate) struct RunSession {
    pub(crate) room_id: u16,
    pub(crate) playtest: Playtest,
    /// What the player asked for on the difficulty screen; it outlives a run,
    /// so the next one opens on the same choice.
    pub(crate) difficulty: Difficulty,
    pub(crate) health: f32,
    pub(crate) max_health: f32,
    pub(crate) rerolls: u8,
    pub(crate) owned_bonuses: Vec<String>,
    /// The category of the bonus item picked up in this room, which the offers
    /// and the rerolls are drawn from.
    pub(crate) bonus_category: Option<String>,
    pub(crate) offered_bonuses: Vec<String>,
    pub(crate) previous_offers: BTreeSet<String>,
    pub(crate) selected_choice: bonus_screen::BonusChoice,
    pub(crate) bonus_reroll_animation_remaining_s: f32,
    /// What is left of the hold the bonus screen keeps after a bonus is
    /// chosen, so the droplet lands on the box before the room comes back.
    pub(crate) bonus_chosen_hold_remaining_s: f32,
    /// The last wave of the room's enemies to have arrived: `wave` of a room
    /// spawn, 0 as the room opens.
    pub(crate) wave: u8,
    /// How long the room has stood empty with a wave still to come.
    pub(crate) wave_wait_s: f32,
    /// Nothing has cost the player health since the room opened.
    pub(crate) untouched: bool,
    /// The room was cleared untouched: its bonus is drawn leaning toward
    /// the rarer ones, and its item says so.
    pub(crate) flawless: bool,
    /// The category the next room's bonus item will be of, drawn as this
    /// room's door unlocks so the door can wear its colour.
    pub(crate) next_bonus_category: Option<String>,
    /// Every enemy is down, so the room's bonus item may be laid out.
    pub(crate) room_cleared: bool,
    pub(crate) door_open: bool,
    pub(crate) room_ward_used: bool,
    /// Deflector has turned aside its one shot of this room.
    pub(crate) deflector_used: bool,
    /// The part of a health point Recovery has restored but not yet paid out.
    pub(crate) idle_healing: f32,
    /// Last Stand is spent once a run, not once a room.
    pub(crate) last_stand_used: bool,
    pub(crate) world_loaded: bool,
}

impl RunSession {
    pub(crate) fn has_bonus(&self, id: &str) -> bool {
        self.owned_bonuses.iter().any(|owned| owned == id)
    }

    /// Whether a bonus may still be offered: not one already owned, and not
    /// one that builds on bonuses the run has yet to take.
    pub(crate) fn may_be_offered(&self, bonus: &crate::data::BonusConfig) -> bool {
        !self.has_bonus(&bonus.id) && bonus.requires.iter().all(|id| self.has_bonus(id))
    }

    /// The categories that can still offer a choice of two.
    pub(crate) fn categories_on_offer<'a>(
        &self,
        data: &'a GameData,
    ) -> Vec<(&'a String, &'a crate::data::BonusCategoryConfig)> {
        data.bonuses
            .categories
            .iter()
            .filter(|(name, _)| {
                data.bonuses
                    .bonuses
                    .iter()
                    .filter(|bonus| bonus.category == **name && self.may_be_offered(bonus))
                    .count()
                    >= 2
            })
            .collect()
    }

    /// Whether the room being played has anyone to fight. One that does not
    /// has nothing to clear: its bonus is there from the first frame.
    pub(crate) fn has_enemies(&self, data: &GameData) -> bool {
        !self.playtest.no_enemies && !data.rooms[&self.room_id].enemy_spawns.is_empty()
    }

    /// How many owned bonuses a keystone counts: those of its own category
    /// for one with `per_bonus_in_category`, those that build on others for
    /// one with `per_combo`, and none for any other bonus.
    fn counted_by(&self, data: &GameData, keystone: &crate::data::BonusConfig) -> usize {
        let counts = |owned: &crate::data::BonusConfig| {
            if keystone.effects.contains_key("per_bonus_in_category") {
                owned.category == keystone.category
            } else {
                keystone.effects.contains_key("per_combo") && !owned.requires.is_empty()
            }
        };
        data.bonuses
            .bonuses
            .iter()
            .filter(|owned| self.has_bonus(&owned.id) && counts(owned))
            .count()
    }

    /// What an owned keystone adds: its `per_bonus_in_category` or its
    /// `per_combo` for every owned bonus it counts (itself included when it
    /// counts its own category), or 0 without it.
    pub(crate) fn keystone(&self, data: &GameData, id: &str) -> f32 {
        data.bonuses
            .bonuses
            .iter()
            .find(|bonus| bonus.id == id && self.has_bonus(id))
            .map_or(0.0, |keystone| {
                (data.bonus_effect(id, "per_bonus_in_category")
                    + data.bonus_effect(id, "per_combo"))
                    * self.counted_by(data, keystone) as f32
            })
    }

    /// What a bonus works with in the build already owned, in words, or
    /// nothing when it stands alone: the bonuses it builds on, the owned ones
    /// it lists (or that list it) as `synergies`, or the owned bonuses a
    /// keystone counts. Such a bonus weighs heavier in the draw, and the bonus
    /// screen says why.
    pub(crate) fn synergy(
        &self,
        data: &GameData,
        bonus: &crate::data::BonusConfig,
    ) -> Option<String> {
        let owned = data
            .bonuses
            .bonuses
            .iter()
            .filter(|owned| self.has_bonus(&owned.id));
        if !bonus.requires.is_empty() {
            let parts = owned
                .filter(|owned| bonus.requires.contains(&owned.id))
                .map(|owned| owned.name.as_str())
                .collect::<Vec<_>>();
            return Some(format!("COMBO: {}", parts.join(" + ")));
        }
        let partners = owned
            .filter(|owned| {
                bonus.synergies.contains(&owned.id) || owned.synergies.contains(&bonus.id)
            })
            .map(|owned| owned.name.as_str())
            .collect::<Vec<_>>();
        if !partners.is_empty() {
            return Some(format!("SYNERGY: {}", partners.join(", ")));
        }
        match self.counted_by(data, bonus) {
            0 => None,
            count if bonus.effects.contains_key("per_combo") => {
                Some(format!("SYNERGY: {count} combos owned"))
            }
            count => Some(format!("SYNERGY: {count} {} owned", bonus.category)),
        }
    }

    /// Pays out what a playtest (or `--bonuses`) asked the run to open with,
    /// each as if chosen, and opens it whole.
    pub(crate) fn grant_starting_bonuses(&mut self, data: &GameData, random: &mut RandomState) {
        let mut pool = data
            .bonuses
            .bonuses
            .iter()
            .filter(|bonus| !self.has_bonus(&bonus.id))
            .collect::<Vec<_>>();
        let granted = self.playtest.starting_bonuses.min(pool.len());
        for _ in 0..granted {
            let bonus = pool.swap_remove(random.index(pool.len()));
            self.owned_bonuses.push(bonus.id.clone());
            if let Some(increase) = bonus
                .effects
                .get("max_health_increase")
                .and_then(yaml_serde::Value::as_f64)
            {
                self.max_health += increase as f32;
            }
        }
        self.health = self.max_health;
    }

    /// An owned bonus's multiplier effect, or 1 without the bonus.
    pub(crate) fn bonus_multiplier(&self, data: &GameData, id: &str, effect: &str) -> f32 {
        if self.has_bonus(id) {
            data.bonus_effect(id, effect)
        } else {
            1.0
        }
    }
}

impl Default for RunSession {
    fn default() -> Self {
        Self {
            room_id: 0,
            playtest: Playtest::default(),
            difficulty: Difficulty::default(),
            health: 100.0,
            max_health: 100.0,
            rerolls: 3,
            owned_bonuses: Vec::new(),
            bonus_category: None,
            offered_bonuses: Vec::new(),
            previous_offers: BTreeSet::new(),
            selected_choice: bonus_screen::BonusChoice::Offer(0),
            bonus_reroll_animation_remaining_s: 0.0,
            bonus_chosen_hold_remaining_s: 0.0,
            wave: 0,
            wave_wait_s: 0.0,
            untouched: true,
            flawless: false,
            next_bonus_category: None,
            room_cleared: false,
            door_open: false,
            room_ward_used: false,
            deflector_used: false,
            idle_healing: 0.0,
            last_stand_used: false,
            world_loaded: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn keystones_count_the_build_and_the_screen_says_what_a_bonus_works_with() {
        let data = GameData::load(&Path::new(env!("CARGO_MANIFEST_DIR")).join("src/assets"))
            .expect("the game data should be valid");
        let bonus = |id: &str| {
            data.bonuses
                .bonuses
                .iter()
                .find(|bonus| bonus.id == id)
                .expect("the bonus is in the pool")
        };
        let session = RunSession {
            owned_bonuses: [
                "heavy_hands",
                "finisher",
                "arsenal",
                "marked_prey",
                "aftershock",
            ]
            .map(String::from)
            .to_vec(),
            ..default()
        };
        // Arsenal counts the three combat bonuses owned, itself included.
        let per_bonus = data.bonus_effect("arsenal", "per_bonus_in_category");
        assert!((session.keystone(&data, "arsenal") - 3.0 * per_bonus).abs() < 1e-6);
        assert_eq!(
            session.keystone(&data, "fortress"),
            0.0,
            "an unowned keystone adds nothing"
        );

        assert_eq!(
            session.synergy(&data, bonus("hunters_feast")).as_deref(),
            Some("COMBO: Marked Prey")
        );
        assert_eq!(
            session.synergy(&data, bonus("riposte")).as_deref(),
            Some("SYNERGY: Aftershock"),
            "a bonus names the owned ones it lists"
        );
        assert_eq!(
            session.synergy(&data, bonus("fortress")),
            None,
            "a keystone with nothing of its category owned stands alone"
        );
        assert_eq!(
            RunSession {
                owned_bonuses: vec!["fleet_feet".to_owned()],
                ..default()
            }
            .synergy(&data, bonus("tailwind"))
            .as_deref(),
            Some("SYNERGY: 1 agility owned")
        );
    }
}
