//! The droplet that falls on a button the moment it is touched: hovered,
//! reached with an arrow, or clicked. It is the flood's water carried onto the
//! menus, drawn by `shaders/ripple.wgsl` on a square UI node laid over the
//! spot, and it is the one shader the game draws through the UI pipeline
//! rather than on a `Mesh2d` quad, which is why it sits beside
//! `src/game/shaders/` instead of inside it.
//!
//! Anything that wants a droplet writes a [`Splash`]; the splashes stack in
//! their own full-screen layer above the menus and die once they have played
//! out.

use super::*;
use crate::data::HexColor;
use bevy::{
    render::render_resource::{AsBindGroup, ShaderType},
    shader::ShaderRef,
    ui_render::prelude::{MaterialNode, UiMaterial, UiMaterialPlugin},
};

pub(super) const SHADER_PATH: &str = "shaders/ripple.wgsl";
/// Above the menus, so a droplet falls over the card it was touched on.
const RIPPLE_Z: i32 = 2;
/// How many droplets may be falling at once. Running the mouse down a long
/// menu touches every button on the way; past this many the oldest is taken
/// away to make room.
const MOST_AT_ONCE: usize = 8;

/// Asks for a droplet to fall on a button.
#[derive(Message, Clone, Copy)]
pub(super) struct Splash(pub(super) Spot);

/// Where a droplet lands.
#[derive(Clone, Copy)]
pub(super) enum Spot {
    /// Exactly where the mouse touched the button, as a pointer event reports
    /// it: window pixels, which [`UiPointer`] turns into the UI's own.
    Pointer(Vec2),
    /// The middle of a button, which is what reaching it with an arrow means.
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

/// The full-screen layer every droplet is drawn in.
#[derive(Component)]
struct RippleLayer;

/// A droplet in flight, and how long it has been falling.
#[derive(Component)]
pub(super) struct Ripple {
    elapsed_s: f32,
}

#[derive(Asset, TypePath, AsBindGroup, Clone)]
struct RippleMaterial {
    #[uniform(0)]
    settings: RippleSettings,
}

#[derive(Clone, ShaderType)]
struct RippleSettings {
    /// The two tones the splash is drawn in, their alpha being how much of the
    /// card below each one covers.
    crest: Vec4,
    foam: Vec4,
    /// How far the splash has run (0 to 1), how many water pixels it is
    /// across, how much of that run the droplet spends falling, and how far
    /// the ordered dither spreads one tone into the next.
    shape: Vec4,
    /// How thick a ring is, in quad radii, and how long after one ring the
    /// next is born.
    rings: Vec4,
}

impl UiMaterial for RippleMaterial {
    fn fragment_shader() -> ShaderRef {
        SHADER_PATH.into()
    }
}

pub(super) struct RipplePlugin;

impl Plugin for RipplePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(UiMaterialPlugin::<RippleMaterial>::default())
            .add_message::<Splash>()
            .add_systems(Startup, spawn_ripple_layer)
            .add_systems(OnExit(GameScreen::Bonus), dry_the_ripples)
            .add_systems(Update, (open_splashes, age_ripples).chain());
    }
}

fn spawn_ripple_layer(mut commands: Commands) {
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: px(0),
            right: px(0),
            top: px(0),
            bottom: px(0),
            ..default()
        },
        GlobalZIndex(RIPPLE_Z),
        Pickable::IGNORE,
        RippleLayer,
    ));
}

/// The water a droplet is drawn in, and the places one can land.
#[derive(SystemParam)]
struct Splashing<'w, 's> {
    data: Res<'w, GameData>,
    materials: ResMut<'w, Assets<RippleMaterial>>,
    pointer: UiPointer<'w, 's>,
    layers: Query<'w, 's, Entity, With<RippleLayer>>,
    nodes: Query<'w, 's, (&'static ComputedNode, &'static UiGlobalTransform)>,
    falling: Query<'w, 's, (Entity, &'static Ripple)>,
}

/// Lays a droplet for every splash asked for this frame.
fn open_splashes(mut commands: Commands, mut splashes: MessageReader<Splash>, water: Splashing) {
    let Splashing {
        data,
        mut materials,
        pointer,
        layers,
        nodes,
        falling,
    } = water;
    let Ok(layer) = layers.single() else {
        return;
    };
    let config = &data.gameplay.effects.ripple;
    let tone = |color: HexColor, opacity: f32| {
        LinearRgba::from(Color::from(color).with_alpha(opacity)).to_vec4()
    };
    // The droplets already falling, the one that has been falling longest
    // first, so a crowded layer gives up its oldest to make room.
    let mut oldest_first = falling.iter().collect::<Vec<_>>();
    oldest_first.sort_by(|(_, one), (_, other)| other.elapsed_s.total_cmp(&one.elapsed_s));
    let mut live = oldest_first.len();
    let mut spare = oldest_first.into_iter();
    for splash in splashes.read() {
        while live >= MOST_AT_ONCE {
            let Some((oldest, _)) = spare.next() else {
                break;
            };
            commands.entity(oldest).despawn();
            live -= 1;
        }
        live += 1;
        // A button that has not been laid out yet has no middle to fall on;
        // the droplet lands at the layer's corner rather than nowhere.
        let at = match splash.0 {
            Spot::Pointer(reported) => pointer.at(reported),
            Spot::Middle(button) => nodes.get(button).map_or(Vec2::ZERO, |(node, transform)| {
                transform.translation * node.inverse_scale_factor
            }),
        };
        let material = materials.add(RippleMaterial {
            settings: RippleSettings {
                crest: tone(config.crest_color, config.crest_opacity),
                foam: tone(config.foam_color, config.foam_opacity),
                shape: Vec4::new(0.0, config.pixels, config.fall, config.dither),
                rings: Vec4::new(config.ring_width, config.ring_interval, 0.0, 0.0),
            },
        });
        commands.spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(at.x - config.radius_px),
                top: px(at.y - config.radius_px),
                width: px(config.radius_px * 2.0),
                height: px(config.radius_px * 2.0),
                ..default()
            },
            MaterialNode(material),
            Pickable::IGNORE,
            Ripple { elapsed_s: 0.0 },
            ChildOf(layer),
        ));
    }
}

/// Takes every droplet away with the screen it fell on, so none is left
/// hanging over the room the bonus screen hands back.
fn dry_the_ripples(mut commands: Commands, falling: Query<Entity, With<Ripple>>) {
    for ripple in &falling {
        commands.entity(ripple).despawn();
    }
}

/// Runs every droplet on, and takes it away once it has played out.
fn age_ripples(
    mut commands: Commands,
    time: Res<Time>,
    data: Res<GameData>,
    mut materials: ResMut<Assets<RippleMaterial>>,
    mut falling: Query<(Entity, &mut Ripple, &MaterialNode<RippleMaterial>)>,
) {
    let duration_s = data.gameplay.effects.ripple.duration_s.max(0.0001);
    for (entity, mut ripple, node) in &mut falling {
        ripple.elapsed_s += time.delta_secs();
        if ripple.elapsed_s >= duration_s {
            commands.entity(entity).despawn();
            continue;
        }
        if let Some(mut material) = materials.get_mut(&node.0) {
            material.settings.shape.x = ripple.elapsed_s / duration_s;
        }
    }
}
