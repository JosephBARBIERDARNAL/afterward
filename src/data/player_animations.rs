use serde::Deserialize;

use super::ImageSizeConfig;

/// Player sprite sheets and how their frames map onto the collision box.
#[derive(Debug, Deserialize)]
pub struct PlayerAnimations {
    pub frame_size_px: ImageSizeConfig,
    pub body_height_px: f32,
    pub feet_row_px: f32,
    pub body_height_h: f32,
    pub jog_speed_fraction: f32,
    pub wounded_health_fraction: f32,
    pub idle_fidget_after_s: f32,
    pub clips: PlayerAnimationClips,
}

impl PlayerAnimations {
    /// Every clip with the sprite-sheet key gameplay code refers to it by.
    pub(crate) fn keyed(&self) -> Vec<(String, &AnimationClip)> {
        let clips = &self.clips;
        [
            ("player_idle", &clips.idle),
            ("player_idle_alert", &clips.idle_alert),
            ("player_idle_wounded", &clips.idle_wounded),
            ("player_idle_fidget_left", &clips.idle_fidget_left),
            ("player_idle_fidget_right", &clips.idle_fidget_right),
            ("player_walk", &clips.walk),
            ("player_jog", &clips.jog),
            ("player_crouch", &clips.crouch),
            ("player_crouch_walk", &clips.crouch_walk),
            ("player_jump", &clips.jump),
            ("player_ascend", &clips.ascend),
            ("player_descend", &clips.descend),
            ("player_land", &clips.land),
            ("player_wall_slide", &clips.wall_slide),
            ("player_wall_jump", &clips.wall_jump),
            ("player_dive", &clips.dive),
            ("player_swim", &clips.swim),
            ("player_cover", &clips.cover),
            ("player_hit", &clips.hit),
            ("player_death", &clips.death),
            ("player_strike_1", &clips.strike_1),
            ("player_strike_2", &clips.strike_2),
            ("player_kick", &clips.kick),
            ("player_fire_kick", &clips.fire_kick),
        ]
        .into_iter()
        .map(|(key, clip)| (key.to_string(), clip))
        .collect()
    }
}

#[derive(Debug, Deserialize)]
pub struct PlayerAnimationClips {
    pub idle: AnimationClip,
    pub idle_alert: AnimationClip,
    pub idle_wounded: AnimationClip,
    pub idle_fidget_left: AnimationClip,
    pub idle_fidget_right: AnimationClip,
    pub walk: AnimationClip,
    pub jog: AnimationClip,
    pub crouch: AnimationClip,
    pub crouch_walk: AnimationClip,
    pub jump: AnimationClip,
    pub ascend: AnimationClip,
    pub descend: AnimationClip,
    pub land: AnimationClip,
    pub wall_slide: AnimationClip,
    pub wall_jump: AnimationClip,
    pub dive: AnimationClip,
    pub swim: AnimationClip,
    pub cover: AnimationClip,
    pub hit: AnimationClip,
    pub death: AnimationClip,
    pub strike_1: AnimationClip,
    pub strike_2: AnimationClip,
    pub kick: AnimationClip,
    pub fire_kick: AnimationClip,
}

/// Which blow a step of the melee combo lands, and so which clip it plays.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
pub enum MeleeStrike {
    #[serde(rename = "strike_1")]
    Strike1,
    #[serde(rename = "strike_2")]
    Strike2,
    #[serde(rename = "kick")]
    Kick,
    #[serde(rename = "fire_kick")]
    FireKick,
}

impl MeleeStrike {
    /// The sprite-sheet key the strike's clip is loaded under.
    pub(crate) fn clip_key(self) -> &'static str {
        match self {
            Self::Strike1 => "player_strike_1",
            Self::Strike2 => "player_strike_2",
            Self::Kick => "player_kick",
            Self::FireKick => "player_fire_kick",
        }
    }

    pub(crate) fn clip(self, clips: &PlayerAnimationClips) -> &AnimationClip {
        match self {
            Self::Strike1 => &clips.strike_1,
            Self::Strike2 => &clips.strike_2,
            Self::Kick => &clips.kick,
            Self::FireKick => &clips.fire_kick,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct AnimationClip {
    pub sheet: String,
    pub frames: usize,
    pub fps: f32,
    /// Grid row of the sheet the clip reads.
    #[serde(default)]
    pub row: u32,
    /// Grid column of the clip's first frame.
    #[serde(default)]
    pub first_frame: u32,
}

impl AnimationClip {
    pub(crate) fn duration_s(&self) -> f32 {
        self.frames as f32 / self.fps
    }
}
