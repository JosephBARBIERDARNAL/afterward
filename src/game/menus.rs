//! Main, pause, settings, controls, and end-of-run menus, drawn in the bonus
//! screen's card style.
//!
//! Every row is a button: the mouse points at it and clicks it, the arrows
//! walk to it, and the confirm key presses it, while the letter on its badge
//! still works from anywhere on the screen. What each one does is a
//! [`MenuAction`], and [`MenuFocus`] holds the one the player is on.
//!
//! Each frame the current screen is described as a [`MenuView`]. The menu's
//! entities are rebuilt only when that view changes, which is why neither the
//! focus nor a volume level is part of it: those are dressed onto the standing
//! buttons instead, so pointing at a row never rebuilds the row under the
//! pointer.

use super::*;
use crate::game::ui_style::{
    ACTIVE_BORDER, CARD_BACKGROUND, CARD_HIGHLIGHT_BACKGROUND, DESCRIPTION_COLOR, KEY_BACKGROUND,
    MUTED, key_badge, overlay_root, title,
};
use bevy::text::FontSize;

mod actions;
mod input;

pub(super) use actions::{MenuAction, MenuFocus, VolumeChannel};
pub(super) use input::{MenuPressed, navigate_menus};

#[derive(Component)]
pub(super) struct MenuRoot;

/// A menu button: what it does, and where it stands in its menu.
#[derive(Component, Clone, Copy)]
pub(super) struct MenuButton {
    pub(super) action: MenuAction,
    pub(super) index: usize,
}

/// The bar of a volume row. It is not picked itself — the row around it is —
/// but a click inside its span sets the volume to where it landed.
#[derive(Component, Clone, Copy)]
pub(super) struct VolumeBar(pub(super) VolumeChannel);

/// The filled part of a volume bar, and the reading beside it.
#[derive(Component, Clone, Copy)]
pub(super) struct VolumeFill(VolumeChannel);

#[derive(Component, Clone, Copy)]
pub(super) struct VolumeReading(VolumeChannel);

pub(super) fn spawn_menu(mut commands: Commands) {
    commands.spawn((overlay_root(), MenuRoot));
}

/// Everything a menu shows; two equal views draw the same entities.
#[derive(Clone, Debug, PartialEq)]
pub(super) struct MenuView {
    title: &'static str,
    subtitle: Option<String>,
    rows: Vec<MenuRow>,
    hints: Vec<MenuHint>,
    /// Long lists (the controls) are laid out in two columns of compact rows.
    two_columns: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct MenuRow {
    key: Option<String>,
    label: String,
    /// Volume rows draw a bar, whose level is dressed on separately.
    slider: Option<VolumeChannel>,
    action: MenuAction,
}

impl MenuRow {
    /// A row with a letter or key on its badge, which presses it from anywhere
    /// on the screen.
    fn action(key: impl Into<String>, label: impl Into<String>, action: MenuAction) -> Self {
        Self {
            key: Some(key.into()),
            label: label.into(),
            slider: None,
            action,
        }
    }

    /// A volume row: a bar instead of a badge, moved with the left and right
    /// keys or by clicking into it.
    fn volume(label: &str, channel: VolumeChannel) -> Self {
        Self {
            key: None,
            label: label.to_owned(),
            slider: Some(channel),
            action: MenuAction::Volume(channel),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct MenuHint {
    key: String,
    label: &'static str,
}

#[derive(SystemParam)]
pub(super) struct MenuState<'w> {
    screen: Res<'w, State<GameScreen>>,
    session: Res<'w, RunSession>,
    controls: Res<'w, ControlBindings>,
}

impl MenuState<'_> {
    fn view(&self) -> Option<MenuView> {
        let screen = *self.screen.get();
        let key = |action| self.controls.primary_label(action);
        let pair = |first, second| format!("{}/{}", key(first), key(second));
        let hint = |key, label| MenuHint { key, label };
        // The buttons, their order and their look all come from one list, so
        // what is drawn and what the arrows walk can never disagree.
        let rows = || {
            MenuAction::of(screen)
                .into_iter()
                .map(|action| self.row(action))
        };
        let plain = |title, subtitle| MenuView {
            title,
            subtitle,
            rows: rows().collect(),
            hints: Vec::new(),
            two_columns: false,
        };
        let view = match screen {
            GameScreen::MainMenu => plain("AFTERWARD", None),
            GameScreen::Paused => plain("PAUSED", None),
            GameScreen::Settings => MenuView {
                title: "SETTINGS",
                subtitle: None,
                rows: rows().collect(),
                hints: vec![
                    hint(
                        pair(GameAction::NavigateUp, GameAction::NavigateDown),
                        "Select",
                    ),
                    hint(pair(GameAction::MoveLeft, GameAction::MoveRight), "Adjust"),
                ],
                two_columns: false,
            },
            GameScreen::Controls => {
                let (subtitle, hints) = if self.controls.rebinding.is_some() {
                    (
                        Some(String::from("PRESS A KEY TO BIND")),
                        vec![hint(String::from("ESCAPE"), "Cancel")],
                    )
                } else {
                    (
                        None,
                        vec![
                            hint(
                                pair(GameAction::NavigateUp, GameAction::NavigateDown),
                                "Select",
                            ),
                            hint(key(GameAction::Interact), "Remap"),
                        ],
                    )
                };
                MenuView {
                    title: "CONTROLS",
                    subtitle,
                    rows: rows().collect(),
                    hints,
                    two_columns: true,
                }
            }
            GameScreen::Dead => plain(
                "RUN ENDED",
                Some(format!("Fell in room {:02}", self.session.room_id)),
            ),
            GameScreen::Complete => plain("CHAPTER ONE CLEARED", None),
            _ => return None,
        };
        Some(view)
    }

    /// How one button is drawn: the key that presses it from anywhere on the
    /// screen, and what it says.
    fn row(&self, action: MenuAction) -> MenuRow {
        let key = |action| self.controls.primary_label(action);
        let row = |badge: String, label: &str| MenuRow::action(badge, label, action);
        match action {
            MenuAction::BeginRun => row(key(GameAction::Interact), "Begin run"),
            // The prologue draws this one itself, under the words it writes.
            MenuAction::EnterTheCity => row(String::from("ENTER"), "Enter Afterward"),
            MenuAction::Continue => row(String::from("ENTER"), "Continue"),
            MenuAction::Restart => row(String::from("R"), "Restart"),
            MenuAction::OpenSettings => row(String::from("S"), "Settings"),
            MenuAction::OpenControls => row(String::from("C"), "Controls"),
            MenuAction::OpenEditor => row(String::from("D"), "Debug editor"),
            MenuAction::LeaveRun => row(String::from("M"), "Main menu"),
            MenuAction::Quit => row(String::from("Q"), "Quit"),
            MenuAction::Volume(VolumeChannel::Music) => {
                MenuRow::volume("Music", VolumeChannel::Music)
            }
            MenuAction::Volume(VolumeChannel::Sound) => {
                MenuRow::volume("Sound", VolumeChannel::Sound)
            }
            MenuAction::ToggleFullscreen => row(key(GameAction::ToggleFullscreen), "Fullscreen"),
            MenuAction::Rebind(bound) => MenuRow {
                key: Some(if self.controls.rebinding == Some(bound) {
                    String::from("...")
                } else {
                    self.controls.binding_label(bound)
                }),
                label: bound.label().to_owned(),
                slider: None,
                action,
            },
            MenuAction::RestoreDefaultBindings => row(String::from("R"), "Restore defaults"),
            // The end-of-run screens have nowhere to go back to but the menu.
            MenuAction::Back => match self.screen.get() {
                GameScreen::Dead | GameScreen::Complete => {
                    row(String::from("ENTER"), "Return to menu")
                }
                _ => row(key(GameAction::Pause), "Back"),
            },
        }
    }
}

pub(super) fn update_menu(
    mut commands: Commands,
    state: MenuState,
    mut roots: Query<(Entity, &mut Visibility), With<MenuRoot>>,
    mut shown: Local<Option<MenuView>>,
) {
    let view = state.view();
    if *shown == view {
        return;
    }
    for (root, mut visibility) in &mut roots {
        *visibility = if view.is_some() {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        let mut root = commands.entity(root);
        root.despawn_children();
        if let Some(view) = &view {
            root.with_children(|parent| view.spawn(parent));
        }
    }
    *shown = view;
}

/// Lights the button the player is on. The standing buttons are dressed each
/// frame rather than rebuilt, so pointing at one never pulls it out from under
/// the pointer.
pub(super) fn highlight_menu_buttons(
    screen: Res<State<GameScreen>>,
    focus: Res<MenuFocus>,
    mut buttons: Query<(&MenuButton, &mut BackgroundColor, &mut BorderColor)>,
) {
    let on = focus.lit(*screen.get());
    for (button, mut background, mut border) in &mut buttons {
        let lit = on == Some(button.index);
        background.set_if_neq(BackgroundColor(if lit {
            CARD_HIGHLIGHT_BACKGROUND
        } else {
            CARD_BACKGROUND
        }));
        border.set_if_neq(BorderColor::all(if lit { ACTIVE_BORDER } else { MUTED }));
    }
}

/// Fills each volume bar to its setting and writes the reading beside it.
pub(super) fn update_volume_bars(
    settings: Res<AudioSettings>,
    mut fills: Query<(&VolumeFill, &mut Node)>,
    mut readings: Query<(&VolumeReading, &mut Text)>,
) {
    for (fill, mut node) in &mut fills {
        node.width = percent(f32::from(settings.volume_of(fill.0)));
    }
    for (reading, mut text) in &mut readings {
        let reading = format!("{}%", settings.volume_of(reading.0));
        if text.0 != reading {
            text.0 = reading;
        }
    }
}

fn text(content: impl Into<String>, font_px: f32, color: Color) -> impl Bundle {
    (
        Text::new(content),
        TextFont {
            font_size: FontSize::Px(font_px),
            ..default()
        },
        TextColor(color),
        Pickable::IGNORE,
    )
}

impl MenuView {
    fn spawn(&self, parent: &mut ChildSpawnerCommands) {
        let compact = self.two_columns;
        parent
            .spawn((
                Node {
                    width: percent(78),
                    max_width: px(if compact { 960 } else { 560 }),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Stretch,
                    row_gap: px(12),
                    ..default()
                },
                Pickable::IGNORE,
            ))
            .with_children(|panel| {
                panel.spawn(title(self.title));
                if let Some(subtitle) = &self.subtitle {
                    panel.spawn((
                        text(subtitle.clone(), 18.0, DESCRIPTION_COLOR),
                        TextLayout::justify(Justify::Center),
                        Node {
                            margin: UiRect::bottom(px(8)),
                            ..default()
                        },
                    ));
                }
                panel
                    .spawn((
                        Node {
                            flex_direction: FlexDirection::Row,
                            flex_wrap: FlexWrap::Wrap,
                            justify_content: JustifyContent::SpaceBetween,
                            row_gap: px(if compact { 8 } else { 12 }),
                            ..default()
                        },
                        Pickable::IGNORE,
                    ))
                    .with_children(|rows| {
                        for (index, row) in self.rows.iter().enumerate() {
                            row.spawn(rows, index, compact);
                        }
                    });
                if !self.hints.is_empty() {
                    panel
                        .spawn((
                            Node {
                                flex_direction: FlexDirection::Row,
                                flex_wrap: FlexWrap::Wrap,
                                justify_content: JustifyContent::Center,
                                column_gap: px(24),
                                row_gap: px(8),
                                margin: UiRect::top(px(8)),
                                ..default()
                            },
                            Pickable::IGNORE,
                        ))
                        .with_children(|hints| {
                            for hint in &self.hints {
                                hints
                                    .spawn((
                                        Node {
                                            align_items: AlignItems::Center,
                                            column_gap: px(8),
                                            ..default()
                                        },
                                        Pickable::IGNORE,
                                    ))
                                    .with_children(|hint_node| {
                                        hint_node.spawn(key_badge(28.0)).with_child(text(
                                            hint.key.clone(),
                                            14.0,
                                            Color::WHITE,
                                        ));
                                        hint_node.spawn(text(hint.label, 16.0, DESCRIPTION_COLOR));
                                    });
                            }
                        });
                }
            });
    }
}

impl MenuRow {
    fn spawn(&self, parent: &mut ChildSpawnerCommands, index: usize, compact: bool) {
        parent
            .spawn((
                Node {
                    width: percent(if compact { 49 } else { 100 }),
                    // Rows without a key badge (the volume sliders) keep the
                    // height of the rows that have one.
                    min_height: px(if compact { 48 } else { 72 }),
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: px(16),
                    padding: UiRect::axes(px(16), px(if compact { 6 } else { 12 })),
                    border: UiRect::all(px(2)),
                    border_radius: BorderRadius::all(px(8)),
                    ..default()
                },
                BackgroundColor(CARD_BACKGROUND),
                BorderColor::all(MUTED),
                // No `Pickable::IGNORE` here, and one on everything inside:
                // the row is what the mouse points at and clicks.
                MenuButton {
                    action: self.action,
                    index,
                },
            ))
            .with_children(|row| {
                let (badge_px, key_font_px, label_font_px) = if compact {
                    (32.0, 15.0, 16.0)
                } else {
                    (44.0, 20.0, 22.0)
                };
                if let Some(key) = &self.key {
                    row.spawn(key_badge(badge_px)).with_child(text(
                        key.clone(),
                        key_font_px,
                        Color::WHITE,
                    ));
                }
                row.spawn((
                    text(self.label.clone(), label_font_px, Color::WHITE),
                    Node {
                        flex_grow: 1.0,
                        min_width: px(0),
                        ..default()
                    },
                ));
                if let Some(channel) = self.slider {
                    row.spawn((
                        Node {
                            width: px(180),
                            height: px(12),
                            border_radius: BorderRadius::all(px(3)),
                            ..default()
                        },
                        BackgroundColor(KEY_BACKGROUND),
                        VolumeBar(channel),
                        Pickable::IGNORE,
                    ))
                    .with_child((
                        Node {
                            width: percent(0),
                            height: percent(100),
                            border_radius: BorderRadius::all(px(3)),
                            ..default()
                        },
                        BackgroundColor(ACTIVE_BORDER),
                        VolumeFill(channel),
                        Pickable::IGNORE,
                    ));
                    row.spawn((
                        text(String::new(), label_font_px, DESCRIPTION_COLOR),
                        TextLayout::justify(Justify::Right),
                        Node {
                            min_width: px(64),
                            ..default()
                        },
                        VolumeReading(channel),
                    ));
                }
            });
    }
}
