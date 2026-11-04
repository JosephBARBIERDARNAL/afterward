use super::super::{SpriteSheetSpec, UVec2};
use crate::data::PlayerAnimations;

pub(super) fn sprites(animations: &PlayerAnimations) -> Vec<SpriteSheetSpec> {
    animations
        .keyed()
        .into_iter()
        .map(|(key, clip)| SpriteSheetSpec {
            key,
            path: clip.sheet.clone(),
            frame_width: animations.frame_size_px.width,
            frame_height: animations.frame_size_px.height,
            frames: clip.frames,
            fps: clip.fps,
            first_cell: UVec2::new(clip.first_frame, clip.row),
        })
        .collect()
}
