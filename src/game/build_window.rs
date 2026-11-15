//! The build window: every bonus the run owns, and what each is doing, over
//! the paused room (`GameScreen::Build`, opened and closed with
//! `GameAction::ShowBuild`). The room stays in sight behind it, dimmed but not
//! blurred. Each bonus is a chip in its rarity's colour, edged in its
//! category's while it is at work; the one under the focus (pointed at, or
//! walked to with the arrows) is described in full below the chips, and its
//! words are explained at the bottom right like an offer's (`glossary.rs`).

use super::*;
use crate::{
    data::BonusConfig,
    game::{
        tells::{Line, lines},
        ui_style::{
            CARD_BACKGROUND, CARD_HIGHLIGHT_BACKGROUND, DESCRIPTION_COLOR, DIMMED_EDGE_ALPHA,
            MUTED, focus_ring, key_badge, overlay_root, title,
        },
    },
};
use bevy::text::FontSize;

/// The owned bonuses in the order the window lays them out (by category, then
/// as `config/bonuses.yaml` lists them), and which of them the focus is on.
#[derive(Resource, Default)]
pub(super) struct BuildFocus {
    pub(super) order: Vec<String>,
    pub(super) index: usize,
}

impl BuildFocus {
    /// The bonus the focus stands on, if the run owns any.
    pub(super) fn focused<'a>(&self, data: &'a GameData) -> Option<&'a BonusConfig> {
        self.order
            .get(self.index)
            .and_then(|id| data.bonuses.bonuses.iter().find(|bonus| bonus.id == *id))
    }
}

#[derive(Component)]
pub(super) struct BuildRoot;

/// Where the chips are laid, afresh each time the window opens.
#[derive(Component)]
pub(super) struct BuildGrid;

/// One owned bonus, by its place in `BuildFocus::order`.
#[derive(Component)]
#[require(Outline)]
pub(super) struct BuildChip(pub(super) usize);

#[derive(Component, Clone, Copy)]
pub(super) enum DetailSpan {
    Heading,
    Name,
    Description,
    State,
}

/// The key that closes the window, on its badge.
#[derive(Component)]
pub(super) struct BuildKeyLabel;

const CHIP_FONT_PX: f32 = 15.0;

pub(super) fn spawn_build_window(mut commands: Commands) {
    commands
        .spawn((overlay_root(), BuildRoot))
        .with_children(|root| {
            root.spawn((
                Node {
                    width: percent(58),
                    max_width: px(860),
                    // Clear of the glossary's column at the bottom right.
                    margin: UiRect::right(px(300)),
                    flex_direction: FlexDirection::Column,
                    row_gap: px(14),
                    padding: UiRect::all(px(22)),
                    border: UiRect::all(px(2)),
                    border_radius: BorderRadius::all(px(10)),
                    ..default()
                },
                BackgroundColor(CARD_BACKGROUND),
                BorderColor::all(MUTED),
            ))
            .with_children(|panel| {
                panel.spawn(title("BONUSES"));
                panel.spawn((
                    Node {
                        flex_direction: FlexDirection::Row,
                        flex_wrap: FlexWrap::Wrap,
                        column_gap: px(8),
                        row_gap: px(8),
                        ..default()
                    },
                    BuildGrid,
                ));
                panel
                    .spawn((
                        Text::default(),
                        TextFont {
                            font_size: FontSize::Px(16.0),
                            ..default()
                        },
                        TextLayout::justify(Justify::Left),
                        Node {
                            min_height: px(96),
                            padding: UiRect::top(px(10)),
                            border: UiRect::top(px(1)),
                            ..default()
                        },
                        BorderColor::all(MUTED),
                        Pickable::IGNORE,
                    ))
                    .with_children(|text| {
                        for (kind, font_size, role) in [
                            (DetailSpan::Heading, 14.0, FontRole::Regular),
                            (DetailSpan::Name, 26.0, FontRole::Display),
                            (DetailSpan::Description, 16.0, FontRole::Regular),
                            (DetailSpan::State, 15.0, FontRole::BoldItalic),
                        ] {
                            text.spawn((
                                TextSpan::default(),
                                TextFont {
                                    font_size: FontSize::Px(font_size),
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                                kind,
                                role,
                            ));
                        }
                    });
                panel
                    .spawn(Node {
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        align_self: AlignSelf::End,
                        column_gap: px(10),
                        ..default()
                    })
                    .with_children(|row| {
                        row.spawn(key_badge(32.0)).with_child((
                            Text::default(),
                            TextFont {
                                font_size: FontSize::Px(16.0),
                                ..default()
                            },
                            TextColor(Color::WHITE),
                            BuildKeyLabel,
                            Pickable::IGNORE,
                        ));
                        row.spawn((
                            Text::new("CLOSE"),
                            TextFont {
                                font_size: FontSize::Px(16.0),
                                ..default()
                            },
                            TextColor(MUTED),
                            Pickable::IGNORE,
                        ));
                    });
            });
        });
}

/// Lays the chips out for the build the run owns now, with the focus on the
/// first of them.
pub(super) fn open_build_window(
    mut commands: Commands,
    data: Res<GameData>,
    session: Res<RunSession>,
    mut focus: ResMut<BuildFocus>,
    grids: Query<Entity, With<BuildGrid>>,
) {
    let categories = data.bonuses.categories.keys().collect::<Vec<_>>();
    let mut owned = data
        .bonuses
        .bonuses
        .iter()
        .enumerate()
        .filter(|(_, bonus)| session.has_bonus(&bonus.id))
        .collect::<Vec<_>>();
    owned.sort_by_key(|(listed, bonus)| {
        (
            categories.iter().position(|name| **name == bonus.category),
            *listed,
        )
    });
    *focus = BuildFocus {
        order: owned.iter().map(|(_, bonus)| bonus.id.clone()).collect(),
        index: 0,
    };
    let Ok(grid) = grids.single() else {
        return;
    };
    commands.entity(grid).despawn_children();
    commands.entity(grid).with_children(|grid| {
        if owned.is_empty() {
            grid.spawn((
                Text::new("No bonus yet: clear a room to earn one."),
                TextFont {
                    font_size: FontSize::Px(16.0),
                    ..default()
                },
                TextColor(MUTED),
            ));
        }
        for (index, (_, bonus)) in owned.iter().enumerate() {
            grid.spawn((
                Node {
                    padding: UiRect::axes(px(10), px(6)),
                    border: UiRect::all(px(2)),
                    border_radius: BorderRadius::all(px(6)),
                    ..default()
                },
                BackgroundColor(CARD_BACKGROUND),
                BorderColor::all(MUTED),
                BuildChip(index),
            ))
            .with_child((
                Text::new(bonus.name.clone()),
                TextFont {
                    font_size: FontSize::Px(CHIP_FONT_PX),
                    ..default()
                },
                TextColor(Color::from(data.bonuses.rarities[bonus.rarity].color)),
                FontRole::Display,
                Pickable::IGNORE,
            ));
        }
    });
}

/// Walks the focus with the arrows, or puts it where the pointer is.
pub(super) fn navigate_build_window(
    keys: Res<ButtonInput<KeyCode>>,
    controls: Res<ControlBindings>,
    mut focus: ResMut<BuildFocus>,
    mut sounds: ResMut<SoundQueue>,
    chips: Query<&BuildChip>,
    mut pointer_overs: MessageReader<PointerOver>,
) {
    let count = focus.order.len();
    if count == 0 {
        return;
    }
    let before = focus.index;
    if let Some(chip) = pointer_overs
        .read()
        .filter_map(|over| chips.get(over.entity).ok())
        .last()
    {
        focus.index = chip.0;
    } else if [GameAction::NavigateUp, GameAction::MoveLeft]
        .into_iter()
        .any(|action| controls.just_pressed(action, &keys))
    {
        focus.index = (before + count - 1) % count;
    } else if [GameAction::NavigateDown, GameAction::MoveRight]
        .into_iter()
        .any(|action| controls.just_pressed(action, &keys))
    {
        focus.index = (before + 1) % count;
    }
    if focus.index != before {
        sounds.push(SoundCue::UiMove);
    }
}

#[derive(SystemParam)]
pub(super) struct BuildWindowResources<'w, 's> {
    screen: Res<'w, State<GameScreen>>,
    data: Res<'w, GameData>,
    session: Res<'w, RunSession>,
    focus: Res<'w, BuildFocus>,
    controls: Res<'w, ControlBindings>,
    players: Query<'w, 's, &'static PlayerBody>,
    enemies: Query<'w, 's, &'static EnemyBody>,
}

/// What the bonus is doing, in words, and whether it is at work right now:
/// what the tray under the health bar says of it, or that a once-a-room or
/// once-a-run bonus has been spent.
fn state_of(bonus: &BonusConfig, at_work: &[Line], session: &RunSession) -> (String, bool) {
    if let Some(line) = at_work.iter().find(|line| line.bonus == bonus.id) {
        let left = line
            .left
            .map(|left| format!(", {:.0}% left", left * 100.0))
            .unwrap_or_default();
        return (format!("\nIN FORCE: {}{left}", line.value), true);
    }
    let spent = match bonus.id.as_str() {
        "room_ward" => session.room_ward_used.then_some("SPENT FOR THIS ROOM"),
        "deflector" => session.deflector_used.then_some("SPENT FOR THIS ROOM"),
        "last_stand" => session.last_stand_used.then_some("SPENT FOR THIS RUN"),
        _ => None,
    };
    (
        spent.map(|spent| format!("\n{spent}")).unwrap_or_default(),
        false,
    )
}

pub(super) fn update_build_window(
    resources: BuildWindowResources,
    mut roots: Query<&mut Visibility, With<BuildRoot>>,
    mut chips: Query<(
        &BuildChip,
        &mut BorderColor,
        &mut BackgroundColor,
        &mut Outline,
    )>,
    mut spans: Query<(&DetailSpan, &mut TextSpan, &mut TextColor)>,
    mut key_labels: Query<&mut Text, With<BuildKeyLabel>>,
) {
    let BuildWindowResources {
        screen,
        data,
        session,
        focus,
        controls,
        players,
        enemies,
    } = resources;
    let visible = *screen.get() == GameScreen::Build;
    for mut visibility in &mut roots {
        *visibility = if visible {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
    if !visible {
        return;
    }
    let at_work = players.single().map_or_else(
        |_| Vec::new(),
        |player| {
            let at_rest = !enemies
                .iter()
                .any(|enemy| enemy.health > 0.0 && enemy.engaged);
            lines(
                &session,
                &data,
                player,
                at_rest && session.health < session.max_health,
            )
        },
    );
    let bonus_at = |index: usize| {
        focus
            .order
            .get(index)
            .and_then(|id| data.bonuses.bonuses.iter().find(|bonus| bonus.id == *id))
    };
    let category_color = |bonus: &BonusConfig| {
        data.bonuses
            .categories
            .get(&bonus.category)
            .map_or(Color::WHITE, |category| Color::from(category.color))
    };

    for (chip, mut border, mut background, mut outline) in &mut chips {
        let Some(bonus) = bonus_at(chip.0) else {
            continue;
        };
        let lit = chip.0 == focus.index;
        let working = state_of(bonus, &at_work, &session).1;
        // A bonus at work wears its category's colour, the rest are muted;
        // either comes up to full strength under the focus.
        let edge = if working {
            category_color(bonus)
        } else {
            MUTED
        };
        *border = BorderColor::all(if lit || working {
            edge
        } else {
            edge.with_alpha(edge.alpha() * DIMMED_EDGE_ALPHA)
        });
        *outline = focus_ring(if lit { Color::WHITE } else { Color::NONE });
        background.0 = if lit {
            CARD_HIGHLIGHT_BACKGROUND
        } else {
            CARD_BACKGROUND
        };
    }

    let focused = focus.focused(&data);
    for (kind, mut span, mut color) in &mut spans {
        let (content, tint) = match (focused, kind) {
            (None, _) => (String::new(), Color::WHITE),
            (Some(bonus), DetailSpan::Heading) => (
                format!(
                    "{}   {}\n",
                    bonus.category.to_uppercase(),
                    bonus.rarity.to_string().to_uppercase()
                ),
                category_color(bonus),
            ),
            (Some(bonus), DetailSpan::Name) => (
                format!("{}\n", bonus.name),
                Color::from(data.bonuses.rarities[bonus.rarity].color),
            ),
            (Some(bonus), DetailSpan::Description) => {
                (bonus.description.clone(), DESCRIPTION_COLOR)
            }
            (Some(bonus), DetailSpan::State) => (
                state_of(bonus, &at_work, &session).0,
                Color::from(data.bonuses.synergy.color),
            ),
        };
        if span.0 != content {
            span.0 = content;
        }
        color.0 = tint;
    }
    for mut label in &mut key_labels {
        let key = controls.primary_label(GameAction::ShowBuild);
        if label.0 != key {
            label.0 = key;
        }
    }
}
