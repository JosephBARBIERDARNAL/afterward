//! What a gamepad can be bound to: its buttons, and the left stick pushed one
//! way, which counts as held once it leans past `controls.gamepad.stick_threshold`.
//! Names are the ones written in `config/gameplay.yaml`; labels are what the
//! screen shows, the PlayStation names, since the pad the game is tried on is a
//! DualSense (gilrs maps an Xbox pad's A, B, X, Y onto the same four places).

use std::{
    fmt::{self, Display, Formatter},
    str::FromStr,
};

use bevy::prelude::{Gamepad, GamepadButton};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum StickDirection {
    Left,
    Right,
    Up,
    Down,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum PadInput {
    Button(GamepadButton),
    LeftStick(StickDirection),
}

impl PadInput {
    pub(super) fn held(self, pad: &Gamepad, stick_threshold: f32) -> bool {
        match self {
            Self::Button(button) => pad.pressed(button),
            Self::LeftStick(direction) => {
                let stick = pad.left_stick();
                match direction {
                    StickDirection::Left => stick.x <= -stick_threshold,
                    StickDirection::Right => stick.x >= stick_threshold,
                    StickDirection::Up => stick.y >= stick_threshold,
                    StickDirection::Down => stick.y <= -stick_threshold,
                }
            }
        }
    }

    /// What the screen calls it.
    pub(super) fn label(self) -> &'static str {
        match self {
            Self::Button(button) => match button {
                GamepadButton::South => "CROSS",
                GamepadButton::East => "CIRCLE",
                GamepadButton::West => "SQUARE",
                GamepadButton::North => "TRIANGLE",
                GamepadButton::LeftTrigger => "L1",
                GamepadButton::RightTrigger => "R1",
                GamepadButton::LeftTrigger2 => "L2",
                GamepadButton::RightTrigger2 => "R2",
                GamepadButton::Select => "CREATE",
                GamepadButton::Start => "OPTIONS",
                GamepadButton::Mode => "PS",
                GamepadButton::LeftThumb => "L3",
                GamepadButton::RightThumb => "R3",
                GamepadButton::DPadUp => "PAD UP",
                GamepadButton::DPadDown => "PAD DOWN",
                GamepadButton::DPadLeft => "PAD LEFT",
                GamepadButton::DPadRight => "PAD RIGHT",
                _ => "?",
            },
            Self::LeftStick(direction) => match direction {
                StickDirection::Left => "STICK LEFT",
                StickDirection::Right => "STICK RIGHT",
                StickDirection::Up => "STICK UP",
                StickDirection::Down => "STICK DOWN",
            },
        }
    }
}

#[derive(Debug)]
pub(crate) struct UnknownPadInput;

impl Display for UnknownPadInput {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str("unsupported gamepad input")
    }
}

impl std::error::Error for UnknownPadInput {}

impl FromStr for PadInput {
    type Err = UnknownPadInput;

    fn from_str(name: &str) -> Result<Self, Self::Err> {
        let button = match name {
            "LeftStickLeft" => return Ok(Self::LeftStick(StickDirection::Left)),
            "LeftStickRight" => return Ok(Self::LeftStick(StickDirection::Right)),
            "LeftStickUp" => return Ok(Self::LeftStick(StickDirection::Up)),
            "LeftStickDown" => return Ok(Self::LeftStick(StickDirection::Down)),
            "South" => GamepadButton::South,
            "East" => GamepadButton::East,
            "West" => GamepadButton::West,
            "North" => GamepadButton::North,
            "LeftTrigger" => GamepadButton::LeftTrigger,
            "RightTrigger" => GamepadButton::RightTrigger,
            "LeftTrigger2" => GamepadButton::LeftTrigger2,
            "RightTrigger2" => GamepadButton::RightTrigger2,
            "Select" => GamepadButton::Select,
            "Start" => GamepadButton::Start,
            "Mode" => GamepadButton::Mode,
            "LeftThumb" => GamepadButton::LeftThumb,
            "RightThumb" => GamepadButton::RightThumb,
            "DPadUp" => GamepadButton::DPadUp,
            "DPadDown" => GamepadButton::DPadDown,
            "DPadLeft" => GamepadButton::DPadLeft,
            "DPadRight" => GamepadButton::DPadRight,
            _ => return Err(UnknownPadInput),
        };
        Ok(Self::Button(button))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pad_names_read_as_buttons_and_stick_directions() {
        assert_eq!(
            "South".parse::<PadInput>().map(PadInput::label).ok(),
            Some("CROSS")
        );
        assert_eq!(
            "LeftStickDown".parse::<PadInput>().ok(),
            Some(PadInput::LeftStick(StickDirection::Down))
        );
        assert!("Trigger3".parse::<PadInput>().is_err());
    }

    #[test]
    fn every_gamepad_binding_of_the_yaml_names_an_action_and_an_input() {
        let asset_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/assets");
        let data = crate::data::GameData::load(&asset_root).expect("game data should load");
        for (action, names) in &data.gameplay.controls.gamepad.bindings {
            assert!(
                action.parse::<super::super::GameAction>().is_ok(),
                "{action} is no action"
            );
            for name in names {
                assert!(name.parse::<PadInput>().is_ok(), "{name} is no pad input");
            }
        }
    }
}
