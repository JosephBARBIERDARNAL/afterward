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
    /// Every enemy is down, so the room's bonus item may be laid out.
    pub(crate) room_cleared: bool,
    pub(crate) door_open: bool,
    pub(crate) room_ward_used: bool,
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

    /// Whether the room being played has anyone to fight. One that does not
    /// has nothing to clear: its bonus is there from the first frame.
    pub(crate) fn has_enemies(&self, data: &GameData) -> bool {
        !self.playtest.no_enemies && !data.rooms[&self.room_id].enemy_spawns.is_empty()
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
            room_cleared: false,
            door_open: false,
            room_ward_used: false,
            idle_healing: 0.0,
            last_stand_used: false,
            world_loaded: false,
        }
    }
}
