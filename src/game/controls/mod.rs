use crate::{
    data::{ControlsConfig, GameData, GamepadControlsConfig},
    game::settings::PreferencesDir,
};
use bevy::{
    input::{
        ButtonState,
        keyboard::{Key, KeyboardInput},
    },
    prelude::*,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fmt::{self, Display, Formatter},
    fs,
    str::FromStr,
};

mod key_codes;
mod pad;

use key_codes::{KeyBindingName, KeyLabel};
use pad::PadInput;

/// The letters typed this frame on the active keyboard layout. Menu letter
/// shortcuts match these instead of key positions, so M is the key labeled M
/// on AZERTY too (where it sits at QWERTY's semicolon).
pub(super) struct TypedLetters(Vec<char>);

impl TypedLetters {
    pub(super) fn contains(&self, letter: char) -> bool {
        self.0.contains(&letter)
    }
}

impl<'a> FromIterator<&'a KeyboardInput> for TypedLetters {
    fn from_iter<I: IntoIterator<Item = &'a KeyboardInput>>(inputs: I) -> Self {
        Self(
            inputs
                .into_iter()
                .filter(|input| input.state == ButtonState::Pressed && !input.repeat)
                .filter_map(|input| match &input.logical_key {
                    Key::Character(text) => text.chars().next(),
                    _ => None,
                })
                .map(|letter| letter.to_ascii_lowercase())
                .collect(),
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(super) enum GameAction {
    MoveLeft,
    MoveRight,
    /// Climbs a ladder.
    MoveUp,
    MoveDown,
    NavigateUp,
    NavigateDown,
    Jump,
    Attack,
    /// The slow blow that goes through a shield, and ends the chain on a
    /// heavier finisher.
    HeavyAttack,
    /// A short invulnerable dash, on the ground and in mid-air.
    Dash,
    Interact,
    Pause,
    ToggleFullscreen,
    ChooseFirstBonus,
    ChooseSecondBonus,
    ChooseThirdBonus,
    RerollBonus,
    /// Opens the window listing the run's bonuses, and closes it.
    ShowBuild,
    /// Presses the menu button the focus stands on.
    Confirm,
    /// Leaves a menu.
    Back,
}

impl GameAction {
    pub const ALL: [Self; 20] = [
        Self::MoveLeft,
        Self::MoveRight,
        Self::MoveUp,
        Self::MoveDown,
        Self::NavigateUp,
        Self::NavigateDown,
        Self::Jump,
        Self::Attack,
        Self::HeavyAttack,
        Self::Dash,
        Self::Interact,
        Self::Pause,
        Self::ToggleFullscreen,
        Self::ChooseFirstBonus,
        Self::ChooseSecondBonus,
        Self::ChooseThirdBonus,
        Self::RerollBonus,
        Self::ShowBuild,
        Self::Confirm,
        Self::Back,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::MoveLeft => "MOVE LEFT",
            Self::MoveRight => "MOVE RIGHT",
            Self::MoveUp => "CLIMB UP",
            Self::MoveDown => "CROUCH",
            Self::NavigateUp => "MENU UP",
            Self::NavigateDown => "MENU DOWN",
            Self::Jump => "JUMP",
            Self::Attack => "ATTACK",
            Self::HeavyAttack => "HEAVY ATTACK",
            Self::Dash => "DASH",
            Self::Interact => "INTERACT / CONFIRM",
            Self::Pause => "PAUSE",
            Self::ToggleFullscreen => "FULLSCREEN",
            Self::ChooseFirstBonus => "FIRST BONUS",
            Self::ChooseSecondBonus => "SECOND BONUS",
            Self::ChooseThirdBonus => "THIRD BONUS",
            Self::RerollBonus => "REROLL BONUSES",
            Self::ShowBuild => "SHOW BONUSES",
            Self::Confirm => "MENU CONFIRM",
            Self::Back => "MENU BACK",
        }
    }
}

impl FromStr for GameAction {
    type Err = &'static str;

    fn from_str(id: &str) -> Result<Self, Self::Err> {
        match id {
            "move_left" => Ok(Self::MoveLeft),
            "move_right" => Ok(Self::MoveRight),
            "move_up" => Ok(Self::MoveUp),
            "move_down" => Ok(Self::MoveDown),
            "navigate_up" => Ok(Self::NavigateUp),
            "navigate_down" => Ok(Self::NavigateDown),
            "jump" => Ok(Self::Jump),
            "attack" => Ok(Self::Attack),
            "heavy_attack" => Ok(Self::HeavyAttack),
            "dash" => Ok(Self::Dash),
            "interact" => Ok(Self::Interact),
            "pause" => Ok(Self::Pause),
            "toggle_fullscreen" => Ok(Self::ToggleFullscreen),
            "choose_first_bonus" => Ok(Self::ChooseFirstBonus),
            "choose_second_bonus" => Ok(Self::ChooseSecondBonus),
            "choose_third_bonus" => Ok(Self::ChooseThirdBonus),
            "reroll_bonus" => Ok(Self::RerollBonus),
            "show_build" => Ok(Self::ShowBuild),
            "confirm" => Ok(Self::Confirm),
            "back" => Ok(Self::Back),
            _ => Err("unknown game action"),
        }
    }
}

impl Display for GameAction {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::MoveLeft => "move_left",
            Self::MoveRight => "move_right",
            Self::MoveUp => "move_up",
            Self::MoveDown => "move_down",
            Self::NavigateUp => "navigate_up",
            Self::NavigateDown => "navigate_down",
            Self::Jump => "jump",
            Self::Attack => "attack",
            Self::HeavyAttack => "heavy_attack",
            Self::Dash => "dash",
            Self::Interact => "interact",
            Self::Pause => "pause",
            Self::ToggleFullscreen => "toggle_fullscreen",
            Self::ChooseFirstBonus => "choose_first_bonus",
            Self::ChooseSecondBonus => "choose_second_bonus",
            Self::ChooseThirdBonus => "choose_third_bonus",
            Self::RerollBonus => "reroll_bonus",
            Self::ShowBuild => "show_build",
            Self::Confirm => "confirm",
            Self::Back => "back",
        })
    }
}

#[derive(Resource, Default)]
pub(super) struct ControlBindings {
    bindings: BTreeMap<GameAction, Vec<KeyCode>>,
    defaults: BTreeMap<GameAction, Vec<KeyCode>>,
    /// The action waiting for a key to be bound to it, while the controls
    /// screen is listening for one.
    pub(crate) rebinding: Option<GameAction>,
    /// The gamepad buttons of each action, from the YAML; never rebound.
    pad: BTreeMap<GameAction, Vec<PadInput>>,
    stick_threshold: f32,
    /// The actions a gamepad holds, read once a frame by [`read_gamepads`].
    pad_held: ButtonInput<GameAction>,
    /// Whether a gamepad was touched after the keyboard, which makes the
    /// labels name its buttons.
    pad_in_use: bool,
}

struct DefaultBindings(BTreeMap<GameAction, Vec<KeyCode>>);

impl From<&ControlsConfig> for DefaultBindings {
    fn from(controls: &ControlsConfig) -> Self {
        let parse_keys = |names: &[String]| {
            names
                .iter()
                .filter_map(|name| name.parse::<KeyBindingName>().ok().map(KeyCode::from))
                .collect::<Vec<_>>()
        };
        Self(BTreeMap::from([
            (GameAction::MoveLeft, parse_keys(&controls.move_left)),
            (GameAction::MoveRight, parse_keys(&controls.move_right)),
            (GameAction::MoveUp, parse_keys(&controls.move_up)),
            (GameAction::MoveDown, parse_keys(&controls.move_down)),
            (GameAction::NavigateUp, parse_keys(&controls.navigate_up)),
            (
                GameAction::NavigateDown,
                parse_keys(&controls.navigate_down),
            ),
            (GameAction::Jump, parse_keys(&controls.jump)),
            (GameAction::Attack, parse_keys(&controls.attack)),
            (GameAction::HeavyAttack, parse_keys(&controls.heavy_attack)),
            (GameAction::Dash, parse_keys(&controls.dash)),
            (GameAction::Interact, parse_keys(&controls.interact)),
            (GameAction::Pause, parse_keys(&controls.pause)),
            (
                GameAction::ToggleFullscreen,
                parse_keys(&controls.toggle_fullscreen),
            ),
            (
                GameAction::ChooseFirstBonus,
                parse_keys(&controls.choose_first_bonus),
            ),
            (
                GameAction::ChooseSecondBonus,
                parse_keys(&controls.choose_second_bonus),
            ),
            (
                GameAction::ChooseThirdBonus,
                parse_keys(&controls.choose_third_bonus),
            ),
            (GameAction::RerollBonus, parse_keys(&controls.reroll_bonus)),
            (GameAction::ShowBuild, parse_keys(&controls.show_build)),
            (GameAction::Confirm, parse_keys(&controls.confirm)),
            (GameAction::Back, parse_keys(&controls.back)),
        ]))
    }
}

/// The gamepad buttons of each action, as `controls.gamepad` of the YAML
/// names them. A name that reads as no action or no button is left out.
struct PadBindings(BTreeMap<GameAction, Vec<PadInput>>);

impl From<&GamepadControlsConfig> for PadBindings {
    fn from(gamepad: &GamepadControlsConfig) -> Self {
        Self(
            gamepad
                .bindings
                .iter()
                .filter_map(|(action, names)| {
                    let inputs = names.iter().filter_map(|name| name.parse().ok());
                    Some((action.parse().ok()?, inputs.collect()))
                })
                .collect(),
        )
    }
}

impl ControlBindings {
    /// Whether the action is held, on the keyboard or a gamepad.
    pub fn pressed(&self, action: GameAction, keys: &ButtonInput<KeyCode>) -> bool {
        self.pad_held.pressed(action)
            || self
                .bindings
                .get(&action)
                .is_some_and(|bindings| keys.any_pressed(bindings.iter().copied()))
    }

    /// Whether the action was pressed this frame, on the keyboard or a gamepad.
    pub fn just_pressed(&self, action: GameAction, keys: &ButtonInput<KeyCode>) -> bool {
        self.pad_held.just_pressed(action)
            || self
                .bindings
                .get(&action)
                .is_some_and(|bindings| keys.any_just_pressed(bindings.iter().copied()))
    }

    /// The gamepad's buttons for the action while the pad is in use and it
    /// has some, else its keys.
    fn labels(&self, action: GameAction) -> Vec<String> {
        match self.pad.get(&action) {
            Some(inputs) if self.pad_in_use && !inputs.is_empty() => inputs
                .iter()
                .map(|input| input.label().to_owned())
                .collect(),
            _ => self
                .bindings
                .get(&action)
                .into_iter()
                .flatten()
                .map(|key| KeyLabel::from(*key).to_string())
                .collect(),
        }
    }

    pub fn binding_label(&self, action: GameAction) -> String {
        self.labels(action).join(" / ")
    }

    pub fn primary_label(&self, action: GameAction) -> String {
        self.labels(action).into_iter().next().unwrap_or_default()
    }

    /// Binds the next key pressed to the action being rebound, taking it off
    /// whatever else held it. Returns whether a key was bound.
    pub fn capture_key(&mut self, keys: &ButtonInput<KeyCode>) -> bool {
        let Some(action) = self.rebinding else {
            return false;
        };
        let Some(key) = keys
            .get_just_pressed()
            .copied()
            .find(|key| KeyBindingName::try_from(*key).is_ok())
        else {
            return false;
        };
        for bindings in self.bindings.values_mut() {
            bindings.retain(|bound| *bound != key);
        }
        self.bindings.insert(action, vec![key]);
        self.rebinding = None;
        true
    }

    pub fn restore_defaults(&mut self, preferences: &PreferencesDir) {
        self.bindings.clone_from(&self.defaults);
        self.rebinding = None;
        save_bindings(self, preferences);
    }
}

/// Reads every connected gamepad into the actions it holds, so
/// [`ControlBindings::pressed`] hears a pad as it hears a key. Whichever of
/// the keyboard and a pad was touched last decides which names the screen
/// shows.
pub(super) fn read_gamepads(
    gamepads: Query<&Gamepad>,
    keys: Res<ButtonInput<KeyCode>>,
    mut bindings: ResMut<ControlBindings>,
) {
    // Written every frame: bypassed so that nothing watching the bindings
    // for a rebind thinks they changed.
    let bindings = bindings.bypass_change_detection();
    bindings.pad_held.clear();
    let threshold = bindings.stick_threshold;
    for (action, inputs) in &bindings.pad {
        let held = gamepads.iter().any(|pad| {
            inputs.iter().any(|input| {
                input.held(pad, threshold)
                    || matches!(input, PadInput::Button(button) if pad.just_pressed(*button))
            })
        });
        match (held, bindings.pad_held.pressed(*action)) {
            (true, false) => bindings.pad_held.press(*action),
            (false, true) => bindings.pad_held.release(*action),
            _ => {}
        }
    }
    if keys.get_just_pressed().next().is_some() {
        bindings.pad_in_use = false;
    } else if bindings.pad_held.get_just_pressed().next().is_some() {
        bindings.pad_in_use = true;
    }
}

#[derive(Deserialize, Serialize, Default)]
struct SavedBindings {
    bindings: BTreeMap<String, Vec<String>>,
}

pub(super) fn initialize_control_bindings(
    data: Res<GameData>,
    preferences: Res<PreferencesDir>,
    mut bindings: ResMut<ControlBindings>,
) {
    let controls = &data.gameplay.controls;
    bindings.pad = PadBindings::from(&controls.gamepad).0;
    bindings.stick_threshold = controls.gamepad.stick_threshold;
    let defaults = DefaultBindings::from(controls).0;
    bindings.defaults = defaults.clone();
    bindings.bindings = defaults;

    let Some(path) = preferences.file("controls.yaml") else {
        return;
    };
    let Ok(serialized) = fs::read_to_string(path) else {
        return;
    };
    let Ok(saved) = yaml_serde::from_str::<SavedBindings>(&serialized) else {
        return;
    };
    for (action_id, names) in saved.bindings {
        let Ok(action) = action_id.parse::<GameAction>() else {
            continue;
        };
        let keys = names
            .iter()
            .filter_map(|name| name.parse::<KeyBindingName>().ok().map(KeyCode::from))
            .collect::<Vec<_>>();
        if !keys.is_empty() {
            bindings.bindings.insert(action, keys);
        }
    }
}

pub(super) fn save_bindings(bindings: &ControlBindings, preferences: &PreferencesDir) {
    let Some(path) = preferences.file("controls.yaml") else {
        return;
    };
    let Some(directory) = path.parent() else {
        return;
    };
    let saved = SavedBindings {
        bindings: bindings
            .bindings
            .iter()
            .map(|(action, keys)| {
                (
                    action.to_string(),
                    keys.iter()
                        .filter_map(|key| KeyBindingName::try_from(*key).ok())
                        .map(|name| name.to_string())
                        .collect(),
                )
            })
            .collect(),
    };
    let Ok(serialized) = yaml_serde::to_string(&saved) else {
        return;
    };
    if fs::create_dir_all(directory).is_ok() {
        let _ = fs::write(path, serialized);
    }
}
