//! Main, pause, settings, controls, and end-of-run menus, drawn in the bonus
//! screen's card style. Each frame the current screen is described as a
//! [`MenuView`]; the menu's entities are rebuilt only when that view changes.

use super::*;
use crate::game::ui_style::{
    ACTIVE_BORDER, CARD_BACKGROUND, CARD_HIGHLIGHT_BACKGROUND, DESCRIPTION_COLOR, KEY_BACKGROUND,
    MUTED, key_badge, overlay_root, title,
};
use bevy::text::FontSize;

#[derive(Component)]
pub(super) struct MenuRoot;

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
    /// A 0–100 slider drawn as a bar, for volume rows.
    level: Option<u8>,
    focus: RowFocus,
}

impl MenuRow {
    /// A row triggered directly by its key, like the reroll box.
    fn action(key: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            key: Some(key.into()),
            label: label.into(),
            level: None,
            focus: RowFocus::Action,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum RowFocus {
    Action,
    Selected,
    Unselected,
}

impl From<bool> for RowFocus {
    fn from(selected: bool) -> Self {
        if selected {
            Self::Selected
        } else {
            Self::Unselected
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
    settings: Res<'w, AudioSettings>,
    controls: Res<'w, ControlBindings>,
}

impl MenuState<'_> {
    fn view(&self) -> Option<MenuView> {
        let controls = &self.controls;
        let key = |action| controls.primary_label(action);
        let pair = |first, second| format!("{}/{}", key(first), key(second));
        let editor_row = cfg!(debug_assertions).then(|| MenuRow::action("D", "Debug editor"));
        let back_to_menu = || vec![MenuRow::action("ENTER", "Return to menu")];
        let plain = |title, subtitle, rows| MenuView {
            title,
            subtitle,
            rows,
            hints: Vec::new(),
            two_columns: false,
        };
        let view = match self.screen.get() {
            GameScreen::MainMenu => plain(
                "AFTERWARD",
                None,
                [
                    MenuRow::action(key(GameAction::Interact), "Begin run"),
                    MenuRow::action("S", "Settings"),
                ]
                .into_iter()
                .chain(editor_row)
                .chain([MenuRow::action("Q", "Quit")])
                .collect(),
            ),
            GameScreen::Paused => plain(
                "PAUSED",
                None,
                [
                    MenuRow::action("ENTER", "Continue"),
                    MenuRow::action("S", "Settings"),
                    MenuRow::action("R", "Restart"),
                    MenuRow::action("M", "Main menu"),
                ]
                .into_iter()
                .chain(editor_row)
                .chain([MenuRow::action("Q", "Quit")])
                .collect(),
            ),
            GameScreen::Settings => {
                let slider = |index, label: &str, level| MenuRow {
                    key: None,
                    label: label.to_owned(),
                    level: Some(level),
                    focus: (self.settings.selected_slider == index).into(),
                };
                MenuView {
                    title: "SETTINGS",
                    subtitle: None,
                    rows: vec![
                        slider(0, "Music", self.settings.music_volume_percent),
                        slider(1, "Sound", self.settings.sfx_volume_percent),
                        MenuRow::action(key(GameAction::Interact), "Controls"),
                        MenuRow::action(key(GameAction::ToggleFullscreen), "Fullscreen"),
                        MenuRow::action(key(GameAction::Pause), "Back"),
                    ],
                    hints: vec![
                        MenuHint {
                            key: pair(GameAction::NavigateUp, GameAction::NavigateDown),
                            label: "Select",
                        },
                        MenuHint {
                            key: pair(GameAction::MoveLeft, GameAction::MoveRight),
                            label: "Adjust",
                        },
                    ],
                    two_columns: false,
                }
            }
            GameScreen::Controls => {
                let rows = GameAction::ALL
                    .iter()
                    .enumerate()
                    .map(|(index, action)| {
                        let selected = controls.selected_action_index == index;
                        MenuRow {
                            key: Some(if selected && controls.waiting_for_key {
                                String::from("...")
                            } else {
                                controls.binding_label(*action)
                            }),
                            label: action.label().to_owned(),
                            level: None,
                            focus: selected.into(),
                        }
                    })
                    .collect();
                let hint = |key, label| MenuHint { key, label };
                let (subtitle, hints) = if controls.waiting_for_key {
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
                            hint(String::from("R"), "Restore defaults"),
                            hint(key(GameAction::Pause), "Back"),
                        ],
                    )
                };
                MenuView {
                    title: "CONTROLS",
                    subtitle,
                    rows,
                    hints,
                    two_columns: true,
                }
            }
            GameScreen::Dead => plain(
                "RUN ENDED",
                Some(format!("Fell in room {:02}", self.session.room_id)),
                back_to_menu(),
            ),
            GameScreen::Complete => plain("CHAPTER ONE CLEARED", None, back_to_menu()),
            _ => return None,
        };
        Some(view)
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
                        for row in &self.rows {
                            row.spawn(rows, compact);
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
    fn spawn(&self, parent: &mut ChildSpawnerCommands, compact: bool) {
        let (background, border) = match self.focus {
            RowFocus::Action => (CARD_BACKGROUND, ACTIVE_BORDER),
            RowFocus::Selected => (CARD_HIGHLIGHT_BACKGROUND, ACTIVE_BORDER),
            RowFocus::Unselected => (CARD_BACKGROUND, MUTED),
        };
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
                BackgroundColor(background),
                BorderColor::all(border),
                Pickable::IGNORE,
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
                if let Some(level) = self.level {
                    row.spawn((
                        Node {
                            width: px(180),
                            height: px(12),
                            border_radius: BorderRadius::all(px(3)),
                            ..default()
                        },
                        BackgroundColor(KEY_BACKGROUND),
                        Pickable::IGNORE,
                    ))
                    .with_child((
                        Node {
                            width: percent(f32::from(level)),
                            height: percent(100),
                            border_radius: BorderRadius::all(px(3)),
                            ..default()
                        },
                        BackgroundColor(ACTIVE_BORDER),
                        Pickable::IGNORE,
                    ));
                    row.spawn((
                        text(format!("{level}%"), label_font_px, DESCRIPTION_COLOR),
                        TextLayout::justify(Justify::Right),
                        Node {
                            min_width: px(64),
                            ..default()
                        },
                    ));
                }
            });
    }
}
