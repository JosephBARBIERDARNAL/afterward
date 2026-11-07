//! Surface materials from `surfaces/materials.yaml`: the textures one-way
//! platforms and decoration panels are drawn with, and the footsteps they
//! make. A block can also skip the catalog and be painted a plain color.

use std::{
    borrow::Borrow,
    collections::BTreeMap,
    fmt::{self, Display, Formatter},
    ops::Deref,
    str::FromStr,
};

use serde::{Deserialize, Deserializer, de::Error as DeserializeError};

use super::HexColor;

#[derive(Debug, Deserialize)]
pub struct SurfaceMaterials {
    /// Side of every surface texture, in pixels: each is one seamless tile.
    pub texture_px: u32,
    /// World size of one texture tile, in H.
    pub texture_tile_h: f32,
    pub edges: EdgeStyle,
    pub decoration_layers: DecorationLayerStyles,
    /// The footstep a plain-color block makes, since it names no material.
    pub color_footstep: FootstepSound,
    pub materials: BTreeMap<MaterialId, SurfaceMaterial>,
}

impl SurfaceMaterials {
    /// One texture pixel, in H: the unit of edge thicknesses.
    pub(crate) fn texel_h(&self) -> f32 {
        self.texture_tile_h / self.texture_px as f32
    }

    /// What walking on a block painted with `fill` sounds like.
    pub(crate) fn footstep(&self, fill: &Fill) -> FootstepSound {
        match fill {
            Fill::Material(material) => self
                .materials
                .get(material)
                .map_or(self.color_footstep, |material| material.footstep),
            Fill::Color(_) => self.color_footstep,
        }
    }
}

/// The outline and highlight drawn along every block edge that is not
/// covered by another block.
#[derive(Debug, Deserialize)]
pub struct EdgeStyle {
    pub outline_color: HexColor,
    pub outline_alpha: f32,
    pub outline_px: f32,
    pub highlight_color: HexColor,
    pub highlight_alpha: f32,
    pub highlight_px: f32,
}

#[derive(Debug, Deserialize)]
pub struct DecorationLayerStyles {
    pub background: DecorationLayerStyle,
    pub foreground: DecorationLayerStyle,
}

/// How decoration panels on one layer are tinted.
#[derive(Debug, Deserialize)]
pub struct DecorationLayerStyle {
    pub tint: HexColor,
    pub alpha: f32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SurfaceMaterial {
    /// A seamless `texture_px` square tiled across the block in world space,
    /// so neighboring blocks of one material line up.
    pub texture: String,
    pub footstep: FootstepSound,
}

/// What walking on a surface sounds like.
#[derive(Debug, Clone, Copy, Deserialize, Eq, Ord, PartialEq, PartialOrd)]
#[serde(rename_all = "snake_case")]
pub enum FootstepSound {
    Stone,
    Wood,
}

/// How a platform or a decoration panel is painted: with a material from
/// `surfaces/materials.yaml`, or with a plain color.
#[derive(Debug, Clone, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Fill {
    Material(MaterialId),
    Color(HexColor),
}

/// Names the material, or the color's digits.
impl Display for Fill {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Material(material) => Display::fmt(material, formatter),
            Self::Color(color) => Display::fmt(color, formatter),
        }
    }
}

/// The name of a material in `surfaces/materials.yaml`, as rooms refer to it.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MaterialId(String);

impl FromStr for MaterialId {
    type Err = &'static str;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let mut bytes = value.bytes();
        if !bytes.next().is_some_and(|first| first.is_ascii_lowercase())
            || !bytes.all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
        {
            return Err("material IDs must use lowercase letters, digits, and underscores");
        }
        Ok(Self(value.to_string()))
    }
}

impl<'de> Deserialize<'de> for MaterialId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer)?
            .parse()
            .map_err(D::Error::custom)
    }
}

impl Borrow<str> for MaterialId {
    fn borrow(&self) -> &str {
        &self.0
    }
}

impl Deref for MaterialId {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for MaterialId {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}
