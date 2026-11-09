use serde::Deserialize;

use super::ImageSizeConfig;

/// Where the gun points: straight ahead, or diagonally down or up.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Aim {
    #[default]
    Straight,
    Down,
    Up,
}

impl Aim {
    pub const ALL: [Self; 3] = [Self::Straight, Self::Down, Self::Up];

    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::Straight => "straight",
            Self::Down => "down",
            Self::Up => "up",
        }
    }
}

/// One value per aim direction.
#[derive(Debug, Deserialize)]
pub struct PerAim<T> {
    pub straight: T,
    pub down: T,
    pub up: T,
}

impl<T> PerAim<T> {
    pub(crate) fn get(&self, aim: Aim) -> &T {
        match aim {
            Aim::Straight => &self.straight,
            Aim::Down => &self.down,
            Aim::Up => &self.up,
        }
    }
}

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
    pub aim_hold_s: f32,
    pub clips: PlayerAnimationClips,
    pub run_shooting_shot_frame: usize,
    pub shooting: PerAim<AimClips>,
}

/// The shot clips of one aim direction and the aim pose held after each.
#[derive(Debug, Deserialize)]
pub struct AimClips {
    pub shot: Vec<AnimationClip>,
    pub aim: Vec<AnimationClip>,
}

impl PlayerAnimations {
    /// Every clip with the sprite-sheet key gameplay code refers to it by;
    /// shot variant `n` (from 1) of an aim is `player_shoot_{aim}_{n}`, and
    /// its aim pose `player_aim_{aim}_{n}`.
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
            ("player_turn", &clips.turn),
            ("player_turn_run", &clips.turn_run),
            ("player_jump", &clips.jump),
            ("player_ascend", &clips.ascend),
            ("player_descend", &clips.descend),
            ("player_land", &clips.land),
            ("player_reload", &clips.reload),
            ("player_cover", &clips.cover),
            ("player_hit", &clips.hit),
            ("player_death", &clips.death),
            ("player_run_shooting", &clips.run_shooting),
        ]
        .into_iter()
        .map(|(key, clip)| (key.to_string(), clip))
        .chain(Aim::ALL.into_iter().flat_map(|aim| {
            let clips = self.shooting.get(aim);
            let shots = clips.shot.iter().enumerate().map(move |(index, clip)| {
                (format!("player_shoot_{}_{}", aim.name(), index + 1), clip)
            });
            let poses = clips.aim.iter().enumerate().map(move |(index, clip)| {
                (format!("player_aim_{}_{}", aim.name(), index + 1), clip)
            });
            shots.chain(poses)
        }))
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
    pub turn: AnimationClip,
    pub turn_run: AnimationClip,
    pub jump: AnimationClip,
    pub ascend: AnimationClip,
    pub descend: AnimationClip,
    pub land: AnimationClip,
    pub reload: AnimationClip,
    pub cover: AnimationClip,
    pub hit: AnimationClip,
    pub death: AnimationClip,
    pub run_shooting: AnimationClip,
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
