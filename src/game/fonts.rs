//! The game's fonts (`fonts:` of `config/gameplay.yaml`): the default one,
//! put in the place of Bevy's built-in font so every text that names no font
//! is written in it, and the others, which a text picks with a `FontRole`.

use super::*;
use bevy::ecs::{lifecycle::HookContext, world::DeferredWorld};
use std::fs;

/// The faces a text can ask for instead of the default one.
#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
#[component(on_insert = FontRole::dress)]
pub(crate) enum FontRole {
    Regular,
    Italic,
    BoldItalic,
    Display,
    DisplayItalic,
}

impl FontRole {
    /// Writes the role's face into the `TextFont` spawned with it, so the text
    /// is laid out in it from its first frame.
    fn dress(mut world: DeferredWorld, context: HookContext) {
        let Some(role) = world.get::<FontRole>(context.entity).copied() else {
            return;
        };
        let Some(font) = world
            .get_resource::<GameFonts>()
            .map(|fonts| fonts.0[role as usize].clone())
        else {
            return;
        };
        if let Some(mut text_font) = world.get_mut::<TextFont>(context.entity) {
            text_font.font = FontSource::Handle(font);
        }
    }
}

/// The face of every `FontRole`, in the order of its variants.
#[derive(Resource)]
pub(crate) struct GameFonts([Handle<Font>; 5]);

/// Reads every font file before anything is spawned. They are read here
/// rather than through the asset server because the default font has to be
/// swapped before the first text is laid out.
pub(super) fn load_the_game_fonts(
    mut commands: Commands,
    data: Res<GameData>,
    mut fonts: ResMut<Assets<Font>>,
) {
    let read = |file: &String| {
        let path = data.asset_root.join(file);
        fs::read(&path)
            .map(Font::from_bytes)
            .inspect_err(|error| {
                error!("the font {} was not loaded: {error}", path.display());
            })
            .ok()
    };
    let config = &data.gameplay.fonts;
    if let Some(font) = read(&config.default)
        && let Err(error) = fonts.insert(AssetId::default(), font)
    {
        error!("the default font was not set: {error}");
    }
    let mut role = |file: &String| read(file).map(|font| fonts.add(font)).unwrap_or_default();
    commands.insert_resource(GameFonts([
        role(&config.regular),
        role(&config.italic),
        role(&config.bold_italic),
        role(&config.display),
        role(&config.display_italic),
    ]));
}
