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

/// A bonus at the rarity it was drawn at: one offered on the bonus screen,
/// or one the run owns.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) struct BonusPick {
    pub(crate) id: String,
    pub(crate) rarity: BonusRarity,
}

impl BonusPick {
    pub(crate) fn new(id: &str, rarity: BonusRarity) -> Self {
        Self {
            id: id.to_owned(),
            rarity,
        }
    }
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
    pub(crate) owned_bonuses: Vec<BonusPick>,
    pub(crate) offered_bonuses: Vec<BonusPick>,
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
        self.owned_bonuses.iter().any(|owned| owned.id == id)
    }

    /// The owned bonuses, each with the rarity it was taken at.
    pub(crate) fn owned<'a>(
        &'a self,
        data: &'a GameData,
    ) -> impl Iterator<Item = (&'a BonusConfig, BonusRarity)> + 'a {
        self.owned_bonuses
            .iter()
            .filter_map(|owned| Some((data.bonuses.bonus(&owned.id)?, owned.rarity)))
    }

    /// An effect of an owned bonus, at the rarity it was taken at, or 0
    /// without the bonus.
    pub(crate) fn bonus_effect(&self, data: &GameData, id: &str, effect: &str) -> f32 {
        self.owned(data)
            .find(|(bonus, _)| bonus.id == id)
            .and_then(|(bonus, rarity)| bonus.effect(rarity, effect))
            .unwrap_or_default()
    }

    /// An owned bonus's multiplier effect, or 1 without the bonus.
    pub(crate) fn bonus_multiplier(&self, data: &GameData, id: &str, effect: &str) -> f32 {
        if self.has_bonus(id) {
            self.bonus_effect(data, id, effect)
        } else {
            1.0
        }
    }

    /// Whether a bonus may still be offered: not one already owned, and not
    /// one that builds on bonuses the run has yet to take.
    pub(crate) fn may_be_offered(&self, bonus: &BonusConfig) -> bool {
        !self.has_bonus(&bonus.id) && bonus.requires.iter().all(|id| self.has_bonus(id))
    }

    /// Whether the pool can still offer a choice of two. A run is longer
    /// than the bonuses are many: once it cannot, a cleared room lays no
    /// bonus and its door simply opens.
    pub(crate) fn has_offers(&self, data: &GameData) -> bool {
        data.bonuses
            .bonuses
            .iter()
            .filter(|bonus| self.may_be_offered(bonus))
            .count()
            >= 2
    }

    /// Takes a bonus at the rarity it was drawn at, and pays out what it
    /// gives once, the moment it is taken.
    pub(crate) fn take(&mut self, data: &GameData, pick: BonusPick) {
        self.owned_bonuses.push(pick.clone());
        let Some(bonus) = data.bonuses.bonus(&pick.id) else {
            return;
        };
        let effect = |key| bonus.effect(pick.rarity, key).unwrap_or_default();
        self.max_health += effect("max_health_increase");
        self.rerolls = self.rerolls.saturating_add(effect("extra_rerolls") as u8);
        self.health = (self.health + effect("restore_health")).min(self.max_health);
    }

    /// Whether the room being played has anyone to fight. One that does not
    /// has nothing to clear: its bonus is there from the first frame.
    pub(crate) fn has_enemies(&self, data: &GameData) -> bool {
        !self.playtest.no_enemies && !data.rooms[&self.room_id].enemy_spawns.is_empty()
    }

    /// Whether the room's fight has been won, which powers its devices down
    /// and puts out what was still flying or burning. A room with no one to
    /// fight is cleared from its first frame, and keeps its devices running:
    /// they are what it is crossed against.
    pub(crate) fn fight_won(&self, data: &GameData) -> bool {
        self.room_cleared && self.has_enemies(data)
    }

    /// How many owned bonuses a keystone counts: every one of them (itself
    /// included) for one with `per_bonus_owned`, those that build on others
    /// for one with `per_combo`, and none for any other bonus.
    fn counted_by(&self, data: &GameData, keystone: &BonusConfig) -> usize {
        if keystone.has_effect("per_bonus_owned") {
            return self.owned_bonuses.len();
        }
        if !keystone.has_effect("per_combo") {
            return 0;
        }
        self.owned(data)
            .filter(|(owned, _)| !owned.requires.is_empty())
            .count()
    }

    /// What an owned keystone adds: its `per_bonus_owned` or its `per_combo`
    /// for every owned bonus it counts, or 0 without it.
    pub(crate) fn keystone(&self, data: &GameData, id: &str) -> f32 {
        data.bonuses
            .bonus(id)
            .filter(|_| self.has_bonus(id))
            .map_or(0.0, |keystone| {
                (self.bonus_effect(data, id, "per_bonus_owned")
                    + self.bonus_effect(data, id, "per_combo"))
                    * self.counted_by(data, keystone) as f32
            })
    }

    /// What a bonus works with in the build already owned, in words, or
    /// nothing when it stands alone: the bonuses it builds on, the owned ones
    /// it lists (or that list it) as `synergies`, or the combos Harmony
    /// counts. Such a bonus weighs heavier in the draw, and the bonus screen
    /// says why. A keystone counting every bonus owned works with any build,
    /// so it is not singled out.
    pub(crate) fn synergy(&self, data: &GameData, bonus: &BonusConfig) -> Option<String> {
        let owned = self.owned(data).map(|(owned, _)| owned);
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
        if !bonus.has_effect("per_combo") {
            return None;
        }
        match self.counted_by(data, bonus) {
            0 => None,
            count => Some(format!("SYNERGY: {count} combos owned")),
        }
    }

    /// Pays out what a playtest (or `--bonuses`) asked the run to open with,
    /// each at one of its rarities drawn at random and as if chosen, and
    /// opens it whole.
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
            let rarities = bonus.rarities.keys().copied().collect::<Vec<_>>();
            let rarity = rarities[random.index(rarities.len())];
            self.take(data, BonusPick::new(&bonus.id, rarity));
        }
        self.health = self.max_health;
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
            offered_bonuses: Vec::new(),
            previous_offers: BTreeSet::new(),
            selected_choice: bonus_screen::BonusChoice::Offer(0),
            bonus_reroll_animation_remaining_s: 0.0,
            bonus_chosen_hold_remaining_s: 0.0,
            wave: 0,
            wave_wait_s: 0.0,
            untouched: true,
            flawless: false,
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
        let bonus = |id: &str| data.bonuses.bonus(id).expect("the bonus is in the pool");
        let session = RunSession {
            owned_bonuses: vec![
                BonusPick::new("heavy_hands", BonusRarity::Common),
                BonusPick::new("finisher", BonusRarity::Uncommon),
                BonusPick::new("arsenal", BonusRarity::Legendary),
                BonusPick::new("marked_prey", BonusRarity::Uncommon),
                BonusPick::new("aftershock", BonusRarity::Rare),
            ],
            ..default()
        };
        // Arsenal counts the five bonuses owned, itself included, at the
        // rarity it was taken at.
        let per_bonus = bonus("arsenal")
            .effect(BonusRarity::Legendary, "per_bonus_owned")
            .expect("a legendary Arsenal");
        assert!((session.keystone(&data, "arsenal") - 5.0 * per_bonus).abs() < 1e-6);
        assert_eq!(
            session.keystone(&data, "fortress"),
            0.0,
            "an unowned keystone adds nothing"
        );
        assert_eq!(
            session.bonus_effect(&data, "aftershock", "dash_shock_damage"),
            bonus("aftershock")
                .effect(BonusRarity::Rare, "dash_shock_damage")
                .expect("a rare Aftershock"),
            "an owned bonus works at the rarity it was taken at"
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
            "a keystone counting every bonus works with any build"
        );
    }
}
