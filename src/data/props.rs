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
