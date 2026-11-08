//! The player body: movement, gun, and bonus-timer state.

use super::*;

#[derive(Component)]
pub(crate) struct PlayerBody {
    pub(crate) velocity: Vec2,
    pub(crate) facing: f32,
    pub(crate) grounded: bool,
    pub(crate) jumps_used: u8,
    /// Rising from a jump whose button is still held; releasing it cuts the rise.
    pub(crate) jump_held: bool,
    pub(crate) dash_timer: f32,
    pub(crate) dash_cooldown: f32,
    pub(crate) dash_direction: f32,
    /// Seconds until the gun can fire again.
    pub(crate) fire_cooldown: f32,
    /// Rounds left in the magazine.
    pub(crate) ammo: u32,
    pub(crate) reload: Option<Reload>,
    /// Shot variant the next shot plays; consecutive shots alternate.
    pub(crate) shot_variant: usize,
    /// Direction of the last shot, held by the aim pose.
    pub(crate) aim: Aim,
    /// Seconds the aim pose keeps holding after a shot.
    pub(crate) aim_hold_timer: f32,
    /// Health Vampiric Rounds restored from the current magazine.
    pub(crate) magazine_healing: f32,
    pub(crate) crouching: bool,
    pub(crate) covering: bool,
    /// Sprinter bonus: running faster since a grounded dash.
    pub(crate) sprinting: bool,
    /// Seconds spent standing idle, for the hand fidgets.
    pub(crate) idle_s: f32,
    /// The next fidget uses the right hand.
    pub(crate) fidget_right: bool,
    /// Seconds since the player died, while the death animation plays.
    pub(crate) dying_s: Option<f32>,
    pub(crate) invulnerability_timer: f32,
    pub(crate) dash_invulnerability_timer: f32,
    pub(crate) dash_shot_timer: f32,
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
            dash_timer: 0.0,
            dash_cooldown: 0.0,
            dash_direction: 1.0,
            fire_cooldown: 0.0,
            ammo: 0,
            reload: None,
            shot_variant: 0,
            aim: Aim::Straight,
            aim_hold_timer: 0.0,
            magazine_healing: 0.0,
            crouching: false,
            covering: false,
            sprinting: false,
            idle_s: 0.0,
            fidget_right: false,
            dying_s: None,
            invulnerability_timer: 0.0,
            dash_invulnerability_timer: 0.0,
            dash_shot_timer: 0.0,
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

/// A one-shot player animation played over the movement clips.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum PlayerAction {
    /// Firing; a straight shot while running plays the run-shooting cycle.
    Shot {
        aim: Aim,
        variant: usize,
        running: bool,
    },
    Hit,
    Land,
    Turn {
        running: bool,
    },
}

/// A magazine refill under way.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Reload {
    pub(crate) remaining_s: f32,
    pub(crate) duration_s: f32,
}
