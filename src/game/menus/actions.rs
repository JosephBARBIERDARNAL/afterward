//! What the buttons of a menu do, and which one the player is on.
//!
//! Every menu is a list of buttons in the order they are drawn. The same list
//! answers for the keyboard and for the mouse: arrows walk it, the pointer
//! points into it, and either way [`MenuFocus`] holds where the player is.

use super::*;

/// What pressing a menu button does. One of these rides on every button, so
/// a click and the confirm key reach the same place.
#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::game) enum MenuAction {
    BeginRun,
    /// Walks into the city, from the words the prologue writes.
    EnterTheCity,
    Continue,
    Restart,
    OpenSettings,
    OpenControls,
    /// Debug builds only; the editor is compiled out of release builds.
    OpenEditor,
    LeaveRun,
    Quit,
    /// A volume slider, which the left and right keys move.
    Volume(VolumeChannel),
    ToggleFullscreen,
    /// Rebinds one action's key, on the controls screen.
    Rebind(GameAction),
    RestoreDefaultBindings,
    /// Back to whichever menu opened this one.
    Back,
}

/// The two volume sliders of the settings screen.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::game) enum VolumeChannel {
    Music,
    Sound,
}

impl MenuAction {
    /// The buttons of `screen`, in the order they are drawn. The menu draws
    /// this list and the input walks it, so the two never disagree about which
    /// button is where.
    pub(in crate::game) fn of(screen: GameScreen) -> Vec<Self> {
        let editor = cfg!(debug_assertions).then_some(Self::OpenEditor);
        match screen {
            GameScreen::MainMenu => [Self::BeginRun, Self::OpenSettings]
                .into_iter()
                .chain(editor)
                .chain([Self::Quit])
                .collect(),
            GameScreen::Paused => [
                Self::Continue,
                Self::OpenSettings,
                Self::Restart,
                Self::LeaveRun,
            ]
            .into_iter()
            .chain(editor)
            .chain([Self::Quit])
            .collect(),
            GameScreen::Settings => vec![
                Self::Volume(VolumeChannel::Music),
                Self::Volume(VolumeChannel::Sound),
                Self::OpenControls,
                Self::ToggleFullscreen,
                Self::Back,
            ],
            GameScreen::Controls => GameAction::ALL
                .iter()
                .map(|action| Self::Rebind(*action))
                .chain([Self::RestoreDefaultBindings, Self::Back])
                .collect(),
            GameScreen::Prologue => vec![Self::EnterTheCity],
            GameScreen::Dead | GameScreen::Complete => vec![Self::Back],
            _ => Vec::new(),
        }
    }
}

/// The button the player is on, and the menu it belongs to. Opening a menu
/// starts the player on its first button.
#[derive(Resource, Default)]
pub(in crate::game) struct MenuFocus {
    screen: Option<GameScreen>,
    /// Which button of `screen` the player is on, once [`Self::settle`] has
    /// squared it with the menu now on show.
    pub(in crate::game) index: usize,
}

impl MenuFocus {
    /// The button `screen` is focused on, after settling the focus against the
    /// menu now on show: a new menu starts at its first button, and a menu
    /// that lost buttons never leaves the focus past its last one.
    pub(in crate::game) fn settle(&mut self, screen: GameScreen, buttons: usize) -> Option<usize> {
        if self.screen != Some(screen) {
            self.screen = Some(screen);
            self.index = 0;
        }
        if buttons == 0 {
            return None;
        }
        self.index = self.index.min(buttons - 1);
        Some(self.index)
    }

    /// Walks `steps` buttons on, wrapping around the ends of the menu.
    pub(in crate::game) fn walk(&mut self, steps: isize, buttons: usize) {
        if buttons == 0 {
            return;
        }
        self.index = (self.index as isize + steps).rem_euclid(buttons as isize) as usize;
    }

    /// Points the focus straight at a button, as the mouse does.
    pub(in crate::game) fn point_at(&mut self, index: usize) {
        self.index = index;
    }

    /// Which button of `screen` is lit, for whoever only draws the menu and
    /// leaves the settling to the input.
    pub(in crate::game) fn lit(&self, screen: GameScreen) -> Option<usize> {
        (self.screen == Some(screen)).then_some(self.index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_menu_lists_the_buttons_it_draws() {
        assert_eq!(
            MenuAction::of(GameScreen::Settings).first(),
            Some(&MenuAction::Volume(VolumeChannel::Music))
        );
        assert_eq!(
            MenuAction::of(GameScreen::Settings).last(),
            Some(&MenuAction::Back)
        );
        assert_eq!(
            MenuAction::of(GameScreen::Controls).len(),
            GameAction::ALL.len() + 2,
            "every binding, then restore-defaults and back"
        );
        assert_eq!(MenuAction::of(GameScreen::Dead), vec![MenuAction::Back]);
        assert!(
            MenuAction::of(GameScreen::Playing).is_empty(),
            "the game itself has no menu buttons"
        );
        assert!(MenuAction::of(GameScreen::MainMenu).contains(&MenuAction::BeginRun));
    }

    #[test]
    fn the_focus_starts_a_new_menu_at_its_first_button_and_wraps_around_it() {
        let mut focus = MenuFocus::default();
        assert_eq!(focus.settle(GameScreen::Paused, 5), Some(0));

        focus.walk(1, 5);
        focus.walk(1, 5);
        assert_eq!(focus.settle(GameScreen::Paused, 5), Some(2));
        focus.walk(-3, 5);
        assert_eq!(
            focus.settle(GameScreen::Paused, 5),
            Some(4),
            "wraps backwards"
        );
        focus.walk(1, 5);
        assert_eq!(focus.settle(GameScreen::Paused, 5), Some(0), "and forwards");

        // Another menu starts over, and a shorter one never strands the focus.
        focus.walk(3, 5);
        assert_eq!(focus.settle(GameScreen::Settings, 5), Some(0));
        focus.walk(4, 5);
        assert_eq!(focus.settle(GameScreen::Settings, 2), Some(1));
        assert_eq!(focus.settle(GameScreen::Playing, 0), None);
    }
}
