//! The water the main menu is drawn over: a slow caustic web of light over
//! deep blue cells, quantized to a pixel grid so it reads as pixel art. The
//! quad is laid once and kept under the camera, exactly covering the viewport,
//! and only shows while the main menu is up.

use super::*;
use crate::data::HexColor;

pub(super) const SHADER_PATH: &str = "shaders/menu_water.wgsl";
/// Behind everything else the camera draws.
const WATER_Z: f32 = -100.0;

/// The quad the menu's water is drawn on.
#[derive(Component)]
pub(in crate::game) struct MenuWater;

#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub(super) struct MenuWaterMaterial {
    #[uniform(0)]
    settings: WaterSettings,
}

#[derive(Clone, ShaderType)]
struct WaterSettings {
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
    /// How thick the lit borders of the web and the veins inside a cell are.
    lines: Vec4,
}

impl Material2d for MenuWaterMaterial {
    fn fragment_shader() -> ShaderRef {
        SHADER_PATH.into()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}

/// Lays the water the main menu shows, hidden until that menu is up.
pub(super) fn spawn_menu_water(
    mut commands: Commands,
    data: Res<GameData>,
    mut assets: ShaderAssets,
) {
    let config = &data.gameplay.effects.menu_water;
    let tone = |color: HexColor| LinearRgba::from(Color::from(color)).to_vec4();
    let viewport_h = Vec2::new(
        data.gameplay.viewport_width_h(),
        data.gameplay.viewport.height_h,
    );
    let material = assets.water.add(MenuWaterMaterial {
        settings: WaterSettings {
            depth: tone(config.depth_color),
            cell: tone(config.cell_color),
            crest: tone(config.crest_color),
            foam: tone(config.foam_color),
            grid: Vec4::new(
                viewport_h.x * config.pixels_per_h,
                viewport_h.y * config.pixels_per_h,
                viewport_h.y / config.cell_h,
                config.dither,
            ),
            flow: Vec4::new(
                config.churn,
                config.drift_h_s / config.cell_h,
                config.wobble,
                0.0,
            ),
            lines: Vec4::new(config.web_width, config.vein_width, 0.0, 0.0),
        },
    });
    commands.spawn((
        Mesh2d(assets.quad.0.clone()),
        MeshMaterial2d(material),
        Transform::from_scale((viewport_h * H_WORLD_UNITS).extend(1.0)),
        Visibility::Hidden,
        MenuWater,
    ));
}

/// Shows the water while the main menu is up, under wherever the camera was
/// left standing.
pub(in crate::game) fn show_menu_water(
    screen: Res<State<GameScreen>>,
    camera: Query<&Transform, (With<Camera2d>, Without<MenuWater>)>,
    mut water: Query<(&mut Transform, &mut Visibility), With<MenuWater>>,
) {
    let on_the_menu = *screen.get() == GameScreen::MainMenu;
    let Ok(camera) = camera.single() else {
        return;
    };
    for (mut transform, mut visibility) in &mut water {
        transform.translation = camera.translation.truncate().extend(WATER_Z);
        *visibility = if on_the_menu {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}
