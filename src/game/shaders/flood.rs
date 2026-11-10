//! The flood the city drowned in: a slow caustic web of cold light over deep
//! wine cells, quantized to a pixel grid so it reads as pixel art. The main
//! menu is drawn over it, so the game opens on the thing it is about, and the
//! screen a run ends on closes on the same water. The quad is laid once and
//! kept under the camera, exactly covering the viewport, and only shows on
//! the screens standing over no room at all.

use super::*;
use crate::{
    data::{FloodConfig, HexColor},
    game::prologue::Prologue,
};

pub(super) const SHADER_PATH: &str = "shaders/flood.wgsl";
/// Behind everything else the camera draws.
const FLOOD_Z: f32 = -100.0;
/// In front of it instead, for the prologue: the water stands over the room
/// the run is leaving while its words are written.
const VEIL_Z: f32 = 50.0;

/// The quad the flood is drawn on.
#[derive(Component)]
pub(in crate::game) struct Flood;

#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub(in crate::game) struct FloodMaterial {
    #[uniform(0)]
    settings: FloodSettings,
}

#[derive(Clone, ShaderType)]
struct FloodSettings {
    /// The four tones, darkest first: the heart of a cell, its body, the
    /// lighter water towards its border, and the lit web itself.
    depth: Vec4,
    cell: Vec4,
    crest: Vec4,
    foam: Vec4,
    /// How many water pixels the quad is across and down, how many cells fit
    /// down it, and how far the ordered dither spreads one tone into the next.
    grid: Vec4,
    /// How many turns a cell's point circles in a second, how fast the whole
    /// field drifts, and how far the slow wave bends it.
    flow: Vec4,
    /// How thick the lit borders of the web and the veins inside a cell are,
    /// and how opaque the water is drawn.
    lines: Vec4,
}

impl Material2d for FloodMaterial {
    fn fragment_shader() -> ShaderRef {
        SHADER_PATH.into()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}

/// The same water at any size: a quad `size_h` across shows that much of the
/// field, so the menu viewport and the prologue's veil are cut from one cloth.
fn flood_material(config: &FloodConfig, size_h: Vec2) -> FloodMaterial {
    let tone = |color: HexColor| LinearRgba::from(Color::from(color)).to_vec4();
    FloodMaterial {
        settings: FloodSettings {
            depth: tone(config.depth_color),
            cell: tone(config.cell_color),
            crest: tone(config.crest_color),
            foam: tone(config.foam_color),
            grid: Vec4::new(
                size_h.x * config.pixels_per_h,
                size_h.y * config.pixels_per_h,
                size_h.y / config.cell_h,
                config.dither,
            ),
            flow: Vec4::new(
                config.churn,
                config.drift_h_s / config.cell_h,
                config.wobble,
                0.0,
            ),
            lines: Vec4::new(config.web_width, config.vein_width, 1.0, 0.0),
        },
    }
}

/// Lays the flood the main menu is drawn over, hidden until that menu is up.
pub(super) fn spawn_the_flood(
    mut commands: Commands,
    data: Res<GameData>,
    mut assets: ShaderAssets,
) {
    let viewport_h = Vec2::new(
        data.gameplay.viewport_width_h(),
        data.gameplay.viewport.height_h,
    );
    let material = assets
        .flood
        .add(flood_material(&data.gameplay.effects.flood, viewport_h));
    commands.spawn((
        Mesh2d(assets.quad.0.clone()),
        MeshMaterial2d(material),
        Transform::from_scale((viewport_h * H_WORLD_UNITS).extend(1.0)),
        Visibility::Hidden,
        Flood,
    ));
}

/// Shows the flood under the screens that stand over no room, wherever the
/// camera was left standing — and stands the same water in front of the room
/// instead while the prologue writes over it.
pub(in crate::game) fn show_the_flood(
    screen: Res<State<GameScreen>>,
    settings_return: Res<SettingsReturn>,
    prologue: Res<Prologue>,
    mut materials: ResMut<Assets<FloodMaterial>>,
    camera: Query<&Transform, (With<Camera2d>, Without<Flood>)>,
    mut water: Query<
        (
            &mut Transform,
            &mut Visibility,
            &MeshMaterial2d<FloodMaterial>,
        ),
        With<Flood>,
    >,
) {
    let over_the_flood = settings_return.over_the_flood(*screen.get());
    let Ok(camera) = camera.single() else {
        return;
    };
    for (mut transform, mut visibility, material) in &mut water {
        let (depth, opacity) = if prologue.veil > 0.0 {
            (VEIL_Z, prologue.veil)
        } else {
            (FLOOD_Z, 1.0)
        };
        transform.translation = camera.translation.truncate().extend(depth);
        if let Some(mut water) = materials.get_mut(&material.0) {
            water.settings.lines.z = opacity;
        }
        *visibility = if over_the_flood || prologue.veil > 0.0 {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}
