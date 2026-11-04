use std::collections::BTreeMap;

use serde::Deserialize;

use super::ImageSizeConfig;

/// Player sprite sheets and how their frames map onto the collision box.
#[derive(Debug, Deserialize)]
pub struct PlayerAnimations {
    pub frame_size_px: ImageSizeConfig,
    pub body_height_px: f32,
    pub feet_row_px: f32,
    pub body_height_h: f32,
    pub run_speed_fraction: f32,
    pub clips: PlayerAnimationClips,
    /// One clip per sword-combo hit, in chain order.
    pub sword_combo: Vec<AnimationClip>,
}

impl PlayerAnimations {
    /// Every clip with the sprite-sheet key gameplay code refers to it by;
    /// combo hit `n` (from 1) is `player_sword_combo_{n}`.
    pub(crate) fn keyed(&self) -> Vec<(String, &AnimationClip)> {
        let clips = &self.clips;
        [
            ("player_idle", &clips.idle),
            ("player_walk", &clips.walk),
            ("player_run", &clips.run),
            ("player_dash", &clips.dash),
            ("player_jump", &clips.jump),
            ("player_fall", &clips.fall),
            ("player_fall_loop", &clips.fall_loop),
            ("player_wall_slide", &clips.wall_slide),
            ("player_ledge_climb", &clips.ledge_climb),
        ]
        .into_iter()
        .map(|(key, clip)| (key.to_string(), clip))
        .chain(
            self.sword_combo
                .iter()
                .enumerate()
                .map(|(index, clip)| (format!("player_sword_combo_{}", index + 1), clip)),
        )
        .collect()
    }
}

#[derive(Debug, Deserialize)]
pub struct PlayerAnimationClips {
    pub idle: AnimationClip,
    pub walk: AnimationClip,
    pub run: AnimationClip,
    pub dash: AnimationClip,
    pub jump: AnimationClip,
    pub fall: AnimationClip,
    pub fall_loop: AnimationClip,
    pub wall_slide: AnimationClip,
    pub ledge_climb: AnimationClip,
}

#[derive(Debug, Deserialize)]
pub struct AnimationClip {
    pub sheet: String,
    pub frames: usize,
    pub fps: f32,
}

/// Optional 3D player model rendered to a texture (see `player/model.yaml`).
#[derive(Debug, Deserialize)]
pub struct PlayerModelConfig {
    pub enabled: bool,
    pub scene: String,
    pub render_size_px: u32,
    pub model_height: f32,
    pub view_height: f32,
    pub feet_margin: f32,
    pub facing_yaw_degrees: f32,
    pub blend_s: f32,
    /// glTF animation index per player clip name (`idle`, `run`, ...).
    pub clips: BTreeMap<String, usize>,
}
