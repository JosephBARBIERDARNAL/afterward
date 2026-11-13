//! Background profiles from `background/background_profiles.yaml`.

use std::collections::BTreeMap;

use bevy::image::ImageSampler;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct BackgroundProfiles {
    pub profiles: BTreeMap<String, BackgroundProfile>,
}

#[derive(Debug, Deserialize)]
pub struct BackgroundProfile {
    pub backdrop: String,
    pub source_size_px: ImageSizeConfig,
    pub tile_h: f32,
    #[serde(default)]
    pub repeat_horizontal: bool,
    pub filtering: TextureFiltering,
    pub layers: Vec<BackgroundLayer>,
}

impl BackgroundProfile {
    /// How wide one tile is drawn, keeping the source image's proportions.
    pub(crate) fn tile_width_h(&self) -> f32 {
        self.tile_h * self.source_size_px.width as f32 / self.source_size_px.height as f32
    }
}

/// How a texture is sampled when drawn larger than its source pixels.
/// `nearest` keeps upscaled pixel art crisp on large displays; `linear`
/// smooths it, which blurs low-resolution art.
#[derive(Debug, Clone, Copy, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TextureFiltering {
    Nearest,
    Linear,
}

impl From<TextureFiltering> for ImageSampler {
    fn from(filtering: TextureFiltering) -> Self {
        match filtering {
            TextureFiltering::Nearest => ImageSampler::nearest(),
            TextureFiltering::Linear => ImageSampler::linear(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ImageSizeConfig {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Deserialize)]
pub struct BackgroundLayer {
    pub asset: String,
    pub parallax: f32,
}
