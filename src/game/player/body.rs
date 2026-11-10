//! The player body: movement, the melee chain, and bonus-timer state.

use super::*;
use crate::data::{AnimationClip, MeleeStrike};
use std::str::FromStr;

#[derive(Component)]
pub(crate) struct PlayerBody {
    pub(crate) velocity: Vec2,
    pub(crate) facing: f32,
    pub(crate) grounded: bool,
    pub(crate) jumps_used: u8,
    /// Rising from a jump whose button is still held; releasing it cuts the rise.
    pub(crate) jump_held: bool,
    /// The blow being thrown, if any.
    pub(crate) strike: Option<Strike>,
    /// Which step of the melee chain the next press throws.
    pub(crate) combo_step: usize,
    /// What is left of the window to ask for the chain's next blow; at zero
    /// the chain drops back to its first step.
    pub(crate) combo_window_s: f32,
    /// Health Lifesteal has restored in this room.
    pub(crate) lifesteal_healing: f32,
    /// Driving at the floor after Attack with Down held in mid-air.
    pub(crate) slamming: bool,
    /// The slam reached the floor this frame and has yet to break it.
    pub(crate) slam_landed: bool,
    /// How long Attack has been held since the blow in hand ended; past
    /// `melee.charge.ready_s` the charged blow is ready to throw.
    pub(crate) charge_s: f32,
    /// How far into the roll the player is, while one is under way.
    pub(crate) roll_s: Option<f32>,
    /// Where a roll just began, until `combat::plant_decoy` has taken it:
    /// the spot Decoy's double is left standing on.
    pub(crate) roll_origin: Option<Vec3>,
    pub(crate) crouching: bool,
    pub(crate) covering: bool,
    /// Clinging to a platform's side in mid-air, and which way that wall lies
    /// (-1 for a wall on the player's left).
    pub(crate) wall_cling: Option<f32>,
    /// Seconds left of the lock a wall jump puts on movement input, so the
    /// push actually carries the player off the wall.
    pub(crate) wall_control_lock_s: f32,
    /// Seconds spent standing idle, for the hand fidgets.
    pub(crate) idle_s: f32,
    /// The next fidget uses the right hand.
    pub(crate) fidget_right: bool,
    /// Seconds since the player died, while the death animation plays.
    pub(crate) dying_s: Option<f32>,
    pub(crate) invulnerability_timer: f32,
    pub(crate) rhythm_timer: f32,
    pub(crate) rhythm_stacks: u8,
    pub(crate) chain_reaction_timer: f32,
    pub(crate) sated_fury_timer: f32,
    /// One-shot animation playing over the movement clips.
    pub(crate) action: Option<PlayerAction>,
    /// Seconds left on `action`; it only grows when an action (re)starts.
    pub(crate) action_animation_timer: f32,
    /// `action` (re)started since the animation last showed it.
    pub(crate) action_started: bool,
    pub(crate) footstep_timer: f32,
}

impl Default for PlayerBody {
    fn default() -> Self {
        Self {
            velocity: Vec2::ZERO,
            facing: 1.0,
            grounded: true,
            jumps_used: 0,
            jump_held: false,
            strike: None,
            combo_step: 0,
            combo_window_s: 0.0,
            lifesteal_healing: 0.0,
            slamming: false,
            slam_landed: false,
            charge_s: 0.0,
            roll_s: None,
            roll_origin: None,
            crouching: false,
            covering: false,
            wall_cling: None,
            wall_control_lock_s: 0.0,
            idle_s: 0.0,
            fidget_right: false,
            dying_s: None,
            invulnerability_timer: 0.0,
            rhythm_timer: 0.0,
            rhythm_stacks: 0,
            chain_reaction_timer: 0.0,
            sated_fury_timer: 0.0,
            action: None,
            action_animation_timer: 0.0,
            action_started: false,
            footstep_timer: 0.0,
        }
    }
}

impl PlayerBody {
    /// Starts a one-shot animation lasting `duration_s`.
    pub(crate) fn start_action(&mut self, action: PlayerAction, duration_s: f32) {
        self.action = Some(action);
        self.action_animation_timer = duration_s;
        self.action_started = true;
    }
}

/// The glow a taken bonus leaves on the player, one per bonus category.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BonusAura {
    Combat,
    Defense,
    Agility,
    Intelligence,
}

impl FromStr for BonusAura {
    type Err = &'static str;

    fn from_str(category: &str) -> Result<Self, Self::Err> {
        match category {
            // The combined category wears the combat glow; it has none of
            // its own, and every combined bonus has some bite to it.
            "combat" | "combined" => Ok(Self::Combat),
            "defense" => Ok(Self::Defense),
            "agility" => Ok(Self::Agility),
            "intelligence" => Ok(Self::Intelligence),
            _ => Err("unknown bonus category"),
        }
    }
}

impl BonusAura {
    pub(crate) fn clip(self, clips: &crate::data::PlayerAnimationClips) -> &AnimationClip {
        match self {
            Self::Combat => &clips.aura_combat,
            Self::Defense => &clips.aura_defense,
            Self::Agility => &clips.aura_agility,
            Self::Intelligence => &clips.aura_intelligence,
        }
    }

    pub(crate) fn clip_key(self) -> &'static str {
        match self {
            Self::Combat => "player_aura_combat",
            Self::Defense => "player_aura_defense",
            Self::Agility => "player_aura_agility",
            Self::Intelligence => "player_aura_intelligence",
        }
    }
}

/// A blow in the air.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Strike {
    pub(crate) thrown: Thrown,
    /// Seconds until it connects; once past zero it already has.
    pub(crate) contact_in_s: f32,
}

/// Which blow is in the air: a step of the chain, or the charged one. The
/// slam is not here, because it lands when it reaches the floor rather than
/// at a time of its own (`slamming`).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Thrown {
    Step(usize),
    Charged,
}

/// A one-shot player animation played over the movement clips.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum PlayerAction {
    /// A blow of the melee chain, played at `speed` times its clip's rate.
    Strike {
        strike: MeleeStrike,
        speed: f32,
    },
    /// Gathering for the charged blow, then throwing it.
    Charging,
    ChargedBlow,
    /// Driving at the floor, and the floor breaking.
    Slam,
    /// The roll Down and Jump together ask for.
    Roll,
    /// Getting up off the floor again, which only Last Stand asks for.
    Rise,
    /// The glow worn for a moment when a bonus is taken.
    Aura(BonusAura),
    /// Healing, by where it came from: a flask off the floor, or a bonus
    /// paying out on a cleared room.
    Drink,
    Healing,
    Hit,
    Land,
    /// Pushing off a wall.
    WallJump,
}
