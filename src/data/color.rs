//! `#RRGGBB` colors, as the YAML files write them.

use std::{
    fmt::{self, Display, Formatter},
    str::FromStr,
};

use bevy::prelude::Color;
use serde::{Deserialize, Deserializer, de::Error as DeserializeError};

/// A color written as `#RRGGBB`, kept as its channels so it is written back
/// exactly as it reads.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct HexColor([u8; 3]);

impl FromStr for HexColor {
    type Err = &'static str;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let hex = value.strip_prefix('#').unwrap_or(value);
        if hex.len() != 6 {
            return Err("hex colors must contain six digits");
        }
        let rgb = u32::from_str_radix(hex, 16).map_err(|_| "hex colors must contain six digits")?;
        Ok(Self([(rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8]))
    }
}

impl<'de> Deserialize<'de> for HexColor {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer)?
            .parse()
            .map_err(D::Error::custom)
    }
}

impl Display for HexColor {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        let [red, green, blue] = self.0;
        write!(formatter, "#{red:02X}{green:02X}{blue:02X}")
    }
}

impl From<HexColor> for Color {
    fn from(color: HexColor) -> Self {
        let [red, green, blue] = color.0;
        Self::srgb_u8(red, green, blue)
    }
}

impl From<Color> for HexColor {
    fn from(color: Color) -> Self {
        let srgb = color.to_srgba();
        let channel = |value: f32| (value.clamp(0.0, 1.0) * 255.0).round() as u8;
        Self([channel(srgb.red), channel(srgb.green), channel(srgb.blue)])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_colors_read_and_write_back_the_same_digits() {
        let color: HexColor = "#2DE2D5".parse().expect("six digits parse");
        assert_eq!(color.to_string(), "#2DE2D5");
        assert_eq!(HexColor::from(Color::from(color)), color);
        assert!("#2DE2".parse::<HexColor>().is_err());
        assert!("#2DE2DZ".parse::<HexColor>().is_err());
    }
}
