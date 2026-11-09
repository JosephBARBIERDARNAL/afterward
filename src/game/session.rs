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

#[derive(Resource, Debug)]
pub(crate) struct RunSession {
    pub(crate) room_id: u16,
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
    pub(crate) world_loaded: bool,
}

impl RunSession {
    pub(crate) fn has_bonus(&self, id: &str) -> bool {
        self.owned_bonuses.iter().any(|owned| owned == id)
    }

    /// An owned bonus's multiplier effect, or 1 without the bonus.
    pub(crate) fn bonus_multiplier(&self, data: &GameData, id: &str, effect: &str) -> f32 {
        if self.has_bonus(id) {
            data.bonus_effect(id, effect)
        } else {
            1.0
        }
    }

    /// Rounds a full magazine holds, with Extended Magazine.
    pub(crate) fn magazine_capacity(&self, data: &GameData) -> u32 {
        let extra = if self.has_bonus("extended_magazine") {
            data.bonus_effect("extended_magazine", "extra_magazine_rounds") as u32
        } else {
            0
        };
        data.gameplay.player.gun.magazine_size + extra
    }
}

impl Default for RunSession {
    fn default() -> Self {
        Self {
            room_id: 0,
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
            world_loaded: false,
        }
    }
}
