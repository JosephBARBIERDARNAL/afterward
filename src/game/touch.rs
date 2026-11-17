//! What touching a button shows. Hovering it or reaching it with an arrow
//! snaps four corner brackets onto it, a target locked; pressing it throws the
//! spark a landed blow throws, where the mouse pressed or in the middle of the
//! button a key pressed. Both are tuned from `effects.touch`.
//!
//! Anything that wants either writes a [`Touch`]; they stack in their own
//! full-screen layer above the menus and are taken away once played out.

use super::*;

/// Above the menus, so brackets and sparks fall over the card touched.
const TOUCH_Z: i32 = 2;
/// The sheet a press throws: the spark of a landed blow.
const STRIKE_SHEET: &str = "effect_hit_spark";

/// Asks for a button to show it was touched.
#[derive(Message, Clone, Copy)]
pub(super) enum Touch {
    /// The focus reached this button: brackets snap onto it, and off the one
    /// they were on.
    Lock(Entity),
    /// A button was pressed here: a blow's spark.
    Strike(Spot),
}

/// Where a spark lands.
#[derive(Clone, Copy)]
pub(super) enum Spot {
    /// Exactly where the mouse pressed, as a pointer event reports it: window
    /// pixels, which [`UiPointer`] turns into the UI's own.
    Pointer(Vec2),
    /// The middle of a button, which is what pressing it with a key means.
    Middle(Entity),
}

/// Turns where a pointer says it is — window pixels, measured from the
/// window's own corner — into where the UI has that spot, which starts at the
/// letterboxed viewport's corner instead. The two agree only on a window that
/// happens to be exactly 16:9.
#[derive(SystemParam)]
pub(in crate::game) struct UiPointer<'w, 's> {
    cameras: Query<'w, 's, &'static Camera, With<Camera2d>>,
}

impl UiPointer<'_, '_> {
    pub(in crate::game) fn at(&self, reported: Vec2) -> Vec2 {
        let Ok(camera) = self.cameras.single() else {
            return reported;
        };
        let Some(viewport) = camera.viewport.as_ref() else {
            return reported;
        };
        let scale = camera.target_scaling_factor().unwrap_or(1.0);
        reported - viewport.physical_position.as_vec2() / scale.max(f32::EPSILON)
    }
}

/// The full-screen layer every touch is drawn in.
#[derive(Component)]
struct TouchLayer;

/// The four brackets locked on a button, framing it, and how long they have
/// been there.
#[derive(Component)]
pub(super) struct Brackets {
    elapsed_s: f32,
}

/// One of the four brackets: the corner of the button it holds.
#[derive(Component, Clone, Copy)]
struct Corner {
    right: bool,
    bottom: bool,
}

/// A press's spark, and how long it has been playing.
#[derive(Component)]
pub(super) struct Strike {
    elapsed_s: f32,
}

pub(super) struct TouchPlugin;

impl Plugin for TouchPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<Touch>()
            .add_systems(Startup, spawn_touch_layer)
            .add_systems(OnExit(GameScreen::Bonus), clear_the_touches)
            .add_systems(
                Update,
                (open_touches, snap_the_brackets, play_the_strikes).chain(),
            );
    }
}

fn spawn_touch_layer(mut commands: Commands) {
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: px(0),
            right: px(0),
            top: px(0),
            bottom: px(0),
            ..default()
        },
        GlobalZIndex(TOUCH_Z),
        Pickable::IGNORE,
        TouchLayer,
    ));
}

/// What a touch is drawn with, and the buttons it can land on.
#[derive(SystemParam)]
struct Touching<'w, 's> {
    data: Res<'w, GameData>,
    sheets: Res<'w, SpriteSheets>,
    pointer: UiPointer<'w, 's>,
    layers: Query<'w, 's, Entity, With<TouchLayer>>,
    nodes: Query<'w, 's, (&'static ComputedNode, &'static UiGlobalTransform)>,
    locked: Query<'w, 's, Entity, With<Brackets>>,
}

/// Lays the brackets and sparks asked for this frame.
fn open_touches(mut commands: Commands, mut touches: MessageReader<Touch>, touching: Touching) {
    let Touching {
        data,
        sheets,
        pointer,
        layers,
        nodes,
        locked,
    } = touching;
    let Ok(layer) = layers.single() else {
        return;
    };
    let config = &data.gameplay.effects.touch;
    // A button's middle and size in the UI's own pixels; one not laid out
    // yet has neither, and nothing is drawn on it.
    let bounds = |button: Entity| {
        nodes.get(button).ok().map(|(node, transform)| {
            (
                transform.translation * node.inverse_scale_factor,
                node.size * node.inverse_scale_factor,
            )
        })
    };
    for touch in touches.read() {
        match *touch {
            Touch::Lock(button) => {
                let Some((middle, size)) = bounds(button) else {
                    continue;
                };
                // The brackets move to the button reached rather than stack.
                for brackets in &locked {
                    commands.entity(brackets).despawn();
                }
                let color = Color::from(config.bracket_color).with_alpha(0.0);
                let line = px(config.thickness_px);
                commands
                    .spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            left: px(middle.x - size.x / 2.0),
                            top: px(middle.y - size.y / 2.0),
                            width: px(size.x),
                            height: px(size.y),
                            ..default()
                        },
                        Pickable::IGNORE,
                        Brackets { elapsed_s: 0.0 },
                        ChildOf(layer),
                    ))
                    .with_children(|frame| {
                        for (right, bottom) in
                            [(false, false), (true, false), (false, true), (true, true)]
                        {
                            frame.spawn((
                                Node {
                                    position_type: PositionType::Absolute,
                                    width: px(config.arm_px),
                                    height: px(config.arm_px),
                                    border: UiRect {
                                        left: if right { px(0) } else { line },
                                        right: if right { line } else { px(0) },
                                        top: if bottom { px(0) } else { line },
                                        bottom: if bottom { line } else { px(0) },
                                    },
                                    ..default()
                                },
                                BorderColor::all(color),
                                Pickable::IGNORE,
                                Corner { right, bottom },
                            ));
                        }
                    });
            }
            Touch::Strike(spot) => {
                let Some(sheet) = sheets.0.get(STRIKE_SHEET) else {
                    continue;
                };
                let Some(at) = (match spot {
                    Spot::Pointer(reported) => Some(pointer.at(reported)),
                    Spot::Middle(button) => bounds(button).map(|(middle, _)| middle),
                }) else {
                    continue;
                };
                commands.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: px(at.x - config.strike_px / 2.0),
                        top: px(at.y - config.strike_px / 2.0),
                        width: px(config.strike_px),
                        height: px(config.strike_px),
                        ..default()
                    },
                    ImageNode {
                        texture_atlas: Some(TextureAtlas {
                            layout: sheet.layout.clone(),
                            index: config.strike_first_frame,
                        }),
                        ..ImageNode::new(sheet.image.clone())
                    },
                    Pickable::IGNORE,
                    Strike { elapsed_s: 0.0 },
                    ChildOf(layer),
                ));
            }
        }
    }
}

/// Anything a touch drew.
type Touches = Or<(With<Brackets>, With<Strike>)>;

/// Takes every touch away with the screen it was on, so none is left hanging
/// over the room the bonus screen hands back.
fn clear_the_touches(mut commands: Commands, touches: Query<Entity, Touches>) {
    for touch in &touches {
        commands.entity(touch).despawn();
    }
}

/// Snaps the brackets in from outside their button's corners, holds them, and
/// fades them out.
fn snap_the_brackets(
    mut commands: Commands,
    time: Res<Time>,
    data: Res<GameData>,
    mut frames: Query<(Entity, &mut Brackets, &Children)>,
    mut corners: Query<(&mut Node, &mut BorderColor, &Corner)>,
) {
    let config = &data.gameplay.effects.touch;
    for (entity, mut brackets, children) in &mut frames {
        brackets.elapsed_s += time.delta_secs();
        if brackets.elapsed_s >= config.lock_s {
            commands.entity(entity).despawn();
            continue;
        }
        // Eased out: fast off the mark, settling onto the corner.
        let snapped = (brackets.elapsed_s / config.snap_s.max(f32::EPSILON)).min(1.0);
        let eased = 1.0 - (1.0 - snapped).powi(3);
        let off = config.snap_px + (config.gap_px - config.snap_px) * eased;
        let fading = ((config.lock_s - brackets.elapsed_s) / config.fade_s.max(f32::EPSILON))
            .clamp(0.0, 1.0);
        let color = Color::from(config.bracket_color).with_alpha(snapped.min(fading));
        for &child in children {
            let Ok((mut node, mut border, corner)) = corners.get_mut(child) else {
                continue;
            };
            let (outside, unset) = (px(-off), Val::Auto);
            (node.left, node.right) = if corner.right {
                (unset, outside)
            } else {
                (outside, unset)
            };
            (node.top, node.bottom) = if corner.bottom {
                (unset, outside)
            } else {
                (outside, unset)
            };
            *border = BorderColor::all(color);
        }
    }
}

/// Plays every spark through at its sheet's pace and takes it away after its
/// last frame.
fn play_the_strikes(
    mut commands: Commands,
    time: Res<Time>,
    data: Res<GameData>,
    sheets: Res<SpriteSheets>,
    mut strikes: Query<(Entity, &mut Strike, &mut ImageNode)>,
) {
    let Some(sheet) = sheets.0.get(STRIKE_SHEET) else {
        return;
    };
    let first = data.gameplay.effects.touch.strike_first_frame;
    for (entity, mut strike, mut image) in &mut strikes {
        strike.elapsed_s += time.delta_secs();
        let frame = first + (strike.elapsed_s * sheet.fps) as usize;
        if frame >= sheet.frames {
            commands.entity(entity).despawn();
        } else if let Some(atlas) = &mut image.texture_atlas {
            atlas.index = frame;
        }
    }
}
