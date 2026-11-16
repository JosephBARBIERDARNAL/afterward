//! A bonus description written out on screen: its words in the regular
//! face, and every other bonus it names in bold and in white, so a bonus
//! another one builds on stands out of the sentence. The bonus screen, the
//! build window and the glossary's boxes all write theirs this way.

use super::*;
use crate::game::ui_style::DESCRIPTION_COLOR;
use bevy::text::FontSize;

/// A `Text` written as a bonus description: `text`, written for the bonus
/// `except` (never marked in its own description), at `font_px`. Its spans
/// are written again whenever it changes, so whoever fills it in should only
/// touch it when the words do.
#[derive(Component, Clone, Default, PartialEq)]
#[require(Text)]
pub(super) struct Description {
    pub(super) text: String,
    pub(super) except: String,
    pub(super) font_px: f32,
}

pub(super) fn write_descriptions(
    mut commands: Commands,
    data: Res<GameData>,
    texts: Query<(Entity, &Description), Changed<Description>>,
) {
    for (entity, description) in &texts {
        let text = description.text.as_str();
        let mut pieces = Vec::new();
        let mut at = 0;
        for (named, _) in data.bonuses.mentions(text, &description.except) {
            pieces.push((&text[at..named.start], false));
            pieces.push((&text[named.clone()], true));
            at = named.end;
        }
        pieces.push((&text[at..], false));
        commands.entity(entity).despawn_children();
        commands.entity(entity).with_children(|parent| {
            for (words, named) in pieces.into_iter().filter(|(words, _)| !words.is_empty()) {
                let mut span = parent.spawn((
                    TextSpan::new(words),
                    TextFont {
                        font_size: FontSize::Px(description.font_px),
                        ..default()
                    },
                    TextColor(if named {
                        Color::WHITE
                    } else {
                        DESCRIPTION_COLOR
                    }),
                ));
                // A bonus named is left in the default face, the bold one.
                if !named {
                    span.insert(FontRole::Regular);
                }
            }
        });
    }
}
