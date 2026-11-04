use super::*;
use crate::data::PlayerAnimations;

/// What the animation choice needs to know about the frame's movement.
pub(super) struct AnimationState {
    pub(super) moving: bool,
    pub(super) wall_sliding: bool,
    /// Seconds a wall cling lasts before the slide.
    pub(super) latch_s: f32,
    /// Horizontal speed (world units/s) from which the jog cycle replaces the walk.
    pub(super) jog_speed: f32,
    pub(super) dt: f32,
    pub(super) enemies_alive: bool,
    pub(super) health_fraction: f32,
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

    /// Plays `key` from `frame`, even when it already plays.
    fn restart(&mut self, key: &str, looping: bool, frame: usize) {
        self.animation.sheet.clear();
        self.play(key, looping);
        self.animation.frame = frame.min(self.animation.frames.saturating_sub(1));
        if let Some(atlas) = self.sprite.texture_atlas.as_mut() {
            atlas.index = self.animation.frame;
        }
    }

    /// Whether `key` plays and has reached its last frame.
    fn finished(&self, key: &str) -> bool {
        self.animation.sheet == key && self.animation.frame + 1 >= self.animation.frames
    }

    /// Picks the clip for the player's current state. One-shot actions (shots,
    /// hits, landings, turns, wall jumps) play over the movement clips until
    /// they end; dying and Cover override everything.
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
        if player.covering {
            self.play("player_cover", false);
            return;
        }
        if let Some(action) = player.action {
            let (key, looping, frame) = match action {
                PlayerAction::Shot { running: true, .. } => (
                    String::from("player_run_shooting"),
                    true,
                    animations.run_shooting_shot_frame,
                ),
                PlayerAction::Shot { aim, variant, .. } => (
                    format!("player_shoot_{}_{}", aim.name(), variant + 1),
                    false,
                    0,
                ),
                PlayerAction::Hit => (String::from("player_hit"), false, 0),
                PlayerAction::Land => (String::from("player_land"), false, 0),
                PlayerAction::Turn { running: true } => (String::from("player_turn_run"), false, 0),
                PlayerAction::Turn { running: false } => (String::from("player_turn"), false, 0),
                PlayerAction::WallJump => (String::from("player_wall_jump"), false, 0),
            };
            if restart {
                self.restart(&key, looping, frame);
            } else {
                self.play(&key, looping);
            }
            return;
        }
        if player.dash_timer > 0.0 {
            self.play("player_dash", false);
            return;
        }
        if !player.grounded {
            let key = if state.wall_sliding {
                // The latch cling, then the landing against the wall, then the slide.
                if player.wall_contact_s <= state.latch_s {
                    "player_wall_latch"
                } else if self.animation.sheet == "player_wall_latch"
                    || (self.animation.sheet == "player_wall_land"
                        && !self.finished("player_wall_land"))
                {
                    "player_wall_land"
                } else {
                    "player_wall_slide"
                }
            } else if player.velocity.y > 0.0 {
                // Take-off, then the rising loop.
                if self.animation.sheet == "player_ascend" || self.finished("player_jump") {
                    "player_ascend"
                } else {
                    "player_jump"
                }
            } else {
                "player_descend"
            };
            let looping = matches!(
                key,
                "player_wall_slide" | "player_ascend" | "player_descend"
            );
            self.play(key, looping);
            return;
        }

        let speed = player.velocity.x.abs();
        if !state.moving {
            player.idle_s += state.dt;
        } else {
            player.idle_s = 0.0;
        }
        if let Some(reload) = player.reload.filter(|_| !state.moving) {
            if self.animation.sheet != "player_reload" {
                // Fit the clip to the reload, from how far it already got.
                let frames = self
                    .sheets
                    .0
                    .get("player_reload")
                    .map_or(1, |sheet| sheet.frames);
                let progress = 1.0 - reload.remaining_s / reload.duration_s;
                self.restart("player_reload", false, (progress * frames as f32) as usize);
                self.animation.timer = Timer::from_seconds(
                    (reload.duration_s / frames as f32).max(f32::EPSILON),
                    TimerMode::Repeating,
                );
            }
            return;
        }
        if player.aim_hold_timer > 0.0 && !player.crouching {
            let variants = animations.shooting.get(player.aim).aim.len();
            let variant = (player.shot_variant + variants - 1) % variants;
            if !state.moving {
                self.play(
                    &format!("player_aim_{}_{}", player.aim.name(), variant + 1),
                    false,
                );
                return;
            }
            if player.aim == Aim::Straight && speed >= state.jog_speed {
                self.play("player_run_shooting", true);
                return;
            }
        }
        if player.crouching {
            let key = if state.moving {
                "player_crouch_walk"
            } else {
                "player_crouch"
            };
            self.play(key, true);
            return;
        }
        if state.moving {
            let key = if player.sprinting {
                "player_sprint"
            } else if speed >= state.jog_speed {
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
