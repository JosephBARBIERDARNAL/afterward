//! Colorways from `surfaces/colorways.yaml`: the sci-fi art redrawn in other
//! colors. A platform, decoration, prop or device names one in its room entry
//! (`colorway:`, part of its `Placement`), and every image it is drawn with
//! is recolored by it as the game starts.

use std::{
    borrow::Borrow,
    collections::BTreeMap,
    fmt::{self, Display, Formatter},
    ops::Deref,
    str::FromStr,
};

use bevy::color::{Hsla, Srgba};
use serde::{Deserialize, Deserializer, de::Error as DeserializeError};

#[cfg(debug_assertions)]
use super::HexColor;

#[derive(Debug, Deserialize)]
pub struct Colorways {
    pub colorways: BTreeMap<ColorwayId, Colorway>,
}

/// One way of redrawing the art: each color is changed by the first shift
/// whose hues hold it, and kept as it is when none does.
#[derive(Debug, Clone, Deserialize)]
pub struct Colorway {
    /// The color the editor's swatch for it is painted.
    #[cfg(debug_assertions)]
    pub swatch: HexColor,
    pub shifts: Vec<HueShift>,
}

impl Colorway {
    /// `color` redrawn in this colorway, its alpha kept.
    pub(crate) fn recolor(&self, color: Srgba) -> Srgba {
        let hsla = Hsla::from(color);
        let Some(shift) = self.shifts.iter().find(|shift| shift.holds(hsla)) else {
            return color;
        };
        Srgba::from(Hsla {
            hue: (hsla.hue + shift.rotate_deg).rem_euclid(360.0),
            saturation: (hsla.saturation * shift.saturation).clamp(0.0, 1.0),
            lightness: (hsla.lightness * shift.lightness).clamp(0.0, 1.0),
            alpha: hsla.alpha,
        })
    }
}

/// A family of colors (the hues it holds, as saturated as it asks) and how
/// it is changed: its hue turned, its saturation and lightness scaled.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct HueShift {
    /// What the family is ("green trim"): a note for whoever tunes it.
    pub label: Option<String>,
    /// The hues it holds, in degrees, read round the wheel from the first to
    /// the second, so `[330, 40]` holds the reds and oranges.
    pub hue_deg: [f32; 2],
    /// Colors greyer than this are left to the next shift.
    pub min_saturation: f32,
    pub rotate_deg: f32,
    pub saturation: f32,
    pub lightness: f32,
}

impl Default for HueShift {
    /// Every hue, and nothing changed.
    fn default() -> Self {
        Self {
            label: None,
            hue_deg: [0.0, 360.0],
            min_saturation: 0.0,
            rotate_deg: 0.0,
            saturation: 1.0,
            lightness: 1.0,
        }
    }
}

impl HueShift {
    fn holds(&self, color: Hsla) -> bool {
        let [from, to] = self.hue_deg;
        let span = (to - from).rem_euclid(360.0);
        let span = if span == 0.0 && to != from {
            360.0
        } else {
            span
        };
        color.saturation >= self.min_saturation && (color.hue - from).rem_euclid(360.0) <= span
    }

    /// Whether every value is one the shift can use.
    pub(crate) fn is_valid(&self) -> bool {
        self.hue_deg
            .iter()
            .all(|hue| hue.is_finite() && (0.0..=360.0).contains(hue))
            && (0.0..=1.0).contains(&self.min_saturation)
            && self.rotate_deg.is_finite()
            && [self.saturation, self.lightness]
                .iter()
                .all(|scale| scale.is_finite() && *scale >= 0.0)
    }
}

/// The name of a colorway in `surfaces/colorways.yaml`, as rooms refer to it.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ColorwayId(String);

impl FromStr for ColorwayId {
    type Err = &'static str;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let mut bytes = value.bytes();
        if !bytes.next().is_some_and(|first| first.is_ascii_lowercase())
            || !bytes.all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
        {
            return Err("colorway IDs must use lowercase letters, digits, and underscores");
        }
        Ok(Self(value.to_string()))
    }
}

impl<'de> Deserialize<'de> for ColorwayId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer)?
            .parse()
            .map_err(D::Error::custom)
    }
}

impl Borrow<str> for ColorwayId {
    fn borrow(&self) -> &str {
        &self.0
    }
}

impl Deref for ColorwayId {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for ColorwayId {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_colorway_turns_the_hues_its_shifts_hold_and_keeps_the_rest() {
        let colorway = Colorway {
            swatch: "#FF0000".parse().expect("a valid color"),
            shifts: vec![
                HueShift {
                    hue_deg: [90.0, 180.0],
                    min_saturation: 0.2,
                    rotate_deg: -120.0,
                    ..HueShift::default()
                },
                HueShift {
                    // Round the wheel through 0.
                    hue_deg: [330.0, 30.0],
                    lightness: 0.5,
                    ..HueShift::default()
                },
            ],
        };
        let hsl = |hue, saturation, lightness| Srgba::from(Hsla::hsl(hue, saturation, lightness));
        let hue_of = |color: Srgba| Hsla::from(color).hue;

        // A green turns red.
        let green = colorway.recolor(hsl(120.0, 0.5, 0.4));
        assert!((hue_of(green) - 0.0).abs() < 0.5 || (hue_of(green) - 360.0).abs() < 0.5);
        // A greyish green is too grey for the first shift, and no other holds it.
        let grey_green = hsl(120.0, 0.1, 0.4);
        assert_eq!(colorway.recolor(grey_green), grey_green);
        // A red, held by the shift that wraps round 0, is darkened.
        let red = colorway.recolor(hsl(350.0, 0.6, 0.6));
        assert!((Hsla::from(red).lightness - 0.3).abs() < 0.01);
        // A blue is held by none.
        let blue = hsl(230.0, 0.4, 0.3);
        assert_eq!(colorway.recolor(blue), blue);
    }
}
