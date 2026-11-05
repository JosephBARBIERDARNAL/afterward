//! The current run: room, health, and the bonus build.

use super::*;

#[derive(Resource, Debug)]
pub(crate) struct RunSession {
    pub(crate) room_id: u16,
    pub(crate) health: f32,
    pub(crate) max_health: f32,
    pub(crate) rerolls: u8,
    pub(crate) owned_bonuses: Vec<String>,
    pub(crate) offered_bonuses: Vec<String>,
    pub(crate) previous_offers: BTreeSet<String>,
    pub(crate) selected_choice: bonus_screen::BonusChoice,
    pub(crate) bonus_reroll_animation_remaining_s: f32,
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
            room_id: 1,
            health: 100.0,
            max_health: 100.0,
            rerolls: 3,
            owned_bonuses: Vec::new(),
            offered_bonuses: Vec::new(),
            previous_offers: BTreeSet::new(),
            selected_choice: bonus_screen::BonusChoice::Offer(0),
            bonus_reroll_animation_remaining_s: 0.0,
            door_open: false,
            room_ward_used: false,
            world_loaded: false,
        }
    }
}
