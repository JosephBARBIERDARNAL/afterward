use super::{GameData, settings::preferences_path};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fs};

mod key_codes;

use key_codes::{key_from_name, key_label, key_name};
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) enum GameAction {
    MoveLeft,
    MoveRight,
    NavigateUp,
    NavigateDown,
    Jump,
    Dash,
    Sword,
    Gun,
    Interact,
    Pause,
}

impl GameAction {
    pub const ALL: [Self; 10] = [
        Self::MoveLeft,
        Self::MoveRight,
        Self::NavigateUp,
        Self::NavigateDown,
        Self::Jump,
        Self::Dash,
        Self::Sword,
        Self::Gun,
        Self::Interact,
        Self::Pause,
    ];

    fn id(self) -> &'static str {
        match self {
            Self::MoveLeft => "move_left",
            Self::MoveRight => "move_right",
            Self::NavigateUp => "navigate_up",
            Self::NavigateDown => "navigate_down",
            Self::Jump => "jump",
            Self::Dash => "dash",
            Self::Sword => "sword",
            Self::Gun => "gun",
            Self::Interact => "interact",
            Self::Pause => "pause",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::MoveLeft => "MOVE LEFT",
            Self::MoveRight => "MOVE RIGHT",
            Self::NavigateUp => "MENU UP",
            Self::NavigateDown => "MENU DOWN",
            Self::Jump => "JUMP",
            Self::Dash => "DASH",
            Self::Sword => "SWORD",
            Self::Gun => "GUN",
            Self::Interact => "INTERACT / CONFIRM",
            Self::Pause => "PAUSE",
        }
    }

    fn from_id(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|action| action.id() == id)
    }
}

#[derive(Resource, Default)]
pub(super) struct ControlBindings {
    bindings: BTreeMap<GameAction, Vec<KeyCode>>,
    defaults: BTreeMap<GameAction, Vec<KeyCode>>,
    selected_action: usize,
    waiting_for_key: bool,
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
        GameAction::ALL[self.selected_action]
    }

    pub fn selected_index(&self) -> usize {
        self.selected_action
    }

    pub fn is_waiting_for_key(&self) -> bool {
        self.waiting_for_key
    }

    pub fn binding_label(&self, action: GameAction) -> String {
        self.bindings
            .get(&action)
            .into_iter()
            .flatten()
            .map(|key| key_label(*key))
            .collect::<Vec<_>>()
            .join(" / ")
    }

    pub fn move_selection(&mut self, direction: i8) {
        let len = GameAction::ALL.len();
        self.selected_action = (self.selected_action as isize + isize::from(direction))
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
            .find(|key| key_name(*key).is_some())
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

    pub fn restore_defaults(&mut self) {
        self.bindings.clone_from(&self.defaults);
        self.waiting_for_key = false;
        save_bindings(self);
    }
}

#[derive(Deserialize, Serialize, Default)]
struct SavedBindings {
    bindings: BTreeMap<String, Vec<String>>,
}

pub(super) fn initialize_control_bindings(
    data: Res<GameData>,
    mut bindings: ResMut<ControlBindings>,
) {
    let defaults = default_bindings(&data);
    bindings.defaults = defaults.clone();
    bindings.bindings = defaults;

    let Some(path) = preferences_path("controls.yaml") else {
        return;
    };
    let Ok(serialized) = fs::read_to_string(path) else {
        return;
    };
    let Ok(saved) = yaml_serde::from_str::<SavedBindings>(&serialized) else {
        return;
    };
    for (action_id, names) in saved.bindings {
        let Some(action) = GameAction::from_id(&action_id) else {
            continue;
        };
        let keys = names
            .iter()
            .filter_map(|name| key_from_name(name))
            .collect::<Vec<_>>();
        if !keys.is_empty() {
            bindings.bindings.insert(action, keys);
        }
    }
}

pub(super) fn initialize_default_control_bindings(
    data: Res<GameData>,
    mut bindings: ResMut<ControlBindings>,
) {
    let defaults = default_bindings(&data);
    bindings.defaults = defaults.clone();
    bindings.bindings = defaults;
}

fn default_bindings(data: &GameData) -> BTreeMap<GameAction, Vec<KeyCode>> {
    let controls = &data.gameplay.controls;
    BTreeMap::from([
        (GameAction::MoveLeft, parse_keys(&controls.move_left)),
        (GameAction::MoveRight, parse_keys(&controls.move_right)),
        (GameAction::NavigateUp, parse_keys(&controls.navigate_up)),
        (
            GameAction::NavigateDown,
            parse_keys(&controls.navigate_down),
        ),
        (GameAction::Jump, parse_keys(&controls.jump)),
        (GameAction::Dash, parse_keys(&controls.dash)),
        (GameAction::Sword, parse_keys(&controls.sword)),
        (GameAction::Gun, parse_keys(&controls.gun)),
        (GameAction::Interact, parse_keys(&controls.interact)),
        (GameAction::Pause, parse_keys(&controls.pause)),
    ])
}

pub(super) fn save_bindings(bindings: &ControlBindings) {
    let Some(path) = preferences_path("controls.yaml") else {
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
                    action.id().to_string(),
                    keys.iter()
                        .filter_map(|key| key_name(*key))
                        .map(str::to_string)
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

fn parse_keys(names: &[String]) -> Vec<KeyCode> {
    names
        .iter()
        .filter_map(|name| key_from_name(name))
        .collect()
}
