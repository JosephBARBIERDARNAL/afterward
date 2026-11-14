//! Terminals: used with Interact, a terminal types its `text` out in a
//! speech bubble over it, a box with a tail pointing down at it, as the
//! prologue types its words (same pace, same font, the same typing heard for
//! as long as the typing lasts). Interact again finishes the writing, and
//! once more closes the bubble; walking out of reach closes it too.

use super::*;
use crate::game::{interact::InReach, ui_style::MUTED};
use bevy::{
    sprite::Anchor,
    text::{FontSize, TextBounds, TextLayoutInfo},
};

/// The bubble's words, in world units.
const FONT_SIZE: f32 = 15.0;
/// How wide the words run before they wrap, in H.
const WRAP_H: f32 = 6.0;
/// Room between the words and the box's edge, and the edge's thickness.
const PADDING: f32 = 8.0;
const EDGE: f32 = 2.0;
/// How far the box keeps from the room's sides, in H.
const SIDE_MARGIN_H: f32 = 1.25;
/// How far above the terminal's top the box stands, in H: the tail spans it.
const GAP_H: f32 = 0.3;
/// The tail's steps, from the box down to its point: width and height.
const TAIL_STEPS: [Vec2; 3] = [
    Vec2::new(14.0, 4.0),
    Vec2::new(9.0, 4.0),
    Vec2::new(4.0, 4.0),
];
/// Over the room and its bodies, under the interact prompt.
const BUBBLE_Z: f32 = 9.0;
/// The prologue's colours: a dark box, a muted edge, pale words.
const BOX_FILL: Color = Color::srgba(0.02, 0.025, 0.05, 0.93);
const WORDS: Color = Color::srgb(0.88, 0.9, 0.96);

/// The terminal being read: its bubble, and how much of its text is out.
#[derive(Resource, Default)]
pub(crate) struct Reading(Option<Read>);

pub(crate) struct Read {
    terminal: Entity,
    bubble: Entity,
    text: String,
    typed_characters: f32,
}

impl Read {
    fn fully_typed(&self) -> bool {
        self.typed_characters as usize >= self.text.chars().count()
    }
}

/// The parts of a bubble.
#[derive(Component)]
pub(crate) enum BubblePart {
    /// The words: its first span is what is typed, its second the rest,
    /// written but unseen, so the box is the words' full size from the start
    /// and nothing rewraps as they come.
    Words,
    Edge,
    Fill,
}

/// A bubble's parts, with what is sized and placed of them.
type BubbleParts<'world, 'state> = Query<
    'world,
    'state,
    (
        &'static BubblePart,
        Option<&'static TextLayoutInfo>,
        &'static mut Transform,
        Option<&'static mut Sprite>,
    ),
>;

/// Opens the bubble of the terminal in reach when Interact goes down, hurries
/// its writing, or closes it.
pub(crate) fn read_terminals(
    mut commands: Commands,
    (keys, controls, in_reach): (
        Res<ButtonInput<KeyCode>>,
        Res<ControlBindings>,
        Res<InReach>,
    ),
    (bank, settings, data): (Res<audio::SoundBank>, Res<AudioSettings>, Res<GameData>),
    mut reading: ResMut<Reading>,
    terminals: Query<(&Device, &Transform)>,
    typing: Query<Entity, With<audio::TypingSound>>,
) {
    if !controls.just_pressed(GameAction::Interact, &keys) {
        return;
    }
    let Some((terminal, device, transform)) = in_reach
        .0
        .and_then(|entity| Some((entity, terminals.get(entity).ok()?)))
        .map(|(entity, (device, transform))| (entity, device, transform))
    else {
        return;
    };
    let DeviceKind::Terminal(config) = &device.config.kind else {
        return;
    };
    let stop_typing = |commands: &mut Commands| {
        for entity in &typing {
            commands.entity(entity).despawn();
        }
    };
    match reading.0.as_mut() {
        Some(read) if read.terminal == terminal && !read.fully_typed() => {
            read.typed_characters = read.text.chars().count() as f32;
            stop_typing(&mut commands);
            return;
        }
        Some(read) if read.terminal == terminal => {
            if let Ok(mut bubble) = commands.get_entity(read.bubble) {
                bubble.despawn();
            }
            reading.0 = None;
            return;
        }
        Some(read) => {
            if let Ok(mut bubble) = commands.get_entity(read.bubble) {
                bubble.despawn();
            }
            stop_typing(&mut commands);
        }
        None => {}
    }
    let top =
        transform.translation.y + device.config.kind.size_h(&data.devices).y / 2.0 * H_WORLD_UNITS;
    let bubble = spawn_bubble(
        &mut commands,
        Vec2::new(transform.translation.x, top + GAP_H * H_WORLD_UNITS),
        &config.text,
    );
    audio::start_typing(&mut commands, &bank, &settings);
    reading.0 = Some(Read {
        terminal,
        bubble,
        text: config.text.clone(),
        typed_characters: 0.0,
    });
}

/// Lays an empty bubble with its tail's point at `point`.
fn spawn_bubble(commands: &mut Commands, point: Vec2, text: &str) -> Entity {
    commands
        .spawn((
            Transform::from_translation(point.extend(BUBBLE_Z)),
            Visibility::default(),
            RoomEntity,
        ))
        .with_children(|bubble| {
            bubble.spawn((
                Sprite::from_color(MUTED, Vec2::ZERO),
                Transform::default(),
                BubblePart::Edge,
            ));
            bubble.spawn((
                Sprite::from_color(BOX_FILL, Vec2::ZERO),
                Transform::from_xyz(0.0, 0.0, 0.01),
                BubblePart::Fill,
            ));
            bubble
                .spawn((
                    Text2d::new(""),
                    TextFont {
                        font_size: FontSize::Px(FONT_SIZE),
                        ..default()
                    },
                    TextLayout::justify(Justify::Left),
                    TextBounds::new_horizontal(WRAP_H * H_WORLD_UNITS),
                    Anchor::CENTER,
                    Transform::from_xyz(0.0, 0.0, 0.02),
                    BubblePart::Words,
                ))
                .with_children(|words| {
                    words.spawn((TextSpan::new(""), TextColor(WORDS)));
                    words.spawn((TextSpan::new(text), TextColor(Color::NONE)));
                });
            // The tail: steps narrowing from the box down to the terminal,
            // each edged and filled as the box is.
            let mut below = 0.0;
            for step in TAIL_STEPS {
                below += step.y;
                let y = -below + step.y / 2.0 + EDGE;
                bubble.spawn((
                    Sprite::from_color(MUTED, step + Vec2::new(2.0 * EDGE, EDGE)),
                    Transform::from_xyz(0.0, y - EDGE / 2.0, 0.0),
                ));
                bubble.spawn((
                    Sprite::from_color(BOX_FILL, step),
                    Transform::from_xyz(0.0, y, 0.01),
                ));
            }
        })
        .id()
}

/// Writes the words out at the prologue's pace, stops the typing with the
/// last character, sizes the box around the words, and closes the bubble
/// once the player walks out of reach of its terminal.
pub(crate) fn type_terminal_text(
    mut commands: Commands,
    (time, data, in_reach, session): (Res<Time>, Res<GameData>, Res<InReach>, Res<RunSession>),
    mut reading: ResMut<Reading>,
    typing: Query<Entity, With<audio::TypingSound>>,
    (bubbles, children): (Query<&GlobalTransform>, Query<&Children>),
    mut parts: BubbleParts,
    mut spans: Query<&mut TextSpan>,
) {
    let Some(read) = reading.0.as_mut() else {
        return;
    };
    if in_reach.0 != Some(read.terminal) {
        // Gone with the room, or walked away from.
        if let Ok(mut bubble) = commands.get_entity(read.bubble) {
            bubble.despawn();
        }
        for entity in &typing {
            commands.entity(entity).despawn();
        }
        reading.0 = None;
        return;
    }
    let was_typing = !read.fully_typed();
    read.typed_characters += time.delta_secs().min(0.05) * data.gameplay.prologue.characters_per_s;
    if was_typing && read.fully_typed() {
        // The typing stops with the last character.
        for entity in &typing {
            commands.entity(entity).despawn();
        }
    }
    let typed = read.typed_characters as usize;
    let shown = read.text.chars().take(typed).collect::<String>();
    let unseen = read.text.chars().skip(typed).collect::<String>();
    let Ok(pieces) = children.get(read.bubble) else {
        return;
    };
    // The words' size, which the box and the words' place follow.
    let mut size = None;
    for piece in pieces {
        if let Ok((BubblePart::Words, Some(layout), _, _)) = parts.get(*piece) {
            size = Some(layout.size / layout.scale_factor.max(f32::EPSILON));
            if let Ok(words) = children.get(*piece) {
                for (span, text) in words.iter().zip([&shown, &unseen]) {
                    if let Ok(mut span) = spans.get_mut(span)
                        && span.0 != *text
                    {
                        span.0.clone_from(text);
                    }
                }
            }
        }
    }
    let Some(size) = size else {
        return;
    };
    let inner = size + Vec2::splat(2.0 * PADDING);
    // Over its terminal, slid sideways as far as it takes to stay inside
    // the room; the tail stays pointing at the terminal.
    let half_width = inner.x / 2.0 + EDGE;
    let margin = SIDE_MARGIN_H * H_WORLD_UNITS;
    let room_width = data
        .rooms
        .get(&session.room_id)
        .map_or(f32::MAX, |room| room.size.width_h * H_WORLD_UNITS);
    let point_x = bubbles
        .get(read.bubble)
        .map_or(0.0, |bubble| bubble.translation().x);
    let slide = (point_x
        .max(margin + half_width)
        .min(room_width - margin - half_width))
        - point_x;
    let middle = Vec2::new(slide, inner.y / 2.0 + EDGE);
    for piece in pieces {
        let Ok((part, _, mut transform, sprite)) = parts.get_mut(*piece) else {
            continue;
        };
        match (part, sprite) {
            (BubblePart::Words, _) => transform.translation = middle.extend(0.02),
            (BubblePart::Edge, Some(mut sprite)) => {
                sprite.custom_size = Some(inner + Vec2::splat(2.0 * EDGE));
                transform.translation = middle.extend(0.0);
            }
            (BubblePart::Fill, Some(mut sprite)) => {
                sprite.custom_size = Some(inner);
                transform.translation = middle.extend(0.01);
            }
            _ => {}
        }
    }
}
