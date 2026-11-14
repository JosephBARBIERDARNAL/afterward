//! Props from `props/props.yaml`: pixel-art sprites a room places at a spot
//! (`props:` of a room file), drawn once at their own size.

use std::{
    borrow::Borrow,
    collections::BTreeMap,
    fmt::{self, Display, Formatter},
    ops::Deref,
    str::FromStr,
};

use bevy::math::Vec2;
use serde::{Deserialize, Deserializer, de::Error as DeserializeError};

use super::{PropConfig, RectH};

#[derive(Debug, Deserialize)]
pub struct PropCatalog {
    /// How many of a sprite's pixels make one H: what sets every prop's size.
    pub px_per_h: f32,
    pub props: BTreeMap<PropId, PropSprite>,
}

impl PropCatalog {
    /// The size, in H, `prop` is drawn at; nothing for a prop not in the catalog.
    pub(crate) fn size_h(&self, prop: &PropId) -> Option<Vec2> {
        self.props.get(prop).map(|sprite| {
            Vec2::new(sprite.size_px[0] as f32, sprite.size_px[1] as f32) / self.px_per_h
        })
    }

    /// The parts of `placed` bodies stand on and bump into, in H: its sprite's
    /// `solid_px` boxes, set where it stands. None for a prop that is scenery.
    pub(crate) fn solid_rects_h(&self, placed: &PropConfig) -> Vec<RectH> {
        let (Some(sprite), Some(size)) = (self.props.get(&placed.prop), self.size_h(&placed.prop))
        else {
            return Vec::new();
        };
        let left = placed.position_h.x_h - size.x / 2.0;
        let top = placed.position_h.y_h + size.y;
        sprite
            .solid_px
            .iter()
            .map(|[x, y, width, height]| RectH {
                x: left + *x as f32 / self.px_per_h,
                y: top - (y + height) as f32 / self.px_per_h,
                width: *width as f32 / self.px_per_h,
                height: *height as f32 / self.px_per_h,
            })
            .collect()
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct PropSprite {
    pub sprite: String,
    /// The width and height in pixels of the sprite, or of one of its frames.
    pub size_px: [u32; 2],
    /// Left out for a still prop. An animated one's sprite is a strip of
    /// frames laid left to right.
    #[serde(default)]
    pub animation: Option<PropAnimation>,
    /// The boxes of the sprite that are solid, as platforms are: `[left, top,
    /// width, height]` in pixels from the frame's top-left corner. Left out,
    /// the prop is scenery.
    #[serde(default)]
    pub solid_px: Vec<[u32; 4]>,
}

/// How an animated prop's strip is played: looped at `fps`.
#[derive(Debug, Clone, Copy, Deserialize)]
pub struct PropAnimation {
    pub frames: usize,
    pub fps: f32,
}

/// The name of a prop in `props/props.yaml`, as rooms refer to it.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PropId(String);

impl FromStr for PropId {
    type Err = &'static str;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let mut bytes = value.bytes();
        if !bytes.next().is_some_and(|first| first.is_ascii_lowercase())
            || !bytes.all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
        {
            return Err("prop IDs must use lowercase letters, digits, and underscores");
        }
        Ok(Self(value.to_string()))
    }
}

impl<'de> Deserialize<'de> for PropId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer)?
            .parse()
            .map_err(D::Error::custom)
    }
}

impl Borrow<str> for PropId {
    fn borrow(&self) -> &str {
        &self.0
    }
}

impl Deref for PropId {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl PropId {
    /// The key the prop's sprite is loaded under in the game's sprite sheets.
    pub(crate) fn sheet(&self) -> String {
        format!("prop_{}", self.0)
    }
}

impl Display for PropId {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{DecorationLayer, GameData, SpawnPosition};
    use std::path::Path;

    #[test]
    fn a_solid_prop_is_solid_where_its_sprite_is_drawn_and_scenery_is_not() {
        let asset_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/assets");
        let props = GameData::load(&asset_root)
            .expect("game data should load")
            .props;
        let placed = |prop: &str| PropConfig {
            label: None,
            prop: prop.parse().expect("a valid prop ID"),
            position_h: SpawnPosition { x_h: 5.0, y_h: 1.0 },
            layer: DecorationLayer::Background,
        };

        // The small crate fills the lower 10 px of its 16 px cell: half an H
        // wide, standing on the spot.
        let crate_rects = props.solid_rects_h(&placed("sci_fi_small_crate"));
        assert_eq!(
            crate_rects,
            vec![RectH {
                x: 4.75,
                y: 1.0,
                width: 0.5,
                height: 10.0 / 32.0,
            }]
        );
        // The stack is two boxes: the crate on top and the pair under it.
        assert_eq!(props.solid_rects_h(&placed("sci_fi_crate_stack")).len(), 2);
        assert!(
            props
                .solid_rects_h(&placed("sci_fi_ceiling_lamp"))
                .is_empty()
        );
    }
}
