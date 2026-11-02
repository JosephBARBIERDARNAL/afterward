use std::{
    fmt::{self, Display, Formatter},
    str::FromStr,
};

use bevy::prelude::KeyCode;

#[derive(Clone, Copy)]
pub(super) struct KeyBindingName(KeyCode);

#[derive(Debug)]
pub(super) struct UnknownKeyCode;

impl Display for UnknownKeyCode {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str("unsupported key code")
    }
}

impl std::error::Error for UnknownKeyCode {}

macro_rules! key_binding_names {
    ($($name:literal => $key:ident),+ $(,)?) => {
        impl FromStr for KeyBindingName {
            type Err = UnknownKeyCode;

            fn from_str(value: &str) -> Result<Self, Self::Err> {
                let key = match value {
                    $($name => KeyCode::$key,)+
                    _ => return Err(UnknownKeyCode),
                };
                Ok(Self(key))
            }
        }

        impl TryFrom<KeyCode> for KeyBindingName {
            type Error = UnknownKeyCode;

            fn try_from(key: KeyCode) -> Result<Self, Self::Error> {
                match key {
                    $(KeyCode::$key => Ok(Self(key)),)+
                    _ => Err(UnknownKeyCode),
                }
            }
        }

        impl From<KeyBindingName> for KeyCode {
            fn from(name: KeyBindingName) -> Self {
                name.0
            }
        }

        impl Display for KeyBindingName {
            fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
                formatter.write_str(match self.0 {
                    $(KeyCode::$key => $name,)+
                    _ => "Unknown",
                })
            }
        }
    };
}

key_binding_names! {
    "KeyA" => KeyA,
    "KeyB" => KeyB,
    "KeyC" => KeyC,
    "KeyD" => KeyD,
    "KeyE" => KeyE,
    "KeyF" => KeyF,
    "KeyG" => KeyG,
    "KeyH" => KeyH,
    "KeyI" => KeyI,
    "KeyJ" => KeyJ,
    "KeyK" => KeyK,
    "KeyL" => KeyL,
    "KeyM" => KeyM,
    "KeyN" => KeyN,
    "KeyO" => KeyO,
    "KeyP" => KeyP,
    "KeyQ" => KeyQ,
    "KeyR" => KeyR,
    "KeyS" => KeyS,
    "KeyT" => KeyT,
    "KeyU" => KeyU,
    "KeyV" => KeyV,
    "KeyW" => KeyW,
    "KeyX" => KeyX,
    "KeyY" => KeyY,
    "KeyZ" => KeyZ,
    "Digit0" => Digit0,
    "Digit1" => Digit1,
    "Digit2" => Digit2,
    "Digit3" => Digit3,
    "Digit4" => Digit4,
    "Digit5" => Digit5,
    "Digit6" => Digit6,
    "Digit7" => Digit7,
    "Digit8" => Digit8,
    "Digit9" => Digit9,
    "F1" => F1,
    "F2" => F2,
    "F3" => F3,
    "F4" => F4,
    "F5" => F5,
    "F6" => F6,
    "F7" => F7,
    "F8" => F8,
    "F9" => F9,
    "F10" => F10,
    "F11" => F11,
    "F12" => F12,
    "ArrowDown" => ArrowDown,
    "ArrowLeft" => ArrowLeft,
    "ArrowRight" => ArrowRight,
    "ArrowUp" => ArrowUp,
    "Space" => Space,
    "ShiftLeft" => ShiftLeft,
    "ShiftRight" => ShiftRight,
    "ControlLeft" => ControlLeft,
    "ControlRight" => ControlRight,
    "AltLeft" => AltLeft,
    "AltRight" => AltRight,
    "Enter" => Enter,
    "Escape" => Escape,
    "Tab" => Tab,
    "Backspace" => Backspace,
    "Delete" => Delete,
    "Home" => Home,
    "End" => End,
    "PageUp" => PageUp,
    "PageDown" => PageDown,
    "Minus" => Minus,
    "Equal" => Equal,
    "BracketLeft" => BracketLeft,
    "BracketRight" => BracketRight,
    "Semicolon" => Semicolon,
    "Quote" => Quote,
    "Comma" => Comma,
    "Period" => Period,
    "Slash" => Slash,
    "Backslash" => Backslash,
    "Backquote" => Backquote,
    "NumpadEnter" => NumpadEnter,
    "Numpad0" => Numpad0,
    "Numpad1" => Numpad1,
    "Numpad2" => Numpad2,
    "Numpad3" => Numpad3,
    "Numpad4" => Numpad4,
    "Numpad5" => Numpad5,
    "Numpad6" => Numpad6,
    "Numpad7" => Numpad7,
    "Numpad8" => Numpad8,
    "Numpad9" => Numpad9,
    "NumpadAdd" => NumpadAdd,
    "NumpadSubtract" => NumpadSubtract,
    "NumpadMultiply" => NumpadMultiply,
    "NumpadDivide" => NumpadDivide,
    "NumpadDecimal" => NumpadDecimal,
}

pub(super) struct KeyLabel(KeyCode);

impl From<KeyCode> for KeyLabel {
    fn from(key: KeyCode) -> Self {
        Self(key)
    }
}

impl Display for KeyLabel {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        let Ok(name) = KeyBindingName::try_from(self.0) else {
            return formatter.write_str("Unknown");
        };
        let name = name.to_string();
        match name.as_str() {
            "ArrowLeft" => formatter.write_str("LEFT"),
            "ArrowRight" => formatter.write_str("RIGHT"),
            "ArrowUp" => formatter.write_str("UP"),
            "ArrowDown" => formatter.write_str("DOWN"),
            name => formatter.write_str(
                name.strip_prefix("Key")
                    .or_else(|| name.strip_prefix("Digit"))
                    .unwrap_or(name),
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arrow_key_labels_use_characters_available_in_the_game_font() {
        assert_eq!(KeyLabel::from(KeyCode::ArrowUp).to_string(), "UP");
        assert_eq!(KeyLabel::from(KeyCode::ArrowDown).to_string(), "DOWN");
        assert_eq!(KeyLabel::from(KeyCode::ArrowLeft).to_string(), "LEFT");
        assert_eq!(KeyLabel::from(KeyCode::ArrowRight).to_string(), "RIGHT");
    }

    #[test]
    fn letter_and_digit_labels_drop_their_key_code_prefix() {
        assert_eq!(KeyLabel::from(KeyCode::KeyR).to_string(), "R");
        assert_eq!(KeyLabel::from(KeyCode::Digit1).to_string(), "1");
    }
}
