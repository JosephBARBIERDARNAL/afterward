use super::*;
use crate::data::PlayerAnimations;

/// What the animation choice needs to know about the frame's movement.
pub(super) struct AnimationState {
    pub(super) moving: bool,
    /// Horizontal speed (world units/s) from which the jog cycle replaces the walk.
    pub(super) jog_speed: f32,
    pub(super) dt: f32,
    pub(super) enemies_alive: bool,
    pub(super) health_fraction: f32,
    /// How many times its clip's pace the ladder is climbed at, as fast as
    /// the bonuses speed the player's run.
    pub(super) climb_pace: f32,
    /// How long a climb over a ledge takes, which its clip is fitted to.
    pub(super) ledge_climb_s: f32,
}

/// The player's sprite and the sheets its clips come from.
pub(super) struct PlayerSprite<'a> {
    pub(super) sprite: &'a mut Sprite,
    pub(super) animation: &'a mut AnimatedSprite,
    pub(super) sheets: &'a SpriteSheets,
}

impl PlayerSprite<'_> {
    /// Switches to `key`, keeping its progress when it already plays.
    pub(super) fn play(&mut self, key: &str, looping: bool) {
        set_animation(self.sprite, self.animation, self.sheets, key, looping);
    }

    /// Plays `key` from its first frame, even when it already plays.
    fn restart(&mut self, key: &str, looping: bool) {
        self.animation.sheet.clear();
        self.play(key, looping);
    }

    /// Plays the clip `key`, already on, at `speed` times its own pace.
    fn pace(&mut self, key: &str, speed: f32) {
        if let Some(sheet) = self.sheets.0.get(key) {
            self.animation.timer = Timer::from_seconds(
                (1.0 / (sheet.fps * speed)).max(f32::EPSILON),
                TimerMode::Repeating,
            );
        }
    }

    /// Whether `key` plays and has reached its last frame.
    fn finished(&self, key: &str) -> bool {
        self.animation.sheet == key && self.animation.frame + 1 >= self.animation.frames
    }

    /// Picks the clip for the player's current state. One-shot actions (blows
    /// of the melee chain, light and heavy, the slam, the dash, hits,
    /// landings, wall pushes) play over the movement clips until they end;
    /// dying overrides everything.
    pub(super) fn update(
        &mut self,
        player: &mut PlayerBody,
        animations: &PlayerAnimations,
        state: &AnimationState,
    ) {
        let restart = player.action_started;
        player.action_started = false;
        if player.dying_s.is_some() {
            self.play("player_death", false);
            return;
        }
        if player.ledge_climb.is_some() {
            if self.animation.sheet != "player_ledge_climb" {
                self.restart("player_ledge_climb", false);
                self.pace(
                    "player_ledge_climb",
                    animations.clips.ledge_climb.duration_s() / state.ledge_climb_s,
                );
            }
            return;
        }
        if let Some(action) = player.action {
            let key = match action {
                PlayerAction::Strike { strike, .. } => strike.clip_key(),
                PlayerAction::Slam => "player_ground_slam",
                PlayerAction::Dash { .. } => "player_dash",
                PlayerAction::Rise => "player_rise",
                PlayerAction::Aura(aura) => aura.clip_key(),
                PlayerAction::Healing => "player_healing",
                PlayerAction::Hit => "player_hit",
                PlayerAction::Land => "player_land",
                PlayerAction::WallJump => "player_wall_jump",
            };
            if restart {
                self.restart(key, false);
                // A chain sped up by a bonus plays its blow that much faster,
                // and the dash fits the roll to its own length.
                let speed = match action {
                    PlayerAction::Strike { speed, .. } | PlayerAction::Dash { speed } => speed,
                    _ => 1.0,
                };
                self.pace(key, speed);
            } else {
                self.play(key, false);
            }
            return;
        }
        if player.wall_cling.is_some() {
            self.play("player_wall_slide", true);
            return;
        }
        if player.climbing.is_some() {
            // Hanging still on the ladder holds the frame the climb is on.
            if self.animation.sheet != "player_climb" {
                self.play("player_climb", true);
                self.pace("player_climb", state.climb_pace);
            }
            if player.velocity.y == 0.0 {
                self.animation.timer.reset();
            }
            return;
        }
        if !player.grounded {
            let key = if player.velocity.y > 0.0 {
                // Take-off, then the rising loop.
                if self.animation.sheet == "player_ascend" || self.finished("player_jump") {
                    "player_ascend"
                } else {
                    "player_jump"
                }
            } else {
                "player_descend"
            };
            let looping = matches!(key, "player_ascend" | "player_descend");
            self.play(key, looping);
            return;
        }

        let speed = player.velocity.x.abs();
        if !state.moving {
            player.idle_s += state.dt;
        } else {
            player.idle_s = 0.0;
        }
        if player.crouching {
            let key = if state.moving {
                "player_crouch_walk"
            } else {
                "player_crouch"
            };
            self.play(key, false);
            return;
        }
        if state.moving {
            let key = if speed >= state.jog_speed {
                "player_jog"
            } else {
                "player_walk"
            };
            self.play(key, true);
            return;
        }

        // Standing still: wounded, then an occasional hand fidget, then the
        // alert or calm idle.
        if state.health_fraction < animations.wounded_health_fraction {
            self.play("player_idle_wounded", true);
            return;
        }
        let fidget = if player.fidget_right {
            "player_idle_fidget_right"
        } else {
            "player_idle_fidget_left"
        };
        if self.finished(fidget) {
            player.idle_s = 0.0;
            player.fidget_right = !player.fidget_right;
        } else if player.idle_s >= animations.idle_fidget_after_s {
            self.play(fidget, false);
            return;
        }
        let key = if state.enemies_alive {
            "player_idle_alert"
        } else {
            "player_idle"
        };
        self.play(key, true);
    }
}
