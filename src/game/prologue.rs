//! The words the city is entered on, between the opening room and room 1: the
//! flood rises over the room the run is leaving, the lines of `prologue.lines`
//! are typed out over it with the typing heard under them, and the water
//! drains away again over the slide into room 1.
//!
//! Any key hurries it along: the first press finishes the typing, the next
//! leaves for the city.

use super::*;
use crate::game::{
    menus::{MenuAction, MenuButton},
    ui_style::{CARD_BACKGROUND, MUTED, key_badge},
};

use bevy::text::FontSize;

/// How far the prologue has come: how much of its text is on screen, how
/// opaque the water standing over the room is, and how long the finished text
/// has been held.
#[derive(Resource, Default)]
pub(super) struct Prologue {
    typed_characters: f32,
    /// 0 while the room is in the clear, 1 once the water covers it.
    pub(crate) veil: f32,
}

impl Prologue {
    /// The text as it stands. Every character keeps its place from the first
    /// frame, written or not: the words are drawn in a monospace font, so the
    /// block never shifts under itself as it fills in.
    fn written(&self, lines: &[String]) -> String {
        let typed = self.typed_characters as usize;
        lines
            .join("\n\n")
            .chars()
            .enumerate()
            .map(|(index, character)| {
                if index < typed || character == '\n' {
                    character
                } else {
                    ' '
                }
            })
            .collect()
    }

    fn fully_typed(&self, lines: &[String]) -> bool {
        self.typed_characters as usize >= lines.join("\n\n").chars().count()
    }

    /// Answers the Enter button: the first press finishes writing the words,
    /// and once they stand, the next one says to walk in.
    pub(super) fn confirm(&mut self, lines: &[String]) -> bool {
        if self.fully_typed(lines) {
            return true;
        }
        self.typed_characters = lines.join("\n\n").chars().count() as f32;
        false
    }
}

/// The layer the prologue's words are written on.
#[derive(Component)]
pub(super) struct PrologueLayer;

/// The words themselves.
#[derive(Component)]
pub(super) struct PrologueText;

/// The card the words are written on.
#[derive(Component)]
pub(super) struct PrologueCard;

/// The button that walks into the city, shown once the words stand.
#[derive(Component)]
pub(super) struct PrologueButton;

pub(super) fn spawn_prologue_text(mut commands: Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                right: px(0),
                top: px(0),
                bottom: px(0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: px(16),
                ..default()
            },
            // A dark pane over the water, so the words are read rather than
            // searched for; it comes up and goes with the water itself.
            BackgroundColor(Color::srgba(0.02, 0.01, 0.04, 0.0)),
            GlobalZIndex(1),
            Visibility::Hidden,
            Pickable::IGNORE,
            PrologueLayer,
        ))
        .with_children(|layer| {
            // The words sit on a card of their own, as the menus do, so they
            // are read against something still rather than against the water.
            layer
                .spawn((
                    Node {
                        width: percent(78),
                        max_width: px(PANEL_WIDTH_PX),
                        padding: UiRect::axes(px(24), px(20)),
                        border: UiRect::all(px(2)),
                        border_radius: BorderRadius::all(px(8)),
                        align_items: AlignItems::FlexStart,
                        ..default()
                    },
                    // Darker than a menu card: this one is read rather than
                    // glanced at, and the water behind it is lit.
                    BackgroundColor(WORDS_BACKGROUND),
                    BorderColor::all(MUTED),
                    Pickable::IGNORE,
                    PrologueCard,
                ))
                .with_children(|card| {
                    card.spawn((
                        Text::new(String::new()),
                        TextFont {
                            font_size: FontSize::Px(PROLOGUE_FONT_PX),
                            ..default()
                        },
                        // Written out from the left, the way they are read.
                        TextLayout::justify(Justify::Left),
                        TextColor(Color::srgb(0.88, 0.9, 0.96)),
                        Pickable::IGNORE,
                        PrologueText,
                    ));
                });
            // The way in is a button like every other in the game: Enter
            // presses it, and so does a click.
            layer
                .spawn((
                    Node {
                        width: percent(78),
                        max_width: px(PANEL_WIDTH_PX),
                        min_height: px(72),
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        column_gap: px(16),
                        padding: UiRect::axes(px(16), px(12)),
                        border: UiRect::all(px(2)),
                        border_radius: BorderRadius::all(px(8)),
                        ..default()
                    },
                    BackgroundColor(CARD_BACKGROUND),
                    BorderColor::all(MUTED),
                    Visibility::Hidden,
                    // The row is what the mouse points at; everything in it is
                    // left alone.
                    MenuButton {
                        action: MenuAction::EnterTheCity,
                        index: 0,
                    },
                    PrologueButton,
                ))
                .with_children(|row| {
                    row.spawn(key_badge(44.0)).with_child((
                        Text::new("ENTER"),
                        TextFont {
                            font_size: FontSize::Px(20.0),
                            ..default()
                        },
                        TextColor(Color::WHITE),
                        Pickable::IGNORE,
                    ));
                    row.spawn((
                        Text::new("Enter Afterward"),
                        TextFont {
                            font_size: FontSize::Px(22.0),
                            ..default()
                        },
                        TextColor(Color::WHITE),
                        Pickable::IGNORE,
                    ));
                });
        });
}

pub(super) fn start_prologue(
    mut commands: Commands,
    bank: Res<audio::SoundBank>,
    settings: Res<AudioSettings>,
    mut prologue: ResMut<Prologue>,
) {
    prologue.typed_characters = 0.0;
    audio::start_typing(&mut commands, &bank, &settings);
}

/// The card the words are read off, which rises with the water under it.
type PrologueCards<'w, 's> = Query<
    'w,
    's,
    (&'static mut BackgroundColor, &'static mut BorderColor),
    (With<PrologueCard>, Without<PrologueLayer>),
>;

#[derive(SystemParam)]
pub(super) struct PrologueResources<'w> {
    time: Res<'w, Time>,
    data: Res<'w, GameData>,
    prologue: ResMut<'w, Prologue>,
}

/// Raises the water and writes the lines out under it. Where it goes next is
/// the player's to say: the Enter button waits under the words, and
/// `screens::handle_screen_input` carries it out.
pub(super) fn update_prologue(
    mut commands: Commands,
    resources: PrologueResources,
    typing: Query<Entity, With<audio::TypingSound>>,
    mut texts: Query<(&mut Text, &mut TextColor), With<PrologueText>>,
    mut panes: Query<&mut BackgroundColor, With<PrologueLayer>>,
    mut cards: PrologueCards,
    mut button: Query<&mut Visibility, With<PrologueButton>>,
) {
    let PrologueResources {
        time,
        data,
        mut prologue,
    } = resources;
    let config = &data.gameplay.prologue;
    let delta_s = time.delta_secs().min(0.05);
    prologue.veil = move_towards(prologue.veil, 1.0, delta_s / config.veil_fade_s);
    // The words wait for the water to stand over the room.
    if prologue.veil >= 1.0 {
        prologue.typed_characters += delta_s * config.characters_per_s;
    }
    let typed = prologue.fully_typed(&config.lines);
    if typed {
        // The typing stops with the last character, not with the screen.
        for entity in &typing {
            commands.entity(entity).despawn();
        }
    }

    let written = prologue.written(&config.lines);
    for (mut text, mut color) in &mut texts {
        text.0.clone_from(&written);
        color.0 = color.0.with_alpha(prologue.veil);
    }
    for mut pane in &mut panes {
        pane.0 = pane.0.with_alpha(PANE_OPACITY * prologue.veil);
    }
    // The card rises with the water rather than standing there before it.
    for (mut background, mut border) in &mut cards {
        background.0 = WORDS_BACKGROUND.with_alpha(WORDS_BACKGROUND.alpha() * prologue.veil);
        *border = BorderColor::all(MUTED.with_alpha(MUTED.alpha() * prologue.veil));
    }
    for mut visibility in &mut button {
        // The way in is offered once there is nothing left to read. It is
        // shown by inheriting the layer's visibility rather than by being
        // visible of its own: `Visibility::Visible` would stand even once the
        // prologue layer around it is hidden, and follow the player into the
        // city.
        *visibility = if typed {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
}

/// Drains the water away over the slide into the city, so the words give onto
/// the street they were about.
pub(super) fn drain_the_veil(time: Res<Time>, data: Res<GameData>, mut prologue: ResMut<Prologue>) {
    let fade_s = data.gameplay.prologue.veil_fade_s;
    prologue.veil = move_towards(prologue.veil, 0.0, time.delta_secs().min(0.05) / fade_s);
}

/// Shows the prologue's words while it is up, and takes them away with it.
pub(super) fn show_prologue_text(
    screen: Res<State<GameScreen>>,
    mut roots: Query<&mut Visibility, With<PrologueLayer>>,
) {
    for mut visibility in &mut roots {
        *visibility = if *screen.get() == GameScreen::Prologue {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

const PROLOGUE_FONT_PX: f32 = 20.0;
/// How wide the card of words, and the button under it, are drawn.
const PANEL_WIDTH_PX: u32 = 680;
/// How far the pane over the water is drawn down once the words are up.
const PANE_OPACITY: f32 = 0.5;
/// The card the words are read off: nearly solid, so the lit water behind it
/// never crosses a line of text.
const WORDS_BACKGROUND: Color = Color::srgba(0.02, 0.025, 0.05, 0.93);
