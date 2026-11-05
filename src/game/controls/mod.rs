use crate::{
    data::{ControlsConfig, GameData},
    game::settings::PreferencesDir,
};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fmt::{self, Display, Formatter},
    fs,
    str::FromStr,
};

mod key_codes;

use key_codes::{KeyBindingName, KeyLabel};
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) enum GameAction {
    MoveLeft,
    MoveRight,
    MoveDown,
    NavigateUp,
    NavigateDown,
    Jump,
    Dash,
    Shoot,
    AimUp,
    Reload,
    Cover,
    Interact,
    Pause,
    ToggleFullscreen,
    ChooseFirstBonus,
    ChooseSecondBonus,
    RerollBonus,
}

impl GameAction {
    pub const ALL: [Self; 17] = [
        Self::MoveLeft,
        Self::MoveRight,
        Self::MoveDown,
        Self::NavigateUp,
        Self::NavigateDown,
        Self::Jump,
        Self::Dash,
        Self::Shoot,
        Self::AimUp,
        Self::Reload,
        Self::Cover,
        Self::Interact,
        Self::Pause,
        Self::ToggleFullscreen,
        Self::ChooseFirstBonus,
        Self::ChooseSecondBonus,
        Self::RerollBonus,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::MoveLeft => "MOVE LEFT",
            Self::MoveRight => "MOVE RIGHT",
            Self::MoveDown => "CROUCH / AIM DOWN / DROP",
            Self::NavigateUp => "MENU UP",
            Self::NavigateDown => "MENU DOWN",
            Self::Jump => "JUMP",
            Self::Dash => "DASH",
            Self::Shoot => "SHOOT",
            Self::AimUp => "AIM UP",
            Self::Reload => "RELOAD",
            Self::Cover => "COVER",
            Self::Interact => "INTERACT / CONFIRM",
            Self::Pause => "PAUSE",
            Self::ToggleFullscreen => "FULLSCREEN",
            Self::ChooseFirstBonus => "FIRST BONUS",
            Self::ChooseSecondBonus => "SECOND BONUS",
            Self::RerollBonus => "REROLL BONUSES",
        }
    }
}

impl FromStr for GameAction {
    type Err = &'static str;

    fn from_str(id: &str) -> Result<Self, Self::Err> {
        match id {
            "move_left" => Ok(Self::MoveLeft),
            "move_right" => Ok(Self::MoveRight),
            "move_down" => Ok(Self::MoveDown),
            "navigate_up" => Ok(Self::NavigateUp),
            "navigate_down" => Ok(Self::NavigateDown),
            "jump" => Ok(Self::Jump),
            "dash" => Ok(Self::Dash),
            "shoot" => Ok(Self::Shoot),
            "aim_up" => Ok(Self::AimUp),
            "reload" => Ok(Self::Reload),
            "cover" => Ok(Self::Cover),
            "interact" => Ok(Self::Interact),
            "pause" => Ok(Self::Pause),
            "toggle_fullscreen" => Ok(Self::ToggleFullscreen),
            "choose_first_bonus" => Ok(Self::ChooseFirstBonus),
            "choose_second_bonus" => Ok(Self::ChooseSecondBonus),
            "reroll_bonus" => Ok(Self::RerollBonus),
            _ => Err("unknown game action"),
        }
    }
}

impl Display for GameAction {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::MoveLeft => "move_left",
            Self::MoveRight => "move_right",
            Self::MoveDown => "move_down",
            Self::NavigateUp => "navigate_up",
            Self::NavigateDown => "navigate_down",
            Self::Jump => "jump",
            Self::Dash => "dash",
            Self::Shoot => "shoot",
            Self::AimUp => "aim_up",
            Self::Reload => "reload",
            Self::Cover => "cover",
            Self::Interact => "interact",
            Self::Pause => "pause",
            Self::ToggleFullscreen => "toggle_fullscreen",
            Self::ChooseFirstBonus => "choose_first_bonus",
            Self::ChooseSecondBonus => "choose_second_bonus",
            Self::RerollBonus => "reroll_bonus",
        })
    }
}

#[derive(Resource, Default)]
pub(super) struct ControlBindings {
    bindings: BTreeMap<GameAction, Vec<KeyCode>>,
    defaults: BTreeMap<GameAction, Vec<KeyCode>>,
    pub(crate) selected_action_index: usize,
    pub(crate) waiting_for_key: bool,
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
            (GameAction::MoveDown, parse_keys(&controls.move_down)),
            (GameAction::NavigateUp, parse_keys(&controls.navigate_up)),
            (
                GameAction::NavigateDown,
                parse_keys(&controls.navigate_down),
            ),
            (GameAction::Jump, parse_keys(&controls.jump)),
            (GameAction::Dash, parse_keys(&controls.dash)),
            (GameAction::Shoot, parse_keys(&controls.shoot)),
            (GameAction::AimUp, parse_keys(&controls.aim_up)),
            (GameAction::Reload, parse_keys(&controls.reload)),
            (GameAction::Cover, parse_keys(&controls.cover)),
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
            (GameAction::RerollBonus, parse_keys(&controls.reroll_bonus)),
        ]))
    }
}

impl ControlBindings {
    pub fn pressed(&self, action: GameAction, keys: &ButtonInput<KeyCode>) -> bool {
        self.bindings
            .get(&action)
            .is_some_and(|bindings| keys.any_pressed(bindings.iter().copied()))
    }

    pub fn just_pressed(&self, action: GameAction, keys: &ButtonInput<KeyCode>) -> bool {
        self.bindings
            .get(&action)
            .is_some_and(|bindings| keys.any_just_pressed(bindings.iter().copied()))
    }

    pub fn selected_action(&self) -> GameAction {
        GameAction::ALL[self.selected_action_index]
    }

    pub fn binding_label(&self, action: GameAction) -> String {
        self.bindings
            .get(&action)
            .into_iter()
            .flatten()
            .map(|key| KeyLabel::from(*key).to_string())
            .collect::<Vec<_>>()
            .join(" / ")
    }

    pub fn primary_label(&self, action: GameAction) -> String {
        self.bindings
            .get(&action)
            .and_then(|bindings| bindings.first())
            .map_or_else(String::new, |key| KeyLabel::from(*key).to_string())
    }

    pub fn move_selection(&mut self, direction: i8) {
        let len = GameAction::ALL.len();
        self.selected_action_index = (self.selected_action_index as isize + isize::from(direction))
            .rem_euclid(len as isize) as usize;
    }

    pub fn begin_rebinding(&mut self) {
        self.waiting_for_key = true;
    }

    pub fn cancel_rebinding(&mut self) {
        self.waiting_for_key = false;
    }

    pub fn capture_key(&mut self, keys: &ButtonInput<KeyCode>) -> bool {
        let Some(key) = keys
            .get_just_pressed()
            .copied()
            .find(|key| KeyBindingName::try_from(*key).is_ok())
        else {
            return false;
        };
        let action = self.selected_action();
        for bindings in self.bindings.values_mut() {
            bindings.retain(|bound| *bound != key);
        }
        self.bindings.insert(action, vec![key]);
        self.waiting_for_key = false;
        true
    }

    pub fn restore_defaults(&mut self, preferences: &PreferencesDir) {
        self.bindings.clone_from(&self.defaults);
        self.waiting_for_key = false;
        save_bindings(self, preferences);
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
    let defaults = DefaultBindings::from(&data.gameplay.controls).0;
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
