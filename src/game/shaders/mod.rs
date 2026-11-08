//! The game's 2D material shaders, drawn on `Mesh2d` quads: the background
//! layers' breathing city lights, and the aura a `Glowing` thing wears. The
//! full-screen pass that muffles the view behind a menu lives in
//! `pause_blur.rs`.

use super::*;
use bevy::{
    render::render_resource::{AsBindGroup, ShaderType},
    shader::ShaderRef,
    sprite_render::{AlphaMode2d, Material2d, Material2dPlugin},
};

mod city_lights;
mod glow;

pub(super) use city_lights::CityLightsMaterial;
#[cfg(test)]
pub(super) use glow::GlowAttached;
pub(super) use glow::{Glowing, attach_glows, light_the_unlocked_door};

pub(super) struct ShaderPlugin;

impl Plugin for ShaderPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            Material2dPlugin::<CityLightsMaterial>::default(),
            Material2dPlugin::<glow::GlowMaterial>::default(),
        ))
        .add_systems(Startup, initialize_shaded_quads);
    }
}

/// Every shaded quad is the same unit square, sized by its transform, so one
/// mesh serves them all.
#[derive(Resource)]
pub(super) struct ShadedQuad(pub(super) Handle<Mesh>);

fn initialize_shaded_quads(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    commands.insert_resource(ShadedQuad(meshes.add(Rectangle::new(1.0, 1.0))));
}

/// What a system needs to build shaded quads.
#[derive(SystemParam)]
pub(super) struct ShaderAssets<'w> {
    pub(super) quad: Res<'w, ShadedQuad>,
    pub(super) city_lights: ResMut<'w, Assets<CityLightsMaterial>>,
    glows: ResMut<'w, Assets<glow::GlowMaterial>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn every_material_points_at_a_shader_that_is_there() {
        let asset_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/assets");
        for path in [city_lights::SHADER_PATH, glow::SHADER_PATH] {
            assert!(
                asset_root.join(path).is_file(),
                "missing shader source: {path}"
            );
        }
    }
}
