//! Props from `props/props.yaml`: pixel-art sprites a room places at a spot
//! (`props:` of a room file), drawn once at their own size.

use std::{
    borrow::Borrow,
    collections::BTreeMap,
    fmt::{self, Display, Formatter},
    ops::Deref,
    str::FromStr,
};

use bevy::math::{Rect, Vec2};
use serde::{Deserialize, Deserializer, de::Error as DeserializeError};

use super::PropConfig;

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
    /// `solid_px` boxes, set where it stands and mirrored and turned about
    /// its middle with it, each as its corners counter-clockwise from its
    /// lower-left. None for a prop that is scenery.
    pub(crate) fn solid_corners_h(&self, placed: &PropConfig) -> Vec<[Vec2; 4]> {
        let (Some(sprite), Some(size)) = (self.props.get(&placed.prop), self.size_h(&placed.prop))
        else {
            return Vec::new();
        };
        let left = placed.position_h.x_h - size.x / 2.0;
        let top = placed.position_h.y_h + size.y;
        let middle = Vec2::new(placed.position_h.x_h, placed.position_h.y_h + size.y / 2.0);
        sprite
            .solid_px
            .iter()
            .map(|[x, y, width, height]| {
                let min = Vec2::new(
                    left + *x as f32 / self.px_per_h,
                    top - (y + height) as f32 / self.px_per_h,
                );
                let max = min + Vec2::new(*width as f32, *height as f32) / self.px_per_h;
                placed
                    .placement
                    .corners_about(Rect::from_corners(min, max), middle)
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
    use crate::data::{DecorationLayer, GameData, Placement, SpawnPosition};
    use std::path::Path;

    #[test]
    fn a_solid_prop_is_solid_where_its_sprite_is_drawn_and_scenery_is_not() {
        let asset_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/assets");
        let props = GameData::load(&asset_root)
            .expect("game data should load")
            .props;
        let placed = |prop: &str, placement: Placement| PropConfig {
            label: None,
            prop: prop.parse().expect("a valid prop ID"),
            position_h: SpawnPosition { x_h: 5.0, y_h: 1.0 },
            layer: DecorationLayer::Background,
            placement,
        };
        let as_drawn = Placement::default();
        // The lower-left and upper-right corners of each solid box.
        let spans = |placed: &PropConfig| {
            props
                .solid_corners_h(placed)
                .into_iter()
                .map(|corners| {
                    (
                        corners.into_iter().reduce(Vec2::min).unwrap_or_default(),
                        corners.into_iter().reduce(Vec2::max).unwrap_or_default(),
                    )
                })
                .collect::<Vec<_>>()
        };

        // The small crate fills the lower 10 px of its 16 px cell: half an H
        // wide, standing on the spot.
        assert_eq!(
            spans(&placed("sci_fi_small_crate", as_drawn)),
            vec![(Vec2::new(4.75, 1.0), Vec2::new(5.25, 1.0 + 10.0 / 32.0))]
        );
        // Turned a quarter about its middle (5, 1.25), it stands on its side.
        let turned = spans(&placed(
            "sci_fi_small_crate",
            Placement {
                rotation_deg: 90.0,
                ..as_drawn
            },
        ));
        let (low, high) = turned[0];
        assert!(low.abs_diff_eq(Vec2::new(4.9375, 1.0), 1e-4), "{low}");
        assert!(high.abs_diff_eq(Vec2::new(5.25, 1.5), 1e-4), "{high}");
        // The stack is two boxes: the crate on top and the pair under it, the
        // top one off its middle, and mirrored to the other side of it.
        let top_crate = |placement| spans(&placed("sci_fi_crate_stack", placement))[0];
        assert_eq!(spans(&placed("sci_fi_crate_stack", as_drawn)).len(), 2);
        assert_eq!(top_crate(as_drawn).0.x, 4.5 + 9.0 / 32.0);
        assert_eq!(
            top_crate(Placement {
                mirrored: true,
                ..as_drawn
            })
            .0
            .x,
            4.5 + 7.0 / 32.0
        );
        assert!(
            props
                .solid_corners_h(&placed("sci_fi_ceiling_lamp", as_drawn))
                .is_empty()
        );
    }
}
