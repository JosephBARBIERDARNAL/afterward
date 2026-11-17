use super::*;
use crate::game::{
    bonus_text::Description,
    build_window::BuildFocus,
    ui_style::{
        ACTIVE_BORDER, CARD_BACKGROUND, CARD_HIGHLIGHT_BACKGROUND, DIMMED_EDGE_ALPHA, MUTED,
        focus_ring, key_badge, overlay_root, title,
    },
};
use bevy::{picking::hover::Hovered, text::FontSize};

#[derive(Component)]
pub(super) struct BonusScreenRoot;

/// A selectable box on the bonus screen, also used as the keyboard selection.
#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
#[require(Outline)]
pub(super) enum BonusChoice {
    Offer(usize),
    Reroll,
}

impl BonusChoice {
    /// The key that picks each offer, in the order the boxes stand: there are
    /// as many offers as there are keys.
    pub(super) const KEYS: [GameAction; 3] = [
        GameAction::ChooseFirstBonus,
        GameAction::ChooseSecondBonus,
        GameAction::ChooseThirdBonus,
    ];
    const LAST: usize = Self::KEYS.len() - 1;

    pub(super) fn previous(self) -> Self {
        match self {
            Self::Offer(index) => Self::Offer(index.saturating_sub(1)),
            Self::Reroll => Self::Offer(Self::LAST),
        }
    }

    pub(super) fn next(self, can_reroll: bool) -> Self {
        match self {
            Self::Offer(index) if index < Self::LAST => Self::Offer(index + 1),
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
    Rarity,
    Name,
    Synergy,
}

/// The description of the offer in that place.
#[derive(Component)]
pub(super) struct OfferDescription(usize);

/// Where the bonuses the run already owns are listed, under the offers, so
/// the choice is made knowing the build it joins.
#[derive(Component)]
pub(super) struct OwnedBonuses;

/// An owned bonus listed under the offers, by its place in the build: clicked,
/// it opens the build window on that bonus, described in full.
#[derive(Component)]
#[require(Outline, Hovered)]
pub(super) struct OwnedChip(pub(super) usize);

const DESCRIPTION_FONT_PX: f32 = 17.0;

pub(super) fn spawn_bonus_screen(mut commands: Commands) {
    commands
        .spawn((overlay_root(), BonusScreenRoot))
        .with_children(|parent| {
            parent
                // Narrow enough that the boxes explaining the focused offer
                // (`glossary.rs`) stand clear of it at the bottom right.
                .spawn(Node {
                    width: percent(48),
                    max_width: px(760),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Stretch,
                    row_gap: px(12),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn(title("CHOOSE A BONUS"));
                    for (index, action) in BonusChoice::KEYS.into_iter().enumerate() {
                        parent
                            .spawn((
                                Node {
                                    flex_direction: FlexDirection::Row,
                                    align_items: AlignItems::Center,
                                    column_gap: px(20),
                                    padding: UiRect::axes(px(18), px(12)),
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
                                        Node {
                                            flex_grow: 1.0,
                                            min_width: px(0),
                                            flex_direction: FlexDirection::Column,
                                            row_gap: px(4),
                                            ..default()
                                        },
                                        Pickable::IGNORE,
                                    ))
                                    .with_children(|parent| {
                                        // The name stands out in the display
                                        // face, the words under it are read in
                                        // the regular one (another bonus they
                                        // name in bold), and what the bonus
                                        // works with in bold italic.
                                        parent.spawn(offer_text(18.0)).with_children(|parent| {
                                            for (kind, font_size, role) in [
                                                (BonusSpanKind::Rarity, 15.0, FontRole::Regular),
                                                (BonusSpanKind::Name, 28.0, FontRole::Display),
                                            ] {
                                                parent.spawn(offer_span(
                                                    BonusSpan { index, kind },
                                                    font_size,
                                                    role,
                                                ));
                                            }
                                        });
                                        parent.spawn((
                                            offer_text(DESCRIPTION_FONT_PX),
                                            Description {
                                                font_px: DESCRIPTION_FONT_PX,
                                                ..default()
                                            },
                                            OfferDescription(index),
                                        ));
                                        parent.spawn(offer_text(15.0)).with_child(offer_span(
                                            BonusSpan {
                                                index,
                                                kind: BonusSpanKind::Synergy,
                                            },
                                            15.0,
                                            FontRole::BoldItalic,
                                        ));
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
                    // The build is only named here: its key (or a click on
                    // one of them) opens the build window over the offers,
                    // where each is described in full.
                    parent
                        .spawn(Node {
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            column_gap: px(10),
                            margin: UiRect::top(px(10)),
                            ..default()
                        })
                        .with_children(|row| {
                            row.spawn(key_badge(28.0)).with_child((
                                Text::default(),
                                TextFont {
                                    font_size: FontSize::Px(13.0),
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                                BonusLabel::Key(GameAction::ShowBuild),
                                Pickable::IGNORE,
                            ));
                            row.spawn((
                                Text::new("YOUR BONUSES"),
                                TextFont {
                                    font_size: FontSize::Px(14.0),
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                            ))
                            .with_child((
                                TextSpan::new("   SEE WHAT EACH ONE DOES"),
                                TextFont {
                                    font_size: FontSize::Px(14.0),
                                    ..default()
                                },
                                TextColor(MUTED),
                            ));
                        });
                    parent.spawn((
                        Node {
                            flex_direction: FlexDirection::Row,
                            flex_wrap: FlexWrap::Wrap,
                            column_gap: px(6),
                            row_gap: px(6),
                            ..default()
                        },
                        OwnedBonuses,
                    ));
                });
        });
}

/// A text of an offer's box, left-aligned, its words in its spans.
fn offer_text(font_px: f32) -> impl Bundle {
    (
        Text::default(),
        TextFont {
            font_size: FontSize::Px(font_px),
            ..default()
        },
        TextLayout::justify(Justify::Left),
        Pickable::IGNORE,
    )
}

fn offer_span(span: BonusSpan, font_px: f32, role: FontRole) -> impl Bundle {
    (
        TextSpan::default(),
        TextFont {
            font_size: FontSize::Px(font_px),
            ..default()
        },
        TextColor(Color::WHITE),
        span,
        role,
    )
}

/// Lists the bonuses the run owns as the bonus screen opens, each in the
/// colour of the rarity it was taken at, in the order they were taken.
pub(super) fn list_owned_bonuses(
    mut commands: Commands,
    data: Res<GameData>,
    session: Res<RunSession>,
    lists: Query<Entity, With<OwnedBonuses>>,
) {
    let Ok(list) = lists.single() else {
        return;
    };
    commands.entity(list).despawn_children();
    commands.entity(list).with_children(|list| {
        if session.owned_bonuses.is_empty() {
            list.spawn((
                Text::new("None yet."),
                TextFont {
                    font_size: FontSize::Px(14.0),
                    ..default()
                },
                TextColor(MUTED),
            ));
        }
        for (index, (bonus, rarity)) in session.owned(&data).enumerate() {
            list.spawn((
                Node {
                    padding: UiRect::axes(px(8), px(4)),
                    border: UiRect::all(px(1)),
                    border_radius: BorderRadius::all(px(5)),
                    ..default()
                },
                BackgroundColor(CARD_BACKGROUND),
                BorderColor::all(MUTED),
                OwnedChip(index),
            ))
            .with_child((
                Text::new(bonus.name.clone()),
                TextFont {
                    font_size: FontSize::Px(14.0),
                    ..default()
                },
                TextColor(Color::from(data.bonuses.rarities[rarity].color)),
                FontRole::Display,
                Pickable::IGNORE,
            ));
        }
    });
}

/// A click on an owned bonus opens the build window on it, over the offers;
/// pointing at one rings it, so it reads as something to click.
pub(super) fn open_owned_bonus(
    session: Res<RunSession>,
    mut focus: ResMut<BuildFocus>,
    mut next_screen: ResMut<NextState<GameScreen>>,
    mut sounds: ResMut<SoundQueue>,
    mut rings: Query<(&Hovered, &mut Outline), With<OwnedChip>>,
    mut pointer_clicks: MessageReader<PointerClick>,
    chips: Query<&OwnedChip>,
) {
    for (hovered, mut outline) in &mut rings {
        *outline = focus_ring(if hovered.get() {
            Color::WHITE
        } else {
            Color::NONE
        });
    }
    let Some(chip) = pointer_clicks
        .read()
        .filter(|click| click.button == PointerButton::Primary)
        .filter_map(|click| chips.get(click.entity).ok())
        .last()
    else {
        return;
    };
    if session.bonus_chosen_hold_remaining_s > 0.0 {
        return;
    }
    focus.index = chip.0;
    focus.over_the_offers = true;
    next_screen.set(GameScreen::Build);
    sounds.push(SoundCue::UiConfirm);
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
    mut boxes: Query<(
        &BonusChoice,
        &mut BorderColor,
        &mut BackgroundColor,
        &mut Outline,
        &mut UiTransform,
    )>,
    mut labels: Query<(&BonusLabel, &mut Text, &mut TextColor)>,
    mut spans: Query<(&BonusSpan, &mut TextSpan, &mut TextColor), Without<BonusLabel>>,
    mut descriptions: Query<(&OfferDescription, &mut Description)>,
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
            .filter(|_| !rerolling)
            .and_then(|offer| Some((data.bonuses.bonus(&offer.id)?, offer.rarity)))
    };
    // A bonus is painted by the rarity it was drawn at, and by nothing else.
    let rarity_color = |index: usize| {
        offer(index).map_or(MUTED, |(_, rarity)| {
            Color::from(data.bonuses.rarities[rarity].color)
        })
    };

    // A choice made flares while the screen holds: how far through the hold.
    let hold_s = data.gameplay.room_flow.bonus_chosen_hold_s;
    let picked = (session.bonus_chosen_hold_remaining_s > 0.0 && hold_s > 0.0)
        .then(|| 1.0 - session.bonus_chosen_hold_remaining_s / hold_s);
    let flare = &data.gameplay.effects.bonus_pick;
    for (choice, mut border, mut background, mut outline, mut transform) in &mut boxes {
        let enabled = matches!(choice, BonusChoice::Offer(_)) || can_reroll;
        let lit = enabled && session.selected_choice == *choice;
        // A box keeps its rarity's colour either way, and only comes up to
        // full strength, ringed, under the focus.
        let color = match choice {
            BonusChoice::Offer(index) => rarity_color(*index),
            BonusChoice::Reroll if can_reroll => ACTIVE_BORDER,
            BonusChoice::Reroll => MUTED,
        };
        *border = BorderColor::all(if lit {
            color
        } else {
            color.with_alpha(color.alpha() * DIMMED_EDGE_ALPHA)
        });
        *outline = focus_ring(if lit { color } else { Color::NONE });
        background.0 = if lit {
            CARD_HIGHLIGHT_BACKGROUND
        } else {
            CARD_BACKGROUND
        };
        transform.scale = Vec2::ONE;
        let Some(progress) = picked else {
            continue;
        };
        if matches!(choice, BonusChoice::Offer(index) if session.chosen_offers.contains(index)) {
            // The box taken swells and settles, washed in its rarity's
            // colour, while a ring of it spreads out from its edge and fades.
            let swell = (progress * std::f32::consts::PI).sin();
            transform.scale = Vec2::splat(1.0 + (flare.pulse_scale - 1.0) * swell);
            *border = BorderColor::all(color);
            background.0 = CARD_HIGHLIGHT_BACKGROUND.mix(&color, 0.3 * (1.0 - progress));
            *outline = Outline {
                width: px(1.0 + 3.0 * (1.0 - progress)),
                offset: px(flare.ring_px * progress),
                color: color.with_alpha(1.0 - progress),
            };
        } else {
            // The others step back.
            *border = BorderColor::all(
                color.with_alpha(color.alpha() * DIMMED_EDGE_ALPHA * (1.0 - progress)),
            );
            *outline = focus_ring(Color::NONE);
            background.0 = CARD_BACKGROUND;
        }
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
            (Some((_, rarity)), BonusSpanKind::Rarity) => (
                format!("{}\n", rarity.to_string().to_uppercase()),
                rarity_color(span.index),
            ),
            (Some((bonus, _)), BonusSpanKind::Name) => (bonus.name.clone(), Color::WHITE),
            // What the bonus works with in the build already owned, so the
            // right pick stands out from the other two.
            (Some((bonus, _)), BonusSpanKind::Synergy) => (
                session
                    .synergy(&data, bonus)
                    .map(|synergy| format!("+ {synergy}"))
                    .unwrap_or_default(),
                Color::from(data.bonuses.synergy.color),
            ),
        };
        text.0 = content;
        color.0 = span_color;
    }
    for (place, mut description) in &mut descriptions {
        let (text, except) = offer(place.0).map_or_else(Default::default, |(bonus, rarity)| {
            (bonus.describe(rarity), bonus.id.clone())
        });
        description.set_if_neq(Description {
            text,
            except,
            font_px: DESCRIPTION_FONT_PX,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arrow_navigation_walks_offers_then_reroll_only_when_available() {
        assert_eq!(BonusChoice::Offer(0).next(true), BonusChoice::Offer(1));
        assert_eq!(BonusChoice::Offer(1).next(true), BonusChoice::Offer(2));
        assert_eq!(BonusChoice::Offer(2).next(true), BonusChoice::Reroll);
        assert_eq!(BonusChoice::Offer(2).next(false), BonusChoice::Offer(2));
        assert_eq!(BonusChoice::Reroll.next(true), BonusChoice::Reroll);
        assert_eq!(BonusChoice::Reroll.previous(), BonusChoice::Offer(2));
        assert_eq!(BonusChoice::Offer(2).previous(), BonusChoice::Offer(1));
        assert_eq!(BonusChoice::Offer(0).previous(), BonusChoice::Offer(0));
    }
}
