//! The words the city is entered on, between the opening room and room 1: the
//! flood rises over the room the run is leaving, the lines of `prologue.lines`
//! are typed out over it with the typing heard under them, and the water
//! drains away again over the slide into room 1.
//!
//! Any key hurries it along: the first press finishes the typing, the next
//! leaves for the city.

use super::*;
use bevy::text::FontSize;

/// How far the prologue has come: how much of its text is on screen, how
/// opaque the water standing over the room is, and how long the finished text
/// has been held.
#[derive(Resource, Default)]
pub(super) struct Prologue {
    typed_characters: f32,
    /// 0 while the room is in the clear, 1 once the water covers it.
    pub(crate) veil: f32,
    held_s: f32,
}

impl Prologue {
    /// The text as it stands, with the characters yet to be typed left out.
    fn written(&self, lines: &[String]) -> String {
        let full = lines.join("\n\n");
        full.chars()
            .take(self.typed_characters as usize)
            .collect::<String>()
    }

    fn fully_typed(&self, lines: &[String]) -> bool {
        self.typed_characters as usize >= lines.join("\n\n").chars().count()
    }
}

/// The layer the prologue's words are written on.
#[derive(Component)]
pub(super) struct PrologueLayer;

/// The words themselves.
#[derive(Component)]
pub(super) struct PrologueText;

pub(super) fn spawn_prologue_text(mut commands: Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                right: px(0),
                top: px(0),
                bottom: px(0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
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
        .with_children(|parent| {
            parent.spawn((
                Text::new(String::new()),
                TextFont {
                    font_size: FontSize::Px(PROLOGUE_FONT_PX),
                    ..default()
                },
                TextLayout::justify(Justify::Center),
                TextColor(Color::srgb(0.88, 0.9, 0.96)),
                // The words stand over moving water, which is bright wherever
                // its web of light crosses.
                TextShadow {
                    offset: Vec2::splat(2.0),
                    color: Color::srgba(0.0, 0.0, 0.0, 0.7),
                },
                Node {
                    max_width: percent(66),
                    ..default()
                },
                PrologueText,
            ));
        });
}

pub(super) fn start_prologue(
    mut commands: Commands,
    bank: Res<audio::SoundBank>,
    settings: Res<AudioSettings>,
    mut prologue: ResMut<Prologue>,
) {
    prologue.typed_characters = 0.0;
    prologue.held_s = 0.0;
    audio::start_typing(&mut commands, &bank, &settings);
}

#[derive(SystemParam)]
pub(super) struct PrologueResources<'w> {
    time: Res<'w, Time>,
    data: Res<'w, GameData>,
    keys: Res<'w, ButtonInput<KeyCode>>,
    prologue: ResMut<'w, Prologue>,
    next_screen: ResMut<'w, NextState<GameScreen>>,
}

/// Raises the water, writes the lines out under it, and leaves for the city
/// once they have been held long enough — or once the player says so.
pub(super) fn update_prologue(
    mut commands: Commands,
    resources: PrologueResources,
    typing: Query<Entity, With<audio::TypingSound>>,
    mut texts: Query<(&mut Text, &mut TextColor), With<PrologueText>>,
    mut panes: Query<&mut BackgroundColor, With<PrologueLayer>>,
) {
    let PrologueResources {
        time,
        data,
        keys,
        mut prologue,
        mut next_screen,
    } = resources;
    let config = &data.gameplay.prologue;
    let delta_s = time.delta_secs().min(0.05);
    let was_typed = prologue.fully_typed(&config.lines);
    prologue.veil = move_towards(prologue.veil, 1.0, delta_s / config.veil_fade_s);
    // The words wait for the water to stand over the room.
    if prologue.veil >= 1.0 {
        prologue.typed_characters += delta_s * config.characters_per_s;
    }
    if keys.get_just_pressed().next().is_some() {
        if was_typed {
            prologue.held_s = config.hold_s;
        } else {
            prologue.typed_characters = f32::MAX;
        }
    }
    let typed = prologue.fully_typed(&config.lines);
    if typed {
        prologue.held_s += delta_s;
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

    if typed && prologue.held_s >= config.hold_s {
        next_screen.set(GameScreen::RoomLoading);
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

const PROLOGUE_FONT_PX: f32 = 26.0;
/// How far the pane over the water is drawn down once the words are up.
const PANE_OPACITY: f32 = 0.5;
