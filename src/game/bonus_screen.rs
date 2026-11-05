use super::*;
use crate::game::ui_style::{
    ACTIVE_BORDER, CARD_BACKGROUND, CARD_HIGHLIGHT_BACKGROUND, DESCRIPTION_COLOR, MUTED, key_badge,
    overlay_root, title,
};
use bevy::text::FontSize;

#[derive(Component)]
pub(super) struct BonusScreenRoot;

/// A selectable box on the bonus screen, also used as the keyboard selection.
#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum BonusChoice {
    Offer(usize),
    Reroll,
}

impl BonusChoice {
    pub(super) fn previous(self) -> Self {
        match self {
            Self::Offer(index) => Self::Offer(index.saturating_sub(1)),
            Self::Reroll => Self::Offer(1),
        }
    }

    pub(super) fn next(self, can_reroll: bool) -> Self {
        match self {
            Self::Offer(0) => Self::Offer(1),
            Self::Offer(_) | Self::Reroll if can_reroll => Self::Reroll,
            choice => choice,
        }
    }
}

#[derive(Component, Clone, Copy)]
pub(super) enum BonusLabel {
    Key(GameAction),
    Reroll,
}

#[derive(Component)]
pub(super) struct BonusSpan {
    index: usize,
    kind: BonusSpanKind,
}

#[derive(Clone, Copy)]
enum BonusSpanKind {
    Category,
    Name,
    Description,
}

pub(super) fn spawn_bonus_screen(mut commands: Commands) {
    commands
        .spawn((overlay_root(), BonusScreenRoot))
        .with_children(|parent| {
            parent
                .spawn(Node {
                    width: percent(78),
                    max_width: px(760),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Stretch,
                    row_gap: px(16),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn(title("CHOOSE A BONUS"));
                    for (index, action) in
                        [GameAction::ChooseFirstBonus, GameAction::ChooseSecondBonus]
                            .into_iter()
                            .enumerate()
                    {
                        parent
                            .spawn((
                                Node {
                                    flex_direction: FlexDirection::Row,
                                    align_items: AlignItems::Center,
                                    column_gap: px(20),
                                    padding: UiRect::all(px(18)),
                                    border: UiRect::all(px(2)),
                                    border_radius: BorderRadius::all(px(8)),
                                    ..default()
                                },
                                BackgroundColor(CARD_BACKGROUND),
                                BorderColor::all(MUTED),
                                BonusChoice::Offer(index),
                            ))
                            .with_children(|parent| {
                                parent.spawn(key_badge(56.0)).with_child((
                                    Text::default(),
                                    TextFont {
                                        font_size: FontSize::Px(30.0),
                                        ..default()
                                    },
                                    TextColor(Color::WHITE),
                                    BonusLabel::Key(action),
                                    Pickable::IGNORE,
                                ));
                                parent
                                    .spawn((
                                        Text::default(),
                                        TextFont {
                                            font_size: FontSize::Px(18.0),
                                            ..default()
                                        },
                                        TextLayout::justify(Justify::Left),
                                        Node {
                                            flex_grow: 1.0,
                                            min_width: px(0),
                                            ..default()
                                        },
                                        Pickable::IGNORE,
                                    ))
                                    .with_children(|parent| {
                                        for (kind, font_size) in [
                                            (BonusSpanKind::Category, 16.0),
                                            (BonusSpanKind::Name, 26.0),
                                            (BonusSpanKind::Description, 19.0),
                                        ] {
                                            parent.spawn((
                                                TextSpan::default(),
                                                TextFont {
                                                    font_size: FontSize::Px(font_size),
                                                    ..default()
                                                },
                                                TextColor(Color::WHITE),
                                                BonusSpan { index, kind },
                                            ));
                                        }
                                    });
                            });
                    }
                    parent
                        .spawn((
                            Node {
                                align_self: AlignSelf::Center,
                                flex_direction: FlexDirection::Row,
                                align_items: AlignItems::Center,
                                column_gap: px(14),
                                margin: UiRect::top(px(12)),
                                padding: UiRect::axes(px(16), px(10)),
                                border: UiRect::all(px(2)),
                                border_radius: BorderRadius::all(px(8)),
                                ..default()
                            },
                            BackgroundColor(CARD_BACKGROUND),
                            BorderColor::all(ACTIVE_BORDER),
                            BonusChoice::Reroll,
                        ))
                        .with_children(|parent| {
                            parent.spawn(key_badge(40.0)).with_child((
                                Text::default(),
                                TextFont {
                                    font_size: FontSize::Px(22.0),
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                                BonusLabel::Key(GameAction::RerollBonus),
                                Pickable::IGNORE,
                            ));
                            parent.spawn((
                                Text::default(),
                                TextFont {
                                    font_size: FontSize::Px(20.0),
                                    ..default()
                                },
                                TextLayout::justify(Justify::Left),
                                TextColor(Color::WHITE),
                                BonusLabel::Reroll,
                                Pickable::IGNORE,
                            ));
                        });
                });
        });
}

#[derive(SystemParam)]
pub(super) struct BonusScreenResources<'w> {
    screen: Res<'w, State<GameScreen>>,
    session: Res<'w, RunSession>,
    data: Res<'w, GameData>,
    controls: Res<'w, ControlBindings>,
}

pub(super) fn update_bonus_screen(
    resources: BonusScreenResources,
    mut roots: Query<&mut Visibility, With<BonusScreenRoot>>,
    mut boxes: Query<(&BonusChoice, &mut BorderColor, &mut BackgroundColor)>,
    mut labels: Query<(&BonusLabel, &mut Text, &mut TextColor)>,
    mut spans: Query<(&BonusSpan, &mut TextSpan, &mut TextColor), Without<BonusLabel>>,
) {
    let BonusScreenResources {
        screen,
        session,
        data,
        controls,
    } = resources;
    let visible = *screen.get() == GameScreen::Bonus;
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

    let rerolling = session.bonus_reroll_animation_remaining_s > 0.0;
    let can_reroll = !rerolling && flow::reroll_available(&session, &data);
    let offer = |index: usize| {
        session
            .offered_bonuses
            .get(index)
            .and_then(|id| data.bonuses.bonuses.iter().find(|bonus| bonus.id == *id))
            .filter(|_| !rerolling)
    };
    let category_color = |index: usize| {
        offer(index)
            .and_then(|bonus| data.bonuses.categories.get(&bonus.category))
            .map_or(MUTED, |category| Color::from(category.color))
    };

    for (choice, mut border, mut background) in &mut boxes {
        let enabled = matches!(choice, BonusChoice::Offer(_)) || can_reroll;
        *border = BorderColor::all(match choice {
            BonusChoice::Offer(index) => category_color(*index),
            BonusChoice::Reroll if can_reroll => ACTIVE_BORDER,
            BonusChoice::Reroll => MUTED,
        });
        background.0 = if enabled && session.selected_choice == *choice {
            CARD_HIGHLIGHT_BACKGROUND
        } else {
            CARD_BACKGROUND
        };
    }
    for (label, mut text, mut color) in &mut labels {
        let (content, label_color) = match label {
            BonusLabel::Key(GameAction::RerollBonus) => (
                controls.primary_label(GameAction::RerollBonus),
                if can_reroll { Color::WHITE } else { MUTED },
            ),
            BonusLabel::Key(action) => (controls.primary_label(*action), Color::WHITE),
            BonusLabel::Reroll if rerolling => {
                let dots = (session.bonus_reroll_animation_remaining_s * 12.0) as usize % 4;
                (format!("REROLLING{}", ".".repeat(dots)), Color::WHITE)
            }
            BonusLabel::Reroll if can_reroll => (
                format!("REROLL   @ {} remaining", session.rerolls),
                Color::WHITE,
            ),
            BonusLabel::Reroll => (
                format!("REROLL UNAVAILABLE   @ {} remaining", session.rerolls),
                MUTED,
            ),
        };
        text.0 = content;
        color.0 = label_color;
    }
    for (span, mut text, mut color) in &mut spans {
        let (content, span_color) = match (offer(span.index), span.kind) {
            (None, BonusSpanKind::Name) if rerolling => ("...".to_owned(), MUTED),
            (None, _) => (String::new(), Color::WHITE),
            (Some(bonus), BonusSpanKind::Category) => {
                let icon = data
                    .bonuses
                    .categories
                    .get(&bonus.category)
                    .map(|category| category.icon.to_string())
                    .unwrap_or_default();
                (
                    format!("{icon} {}\n", bonus.category.to_uppercase()),
                    category_color(span.index),
                )
            }
            (Some(bonus), BonusSpanKind::Name) => (format!("{}\n", bonus.name), Color::WHITE),
            (Some(bonus), BonusSpanKind::Description) => {
                (bonus.description.clone(), DESCRIPTION_COLOR)
            }
        };
        text.0 = content;
        color.0 = span_color;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arrow_navigation_walks_offers_then_reroll_only_when_available() {
        assert_eq!(BonusChoice::Offer(0).next(true), BonusChoice::Offer(1));
        assert_eq!(BonusChoice::Offer(1).next(true), BonusChoice::Reroll);
        assert_eq!(BonusChoice::Offer(1).next(false), BonusChoice::Offer(1));
        assert_eq!(BonusChoice::Reroll.next(true), BonusChoice::Reroll);
        assert_eq!(BonusChoice::Reroll.previous(), BonusChoice::Offer(1));
        assert_eq!(BonusChoice::Offer(1).previous(), BonusChoice::Offer(0));
        assert_eq!(BonusChoice::Offer(0).previous(), BonusChoice::Offer(0));
    }
}
