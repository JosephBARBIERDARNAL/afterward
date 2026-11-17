//! The player body: movement, the melee chain, and bonus-timer state.

use super::*;
use crate::data::MeleeStrike;

#[derive(Component)]
pub(crate) struct PlayerBody {
    pub(crate) velocity: Vec2,
    pub(crate) facing: f32,
    pub(crate) grounded: bool,
    pub(crate) jumps_used: u8,
    /// Rising from a jump whose button is still held; releasing it cuts the rise.
    pub(crate) jump_held: bool,
    /// What is left of a Jump pressed before it could be obeyed.
    pub(crate) jump_buffer_s: f32,
    /// What is left of an Attack pressed before it could be thrown.
    pub(crate) attack_buffer_s: f32,
    /// What is left of a Heavy Attack pressed before it could be thrown.
    pub(crate) heavy_buffer_s: f32,
    /// The blow being thrown, if any.
    pub(crate) strike: Option<Strike>,
    /// Which step of the melee chain the next press of Attack throws, and so
    /// how many of its blows came before: what Heavy Attack ends it on.
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
    /// A dash came to its end this frame: `combat::player_melee` takes it,
    /// for the shock Aftershock leaves there.
    pub(crate) dash_ended: bool,
    /// How far into the dash the player is, while one is under way.
    pub(crate) dash_s: Option<f32>,
    /// The way the dash under way carries the player (-1 for left).
    pub(crate) dash_direction: f32,
    /// What is left of the wait before the next dash.
    pub(crate) dash_cooldown_s: f32,
    /// Dashes taken in mid-air since the player last stood, clung or climbed.
    pub(crate) air_dashes_used: u8,
    /// Where a dash just began, until `combat::plant_decoy` has taken it:
    /// the spot Decoy's double is left standing on.
    pub(crate) dash_origin: Option<Vec3>,
    pub(crate) crouching: bool,
    /// Clinging to a platform's side in mid-air, and which way that wall lies
    /// (-1 for a wall on the player's left).
    pub(crate) wall_cling: Option<f32>,
    /// Running up the wall clung to, under Wall Runner.
    pub(crate) wall_running: bool,
    /// How much higher Wall Runner may still carry the player up a wall
    /// before they next stand, in world units.
    pub(crate) wall_run_left: f32,
    /// The ladder the player is climbing: its column, in world units.
    pub(crate) climbing: Option<Rect>,
    /// Climbing over the edge of a platform, while it lasts.
    pub(crate) ledge_climb: Option<LedgeClimb>,
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
    /// What is left of the player's hiding from the enemies
    /// (`combat/stealth.rs`): none of them can see the player meanwhile.
    pub(crate) invisible_s: f32,
    /// How long each bonus that hides the player still waits before it can
    /// again.
    pub(crate) vanish_cooldowns: BTreeMap<&'static str, f32>,
    /// The bonus that has made the next blow to connect heavier (Riposte or
    /// Retaliation), and what that blow is multiplied by.
    pub(crate) empowered: Option<(&'static str, f32)>,
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
            jump_buffer_s: 0.0,
            attack_buffer_s: 0.0,
            heavy_buffer_s: 0.0,
            strike: None,
            combo_step: 0,
            combo_window_s: 0.0,
            lifesteal_healing: 0.0,
            slamming: false,
            slam_landed: false,
            dash_ended: false,
            dash_s: None,
            dash_direction: 1.0,
            dash_cooldown_s: 0.0,
            air_dashes_used: 0,
            dash_origin: None,
            crouching: false,
            wall_cling: None,
            wall_running: false,
            wall_run_left: 0.0,
            climbing: None,
            ledge_climb: None,
            wall_control_lock_s: 0.0,
            idle_s: 0.0,
            fidget_right: false,
            dying_s: None,
            invulnerability_timer: 0.0,
            rhythm_timer: 0.0,
            rhythm_stacks: 0,
            chain_reaction_timer: 0.0,
            sated_fury_timer: 0.0,
            invisible_s: 0.0,
            vanish_cooldowns: BTreeMap::new(),
            empowered: None,
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

/// A blow in the air.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Strike {
    pub(crate) thrown: Thrown,
    /// Seconds until it connects; once past zero it already has.
    pub(crate) contact_in_s: f32,
    /// Thrown after a blow of the chain rather than opening one.
    pub(crate) ends_a_chain: bool,
}

/// Which blow is in the air: a step of the chain (`melee.steps`), or a heavy
/// one (`melee.heavy`). The slam is not here, because it lands when it
/// reaches the floor rather than at a time of its own (`slamming`).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Thrown {
    Step(usize),
    Heavy(usize),
}

/// A climb over a platform's edge: the body is carried from where it caught
/// the edge, up beside it and then over onto the top.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct LedgeClimb {
    /// Where the feet were when the edge was caught.
    pub(crate) from: Vec2,
    /// Where the feet stand once on top.
    pub(crate) to: Vec2,
    pub(crate) elapsed_s: f32,
}

/// A one-shot player animation played over the movement clips.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum PlayerAction {
    /// A blow of the melee chain, light or heavy, played at `speed` times its
    /// clip's rate.
    Strike {
        strike: MeleeStrike,
        speed: f32,
    },
    /// Driving at the floor, and the floor breaking.
    Slam,
    /// The dash, played at `speed` times the roll clip's rate.
    Dash {
        speed: f32,
    },
    /// Getting up off the floor again, which only Last Stand asks for.
    Rise,
    /// The glow worn for a moment when a bonus is taken.
    Aura,
    /// Healing: a health drop off the floor, or a bonus paying out on a
    /// cleared room.
    Healing,
    Hit,
    Land,
    /// Pushing off a wall.
    WallJump,
}
