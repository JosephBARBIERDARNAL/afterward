//! An enemy's shot. Whatever carries `Bolt` — a plain shot, one without a
//! sheet of its own — gets a quad of it: a hot head the size of what hurts,
//! and a tail burning out behind.

use super::*;

pub(super) const SHADER_PATH: &str = "shaders/bolt.wgsl";

/// A shot `size` across, in world units, flying the way `direction` points
/// (-1 left, 1 right). `attach_bolts` hangs its quad.
#[derive(Component)]
pub(in crate::game) struct Bolt {
    pub(in crate::game) size: Vec2,
    pub(in crate::game) direction: f32,
}

/// Marks a `Bolt` whose quad has been hung, so it is only built once.
#[derive(Component)]
pub(in crate::game) struct BoltAttached;

#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub(super) struct BoltMaterial {
    #[uniform(0)]
    settings: BoltSettings,
}

#[derive(Clone, ShaderType)]
struct BoltSettings {
    core: Vec4,
    tail: Vec4,
    /// The quad's size in art pixels, the way the shot flies, and how much
    /// of the quad's length is the head.
    shape: Vec4,
}

impl Material2d for BoltMaterial {
    fn fragment_shader() -> ShaderRef {
        SHADER_PATH.into()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}

/// Hangs the quad of every `Bolt` that does not have one yet, as a child,
/// reaching back from the shot along the way it came.
pub(in crate::game) fn attach_bolts(
    mut commands: Commands,
    data: Res<GameData>,
    mut assets: ShaderAssets,
    bolts: Query<(Entity, &Bolt), Without<BoltAttached>>,
) {
    let look = &data.gameplay.effects.bolt;
    for (entity, bolt) in &bolts {
        // The head is drawn a little larger than what hurts, so a shot that
        // small is still seen coming.
        let quad = Vec2::new(bolt.size.x * look.length, bolt.size.y * look.thickness);
        let material = assets.bolts.add(BoltMaterial {
            settings: BoltSettings {
                core: LinearRgba::from(Color::from(look.core_color)).to_vec4(),
                tail: LinearRgba::from(Color::from(look.tail_color)).to_vec4(),
                shape: (quad / H_WORLD_UNITS * look.pixels_per_h)
                    .round()
                    .max(Vec2::ONE)
                    .extend(bolt.direction)
                    .extend(1.0 / look.length),
            },
        });
        let trail = (
            Mesh2d(assets.quad.0.clone()),
            MeshMaterial2d(material),
            Transform::from_xyz(-bolt.direction * (quad.x - bolt.size.x) / 2.0, 0.0, 0.0)
                .with_scale(quad.extend(1.0)),
        );
        // A shot may land the frame it is fired.
        commands
            .entity(entity)
            .queue_silenced(move |mut shot: EntityWorldMut| {
                shot.insert(BoltAttached).with_child(trail);
            });
    }
}
