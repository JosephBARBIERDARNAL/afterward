//! The build window: every bonus the run owns, and what each is doing, over
//! the paused room (`GameScreen::Build`, opened and closed with
//! `GameAction::ShowBuild`). The room stays in sight behind it, dimmed but not
//! blurred. Each bonus is a chip named in the colour of the rarity it was
//! taken at, and edged in it while it is at work; the one under the focus
//! (pointed at, or walked to with the arrows) is described in full below the
//! chips, and its words are explained at the bottom right like an offer's
//! (`glossary.rs`). It opens from the bonus screen too, so a choice is made
//! knowing what the build already does, and closing it then goes back to the
//! offers.
//!
//! In the test room it lists every bonus instead, the ones owned lit, and
//! turns them on and off (`toggle_test_room_bonuses`), so a bonus can be
//! tried alone or with others without earning it.

use super::*;
use crate::game::{
    bonus_text::Description,
    tells::{Line, lines},
    ui_style::{
        CARD_BACKGROUND, CARD_HIGHLIGHT_BACKGROUND, DIMMED_EDGE_ALPHA, MUTED, focus_ring,
        key_badge, overlay_root, title,
    },
};
use bevy::text::FontSize;

/// The owned bonuses in the order the window lays them out (the order they
/// were taken in), and which of them the focus is on. Whoever opens the
/// window sets `index` (the bonus it opens on) and `over_the_offers`.
#[derive(Resource, Default)]
pub(super) struct BuildFocus {
    pub(super) order: Vec<BonusPick>,
    pub(super) index: usize,
    /// Opened from the bonus screen, which closing it goes back to, rather
    /// than from the fight.
    pub(super) over_the_offers: bool,
}

impl BuildFocus {
    /// The screen the window was opened from, and closes back to.
    pub(super) fn opened_from(&self) -> GameScreen {
        if self.over_the_offers {
            GameScreen::Bonus
        } else {
            GameScreen::Playing
        }
    }
}

impl BuildFocus {
    /// The bonus the focus stands on, and the rarity it was taken at, if the
    /// run owns any.
    pub(super) fn focused<'a>(&self, data: &'a GameData) -> Option<(&'a BonusConfig, BonusRarity)> {
        let pick = self.order.get(self.index)?;
        Some((data.bonuses.bonus(&pick.id)?, pick.rarity))
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

/// The name on a chip, by its place in `BuildFocus::order`: coloured each
/// frame, since the test room turns its bonus on and off and changes its
/// rarity.
#[derive(Component)]
pub(super) struct ChipName(usize);

#[derive(Component, Clone, Copy)]
pub(super) enum DetailSpan {
    Heading,
    Name,
    State,
}

/// The description of the bonus under the focus.
#[derive(Component)]
pub(super) struct DetailDescription;

const DESCRIPTION_FONT_PX: f32 = 16.0;

/// The key that closes the window, on its badge.
#[derive(Component)]
pub(super) struct BuildKeyLabel;

/// What closing the window goes back to, beside that key.
#[derive(Component)]
pub(super) struct BuildCloseLabel;

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
                        Node {
                            min_height: px(96),
                            flex_direction: FlexDirection::Column,
                            row_gap: px(4),
                            padding: UiRect::top(px(10)),
                            border: UiRect::top(px(1)),
                            ..default()
                        },
                        BorderColor::all(MUTED),
                        Pickable::IGNORE,
                    ))
                    .with_children(|detail| {
                        let text = |font_px: f32| {
                            (
                                Text::default(),
                                TextFont {
                                    font_size: FontSize::Px(font_px),
                                    ..default()
                                },
                                TextLayout::justify(Justify::Left),
                                Pickable::IGNORE,
                            )
                        };
                        let span = |kind: DetailSpan, font_px: f32, role: FontRole| {
                            (
                                TextSpan::default(),
                                TextFont {
                                    font_size: FontSize::Px(font_px),
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                                kind,
                                role,
                            )
                        };
                        detail.spawn(text(16.0)).with_children(|heading| {
                            heading.spawn(span(DetailSpan::Heading, 14.0, FontRole::Regular));
                            heading.spawn(span(DetailSpan::Name, 26.0, FontRole::Display));
                        });
                        detail.spawn((
                            text(DESCRIPTION_FONT_PX),
                            Description {
                                font_px: DESCRIPTION_FONT_PX,
                                ..default()
                            },
                            DetailDescription,
                        ));
                        detail.spawn(text(15.0)).with_child(span(
                            DetailSpan::State,
                            15.0,
                            FontRole::BoldItalic,
                        ));
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
                            Text::default(),
                            BuildCloseLabel,
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
/// one it was opened on.
pub(super) fn open_build_window(
    mut commands: Commands,
    data: Res<GameData>,
    session: Res<RunSession>,
    mut focus: ResMut<BuildFocus>,
    grids: Query<Entity, With<BuildGrid>>,
) {
    focus.order = if session.in_the_test_room() {
        // Every bonus, at the rarity it is owned at, else at its lowest.
        data.bonuses
            .bonuses
            .iter()
            .filter_map(|bonus| {
                session
                    .owned_bonuses
                    .iter()
                    .find(|owned| owned.id == bonus.id)
                    .cloned()
                    .or_else(|| {
                        let rarity = bonus.rarities.keys().next()?;
                        Some(BonusPick::new(&bonus.id, *rarity))
                    })
            })
            .collect()
    } else {
        session
            .owned(&data)
            .map(|(bonus, rarity)| BonusPick::new(&bonus.id, rarity))
            .collect()
    };
    focus.index = focus.index.min(focus.order.len().saturating_sub(1));
    let Ok(grid) = grids.single() else {
        return;
    };
    commands.entity(grid).despawn_children();
    commands.entity(grid).with_children(|grid| {
        if focus.order.is_empty() {
            grid.spawn((
                Text::new("No bonus yet: clear a room to earn one."),
                TextFont {
                    font_size: FontSize::Px(16.0),
                    ..default()
                },
                TextColor(MUTED),
            ));
        }
        for (index, pick) in focus.order.iter().enumerate() {
            let Some(bonus) = data.bonuses.bonus(&pick.id) else {
                continue;
            };
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
                TextColor(MUTED),
                FontRole::Display,
                ChipName(index),
                Pickable::IGNORE,
            ));
        }
    });
}

/// Walks the focus with the arrows, or puts it where the pointer is. In the
/// test room only Left and Right walk it: Up and Down change the rarity.
pub(super) fn navigate_build_window(
    keys: Res<ButtonInput<KeyCode>>,
    controls: Res<ControlBindings>,
    session: Res<RunSession>,
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
        .skip(usize::from(session.in_the_test_room()))
        .any(|action| controls.just_pressed(action, &keys))
    {
        focus.index = (before + count - 1) % count;
    } else if [GameAction::NavigateDown, GameAction::MoveRight]
        .into_iter()
        .skip(usize::from(session.in_the_test_room()))
        .any(|action| controls.just_pressed(action, &keys))
    {
        focus.index = (before + 1) % count;
    }
    if focus.index != before {
        sounds.push(SoundCue::UiMove);
    }
}

/// What presses a chip of the test room's build window: the keys, and a
/// click on it.
#[derive(SystemParam)]
pub(super) struct ChipInput<'w, 's> {
    keys: Res<'w, ButtonInput<KeyCode>>,
    controls: Res<'w, ControlBindings>,
    pointer_clicks: MessageReader<'w, 's, PointerClick>,
    chips: Query<'w, 's, &'static BuildChip>,
}

/// The test room's build window turns bonuses on and off: a click on a
/// chip, or Confirm or Interact on the focused one, toggles it; Up and Down
/// change the rarity it is (or will be) owned at; Backspace gives every bonus
/// back.
pub(super) fn toggle_test_room_bonuses(
    input: ChipInput,
    data: Res<GameData>,
    mut session: ResMut<RunSession>,
    mut focus: ResMut<BuildFocus>,
    mut sounds: ResMut<SoundQueue>,
) {
    let ChipInput {
        keys,
        controls,
        mut pointer_clicks,
        chips,
    } = input;
    let clicked = pointer_clicks
        .read()
        .filter(|click| click.button == PointerButton::Primary)
        .filter_map(|click| chips.get(click.entity).ok())
        .last();
    if !session.in_the_test_room() {
        return;
    }
    if keys.just_pressed(KeyCode::Backspace) {
        let owned = session
            .owned_bonuses
            .iter()
            .map(|owned| owned.id.clone())
            .collect::<Vec<_>>();
        for id in owned {
            session.give_up(&data, &id);
        }
        sounds.push(SoundCue::UiCancel);
        return;
    }
    if let Some(chip) = clicked {
        focus.index = chip.0;
    }
    let index = focus.index;
    let Some(pick) = focus.order.get(index).cloned() else {
        return;
    };
    let step = if controls.just_pressed(GameAction::NavigateUp, &keys) {
        1
    } else if controls.just_pressed(GameAction::NavigateDown, &keys) {
        -1
    } else {
        0
    };
    if step != 0 {
        let Some(bonus) = data.bonuses.bonus(&pick.id) else {
            return;
        };
        let rarities = bonus.rarities.keys().copied().collect::<Vec<_>>();
        let at = rarities
            .iter()
            .position(|rarity| *rarity == pick.rarity)
            .unwrap_or_default() as isize;
        let rarity = rarities[(at + step).rem_euclid(rarities.len() as isize) as usize];
        if rarity == pick.rarity {
            return;
        }
        let repicked = BonusPick::new(&pick.id, rarity);
        if session.has_bonus(&pick.id) {
            session.give_up(&data, &pick.id);
            session.take(&data, repicked.clone());
        }
        focus.order[index] = repicked;
        sounds.push(SoundCue::UiMove);
        return;
    }
    let toggled = clicked.is_some()
        || [GameAction::Confirm, GameAction::Interact]
            .into_iter()
            .any(|action| controls.just_pressed(action, &keys));
    if !toggled {
        return;
    }
    if session.has_bonus(&pick.id) {
        session.give_up(&data, &pick.id);
        sounds.push(SoundCue::UiCancel);
    } else {
        session.take(&data, pick);
        sounds.push(SoundCue::UiConfirm);
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
    names: Query<'w, 's, (&'static ChipName, &'static mut TextColor), Without<DetailSpan>>,
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
        return (format!("IN FORCE: {}{left}", line.value), true);
    }
    let spent = match bonus.id.as_str() {
        "room_ward" => session.room_ward_used.then_some("SPENT FOR THIS ROOM"),
        "deflector" => session.deflector_used.then_some("SPENT FOR THIS ROOM"),
        "last_stand" => session.last_stand_used.then_some("SPENT FOR THIS RUN"),
        _ => None,
    };
    (spent.map(str::to_owned).unwrap_or_default(), false)
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
    mut descriptions: Query<&mut Description, With<DetailDescription>>,
    mut key_labels: Query<&mut Text, (With<BuildKeyLabel>, Without<BuildCloseLabel>)>,
    mut close_labels: Query<&mut Text, With<BuildCloseLabel>>,
) {
    let BuildWindowResources {
        screen,
        data,
        session,
        focus,
        controls,
        players,
        enemies,
        mut names,
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
    let rarity_color = |rarity: BonusRarity| Color::from(data.bonuses.rarities[rarity].color);

    for (chip, mut border, mut background, mut outline) in &mut chips {
        let Some((bonus, rarity)) = focus
            .order
            .get(chip.0)
            .and_then(|pick| Some((data.bonuses.bonus(&pick.id)?, pick.rarity)))
        else {
            continue;
        };
        let lit = chip.0 == focus.index;
        // In the test room, a bonus owned is one at work.
        let working = state_of(bonus, &at_work, &session).1
            || (session.in_the_test_room() && session.has_bonus(&bonus.id));
        // A bonus at work is edged in its rarity's colour, the rest are
        // muted; either comes up to full strength under the focus.
        let edge = if working { rarity_color(rarity) } else { MUTED };
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

    // A bonus the test room lists but the run does not own is muted.
    for (name, mut color) in &mut names {
        let Some(pick) = focus.order.get(name.0) else {
            continue;
        };
        color.0 = if session.has_bonus(&pick.id) {
            rarity_color(pick.rarity)
        } else {
            MUTED
        };
    }

    let focused = focus.focused(&data);
    for (kind, mut span, mut color) in &mut spans {
        let (content, tint) = match (focused, kind) {
            (None, _) => (String::new(), Color::WHITE),
            (Some((_, rarity)), DetailSpan::Heading) => (
                format!("{}\n", rarity.to_string().to_uppercase()),
                rarity_color(rarity),
            ),
            (Some((bonus, rarity)), DetailSpan::Name) => (bonus.name.clone(), rarity_color(rarity)),
            (Some((bonus, _)), DetailSpan::State) => (
                if session.in_the_test_room() {
                    let owned = if session.has_bonus(&bonus.id) {
                        "ON"
                    } else {
                        "OFF"
                    };
                    format!(
                        "{owned}: CLICK OR ENTER TO TOGGLE, UP/DOWN FOR THE RARITY, \
                         BACKSPACE TO TURN ALL OFF"
                    )
                } else {
                    state_of(bonus, &at_work, &session).0
                },
                Color::from(data.bonuses.synergy.color),
            ),
        };
        if span.0 != content {
            span.0 = content;
        }
        color.0 = tint;
    }
    let (text, except) = focused.map_or_else(Default::default, |(bonus, rarity)| {
        (bonus.describe(rarity), bonus.id.clone())
    });
    for mut description in &mut descriptions {
        description.set_if_neq(Description {
            text: text.clone(),
            except: except.clone(),
            font_px: DESCRIPTION_FONT_PX,
        });
    }
    for mut label in &mut key_labels {
        let key = controls.primary_label(GameAction::ShowBuild);
        if label.0 != key {
            label.0 = key;
        }
    }
    let close = if focus.over_the_offers {
        "BACK TO THE CHOICE"
    } else {
        "CLOSE"
    };
    for mut label in &mut close_labels {
        if label.0 != close {
            label.0 = close.to_owned();
        }
    }
}
