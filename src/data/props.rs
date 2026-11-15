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

    /// Whether bodies stand on `placed` and bump into it: as its room entry
    /// says, else as its sprite is.
    pub(crate) fn is_solid(&self, placed: &PropConfig) -> bool {
        placed.solid.unwrap_or_else(|| {
            self.props
                .get(&placed.prop)
                .is_some_and(PropSprite::solid_by_default)
        })
    }

    /// The parts of `placed` bodies stand on and bump into, in H: its sprite's
    /// `solid_px` boxes (its whole frame, for a scenery sprite made solid),
    /// set where it stands and mirrored and turned about its middle with it,
    /// each as its corners counter-clockwise from its lower-left. None for a
    /// prop that is scenery.
    pub(crate) fn solid_corners_h(&self, placed: &PropConfig) -> Vec<[Vec2; 4]> {
        let (Some(sprite), Some(size)) = (self.props.get(&placed.prop), self.size_h(&placed.prop))
        else {
            return Vec::new();
        };
        let whole_frame = [[0, 0, sprite.size_px[0], sprite.size_px[1]]];
        let boxes: &[[u32; 4]] = match placed.solid {
            Some(false) => &[],
            Some(true) if sprite.solid_px.is_empty() => &whole_frame,
            _ => &sprite.solid_px,
        };
        let left = placed.position_h.x_h - size.x / 2.0;
        let top = placed.position_h.y_h + size.y;
        let middle = Vec2::new(placed.position_h.x_h, placed.position_h.y_h + size.y / 2.0);
        boxes
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

impl PropSprite {
    /// Whether a prop of this sprite is solid unless its room says otherwise.
    pub(crate) fn solid_by_default(&self) -> bool {
        !self.solid_px.is_empty()
    }
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
            solid: None,
            placement,
        };
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
            spans(&placed("sci_fi_small_crate", Placement::default())),
            vec![(Vec2::new(4.75, 1.0), Vec2::new(5.25, 1.0 + 10.0 / 32.0))]
        );
        // Turned a quarter about its middle (5, 1.25), it stands on its side.
        let turned = spans(&placed(
            "sci_fi_small_crate",
            Placement {
                rotation_deg: 90.0,
                ..Placement::default()
            },
        ));
        let (low, high) = turned[0];
        assert!(low.abs_diff_eq(Vec2::new(4.9375, 1.0), 1e-4), "{low}");
        assert!(high.abs_diff_eq(Vec2::new(5.25, 1.5), 1e-4), "{high}");
        // The stack is two boxes: the crate on top and the pair under it, the
        // top one off its middle, and mirrored to the other side of it.
        let top_crate = |placement| spans(&placed("sci_fi_crate_stack", placement))[0];
        assert_eq!(
            spans(&placed("sci_fi_crate_stack", Placement::default())).len(),
            2
        );
        assert_eq!(top_crate(Placement::default()).0.x, 4.5 + 9.0 / 32.0);
        assert_eq!(
            top_crate(Placement {
                mirrored: true,
                ..Placement::default()
            })
            .0
            .x,
            4.5 + 7.0 / 32.0
        );
        assert!(
            props
                .solid_corners_h(&placed("sci_fi_ceiling_lamp", Placement::default()))
                .is_empty()
        );

        // A room may say otherwise: a solid crate made scenery has nothing to
        // stand on, and a scenery lamp made solid is solid over its whole
        // frame.
        let mut crate_made_scenery = placed("sci_fi_small_crate", Placement::default());
        crate_made_scenery.solid = Some(false);
        assert!(!props.is_solid(&crate_made_scenery));
        assert!(props.solid_corners_h(&crate_made_scenery).is_empty());
        let mut lamp_made_solid = placed("sci_fi_ceiling_lamp", Placement::default());
        lamp_made_solid.solid = Some(true);
        let size = props
            .size_h(&lamp_made_solid.prop)
            .expect("the lamp is in the catalog");
        assert_eq!(
            spans(&lamp_made_solid),
            vec![(
                Vec2::new(5.0 - size.x / 2.0, 1.0),
                Vec2::new(5.0 + size.x / 2.0, 1.0 + size.y)
            )]
        );
    }
}
