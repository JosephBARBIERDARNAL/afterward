//! Background profiles from `background/background_profiles.yaml`.

use std::{
    collections::BTreeMap,
    fmt::{self, Display, Formatter},
    str::FromStr,
};

use bevy::image::ImageSampler;
use serde::{Deserialize, Deserializer, de::Error as DeserializeError};

use super::HexColor;

/// One entry of a room's `background_profiles`: a profile of
/// `background_profiles.yaml` by name, or a plain sky of one color, written
/// `"#203040"` (quoted, or YAML reads it as a comment).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Backdrop {
    Profile(String),
    Color(HexColor),
}

impl FromStr for Backdrop {
    type Err = &'static str;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let value = value.trim();
        if value.starts_with('#') {
            return value.parse().map(Self::Color);
        }
        if value.is_empty() {
            return Err("a background is a profile name or a #RRGGBB color");
        }
        Ok(Self::Profile(value.to_string()))
    }
}

impl<'de> Deserialize<'de> for Backdrop {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer)?
            .parse()
            .map_err(D::Error::custom)
    }
}

/// Names the profile, or the color's digits.
impl Display for Backdrop {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Profile(profile) => formatter.write_str(profile),
            Self::Color(color) => Display::fmt(color, formatter),
        }
    }
}

/// One layer of a profile a room draws, as its `hidden_background_layers`
/// names it: `city_pink_dusk/sky` for the profile's sky, `city_pink_dusk/2`
/// for the second of its layers counted from the furthest.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct BackgroundLayerRef {
    pub profile: String,
    pub layer: ProfileLayer,
}

/// A profile's sky, or one of its `layers` by index (0 the furthest).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ProfileLayer {
    Sky,
    Scenery(usize),
}

impl Display for ProfileLayer {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Sky => formatter.write_str("sky"),
            Self::Scenery(index) => write!(formatter, "{}", index + 1),
        }
    }
}

impl FromStr for BackgroundLayerRef {
    type Err = &'static str;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        const FORMAT: &str = "a background layer is <profile>/sky or <profile>/<layer number>";
        let (profile, layer) = value.trim().split_once('/').ok_or(FORMAT)?;
        let layer = match layer.trim() {
            "sky" => ProfileLayer::Sky,
            number => match number.parse::<usize>() {
                Ok(number) if number > 0 => ProfileLayer::Scenery(number - 1),
                _ => return Err(FORMAT),
            },
        };
        let profile = profile.trim();
        if profile.is_empty() {
            return Err(FORMAT);
        }
        Ok(Self {
            profile: profile.to_string(),
            layer,
        })
    }
}

impl<'de> Deserialize<'de> for BackgroundLayerRef {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer)?
            .parse()
            .map_err(D::Error::custom)
    }
}

impl Display for BackgroundLayerRef {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}/{}", self.profile, self.layer)
    }
}

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
