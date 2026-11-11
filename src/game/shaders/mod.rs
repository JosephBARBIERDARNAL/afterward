//! The game's 2D material shaders, drawn on `Mesh2d` quads: the background
//! layers' breathing city lights, the aura a `Glowing` thing wears, and the
//! flood the main menu is drawn over. The full-screen pass that muffles the
//! view behind a menu lives in `pause_blur.rs`.

use super::*;
use bevy::{
    render::render_resource::{AsBindGroup, ShaderType},
    shader::ShaderRef,
    sprite_render::{AlphaMode2d, Material2d, Material2dPlugin},
};

mod city_lights;
mod flood;
mod glow;

pub(super) use city_lights::CityLightsMaterial;
#[cfg(test)]
pub(super) use flood::Flood;
pub(super) use flood::show_the_flood;
#[cfg(test)]
pub(super) use glow::GlowAttached;
pub(super) use glow::{Glowing, attach_glows, light_the_unlocked_door};

pub(super) struct ShaderPlugin;

impl Plugin for ShaderPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            Material2dPlugin::<CityLightsMaterial>::default(),
            Material2dPlugin::<glow::GlowMaterial>::default(),
            Material2dPlugin::<flood::FloodMaterial>::default(),
        ))
        .add_systems(
            Startup,
            (initialize_shaded_quads, flood::spawn_the_flood).chain(),
        )
        .add_systems(Update, glow::brighten_what_is_in_reach);
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
    flood: ResMut<'w, Assets<flood::FloodMaterial>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    /// The ripple is a UI material rather than a 2D one, and lives beside this
    /// module, but it is a material all the same and answers to the same check.
    #[test]
    fn every_material_points_at_a_shader_that_is_there() {
        let asset_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/assets");
        for path in [
            city_lights::SHADER_PATH,
            flood::SHADER_PATH,
            glow::SHADER_PATH,
            crate::game::ripple::SHADER_PATH,
        ] {
            assert!(
                asset_root.join(path).is_file(),
                "missing shader source: {path}"
            );
        }
    }
}
