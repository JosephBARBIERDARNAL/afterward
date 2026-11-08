//! The look shared by the bonus screen and every menu: dark cards with a
//! thin border, key badges, and a large centered title.

use super::*;
use bevy::{
    asset::AssetPath,
    image::{ImageLoaderSettings, ImageSampler},
    text::FontSize,
};

// The cards are semi-transparent, so the water behind the main menu (and the
// blurred room behind the other menus) shows through them.
pub(super) const CARD_BACKGROUND: Color = Color::srgba(0.03, 0.035, 0.07, 0.7);
pub(super) const CARD_HIGHLIGHT_BACKGROUND: Color = Color::srgba(0.11, 0.12, 0.19, 0.8);
pub(super) const KEY_BACKGROUND: Color = Color::srgba(1.0, 1.0, 1.0, 0.1);
pub(super) const ACTIVE_BORDER: Color = Color::srgba(0.92, 0.94, 1.0, 0.85);
pub(super) const MUTED: Color = Color::srgba(0.6, 0.62, 0.7, 0.55);
pub(super) const DESCRIPTION_COLOR: Color = Color::srgb(0.84, 0.86, 0.92);
pub(super) const TITLE_FONT_PX: f32 = 32.0;

/// A full-screen layer that centers its content above the game.
pub(super) fn overlay_root() -> impl Bundle {
    (
        Node {
            position_type: PositionType::Absolute,
            left: px(0),
            right: px(0),
            top: px(0),
            bottom: px(0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        GlobalZIndex(1),
        Visibility::Hidden,
        Pickable::IGNORE,
    )
}

/// The screen's heading, centered above its cards.
pub(super) fn title(text: impl Into<String>) -> impl Bundle {
    (
        Text::new(text),
        TextFont {
            font_size: FontSize::Px(TITLE_FONT_PX),
            ..default()
        },
        TextLayout::justify(Justify::Center),
        TextColor(Color::WHITE),
        // The main menu's title stands over moving water, which is bright
        // where its web of light crosses.
        TextShadow {
            offset: Vec2::splat(2.0),
            color: Color::srgba(0.0, 0.0, 0.0, 0.7),
        },
        Node {
            margin: UiRect::bottom(px(8)),
            ..default()
        },
    )
}

/// A bordered box holding a key name. It is at least `size` square and
/// widens for long names such as `ESCAPE`.
pub(super) fn key_badge(size: f32) -> impl Bundle {
    (
        Node {
            min_width: px(size),
            height: px(size),
            flex_shrink: 0.0,
            padding: UiRect::horizontal(px(8)),
            border: UiRect::all(px(2)),
            border_radius: BorderRadius::all(px(6)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(KEY_BACKGROUND),
        BorderColor::all(ACTIVE_BORDER),
        Pickable::IGNORE,
    )
}

/// Loads a pixel-art image (drawn in Typst, see `art/ui/`) so it stays crisp
/// at whatever size it is drawn.
pub(super) fn load_pixel_art(
    asset_server: &AssetServer,
    path: impl Into<AssetPath<'static>>,
) -> Handle<Image> {
    asset_server
        .load_builder()
        .with_settings(|settings: &mut ImageLoaderSettings| {
            settings.sampler = ImageSampler::nearest();
        })
        .load(path)
}
