//! The boxes at the bottom right of the screen that explain the bonus under
//! the focus, on the bonus screen and in the build window: one box for each
//! word of `glossary:` (`config/bonuses.yaml`) its description leans on, and
//! one for each other bonus it names. `BonusesConfig::explain` decides what
//! they say.

use super::*;
use crate::{data::Explanation, game::ui_style::DESCRIPTION_COLOR};
use bevy::text::FontSize;

/// How many boxes can stand at once; a description leaning on more words
/// than this explains the first ones it uses.
const BOXES: usize = 4;
const BOX_BACKGROUND: Color = Color::srgba(0.03, 0.035, 0.07, 0.92);

/// The column the boxes stand in, bottom right.
#[derive(Component)]
pub(super) struct Glossary;

pub(super) fn spawn_glossary(mut commands: Commands) {
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            right: px(24),
            bottom: px(24),
            width: px(300),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::End,
            row_gap: px(8),
            ..default()
        },
        // Over the screen it explains, and never in the way of its pointer.
        GlobalZIndex(2),
        Pickable::IGNORE,
        Glossary,
    ));
}

/// Stands the boxes for the bonus under the focus, building them again only
/// when that bonus changes: a text laid out while hidden would be measured
/// empty, so the boxes are made fresh rather than shown and hidden.
pub(super) fn update_glossary(
    mut commands: Commands,
    screen: Res<State<GameScreen>>,
    session: Res<RunSession>,
    build: Res<build_window::BuildFocus>,
    data: Res<GameData>,
    mut shown: Local<Option<String>>,
    columns: Query<Entity, With<Glossary>>,
) {
    // The offer the focus stands on, while one is on show.
    let explained = match (*screen.get(), session.selected_choice) {
        (GameScreen::Bonus, bonus_screen::BonusChoice::Offer(index))
            if session.bonus_reroll_animation_remaining_s <= 0.0 =>
        {
            session
                .offered_bonuses
                .get(index)
                .and_then(|id| data.bonuses.bonuses.iter().find(|bonus| bonus.id == *id))
        }
        // The owned bonus the build window's focus stands on.
        (GameScreen::Build, _) => build.focused(&data),
        _ => None,
    };
    if explained.map(|bonus| &bonus.id) == shown.as_ref() {
        return;
    }
    *shown = explained.map(|bonus| bonus.id.clone());
    let Ok(column) = columns.single() else {
        return;
    };
    commands.entity(column).despawn_children();
    let Some(bonus) = explained else {
        return;
    };
    commands.entity(column).with_children(|column| {
        for explanation in data.bonuses.explain(bonus).into_iter().take(BOXES) {
            // A word in white, a bonus in its category's colour.
            let (name, definition, color) = match explanation {
                Explanation::Term(term) => (&term.term, &term.definition, Color::WHITE),
                Explanation::Bonus(named) => (
                    &named.name,
                    &named.description,
                    data.bonuses
                        .categories
                        .get(&named.category)
                        .map_or(Color::WHITE, |category| Color::from(category.color)),
                ),
            };
            column
                .spawn((
                    Node {
                        padding: UiRect::axes(px(12), px(8)),
                        border: UiRect::left(px(3)),
                        border_radius: BorderRadius::all(px(4)),
                        ..default()
                    },
                    BackgroundColor(BOX_BACKGROUND),
                    BorderColor::all(color),
                    Pickable::IGNORE,
                ))
                .with_children(|card| {
                    card.spawn((
                        Text::default(),
                        TextFont {
                            font_size: FontSize::Px(14.0),
                            ..default()
                        },
                        TextLayout::justify(Justify::Left),
                        Pickable::IGNORE,
                    ))
                    .with_children(|text| {
                        text.spawn((
                            TextSpan::new(format!("{name}\n")),
                            TextFont {
                                font_size: FontSize::Px(17.0),
                                ..default()
                            },
                            TextColor(color),
                            FontRole::Display,
                        ));
                        text.spawn((
                            TextSpan::new(definition.clone()),
                            TextFont {
                                font_size: FontSize::Px(14.0),
                                ..default()
                            },
                            TextColor(DESCRIPTION_COLOR),
                            FontRole::Regular,
                        ));
                    });
                });
        }
    });
}
