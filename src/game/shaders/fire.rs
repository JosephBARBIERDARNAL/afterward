//! The fire of everything in a room that burns. Whatever carries `Flame` — the
//! patch a mage sets alight, a rust monster's last blast, the wave
//! a brute sends along the floor — gets a quad of it, the size of what hurts.

use super::*;

pub(super) const SHADER_PATH: &str = "shaders/fire.wgsl";

/// Fire the size of `size`, in world units, around the entity carrying it.
/// `attach_flames` hangs the quad and `stoke_the_flames` keeps it as hot as
/// this says.
#[derive(Component, Clone, Copy, PartialEq)]
pub(in crate::game) struct Flame {
    pub(in crate::game) size: Vec2,
    /// From 0, at rest, to 1 as the burn comes: the warning.
    pub(in crate::game) heat: f32,
    pub(in crate::game) burning: bool,
}

/// Marks a `Flame` whose quad has been hung, so it is only built once.
#[derive(Component)]
pub(in crate::game) struct FlameAttached;

#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub(super) struct FireMaterial {
    #[uniform(0)]
    settings: FireSettings,
}

#[derive(Clone, ShaderType)]
struct FireSettings {
    warning: Vec4,
    burn: Vec4,
    /// Heat, whether it burns, (unused), and its seed.
    state: Vec4,
    /// The quad's size in art pixels.
    grid: Vec4,
}

impl Material2d for FireMaterial {
    fn fragment_shader() -> ShaderRef {
        SHADER_PATH.into()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}

/// Hangs the fire of every `Flame` that does not have one yet, as a child, so
/// it follows what it belongs to and dies with it.
pub(in crate::game) fn attach_flames(
    mut commands: Commands,
    data: Res<GameData>,
    mut assets: ShaderAssets,
    flames: Query<(Entity, &Flame), Without<FlameAttached>>,
) {
    let look = &data.gameplay.effects.hazard;
    for (entity, flame) in &flames {
        // Each fire's seed comes from its entity, so two never flicker in step.
        let seed = (entity.index_u32() % 97) as f32 / 97.0;
        let material = assets.fires.add(FireMaterial {
            settings: FireSettings {
                warning: LinearRgba::from(Color::from(look.warning_color)).to_vec4(),
                burn: LinearRgba::from(Color::from(look.burn_color)).to_vec4(),
                state: Vec4::new(flame.heat, f32::from(flame.burning), 0.0, seed),
                grid: (flame.size / H_WORLD_UNITS * look.pixels_per_h)
                    .round()
                    .max(Vec2::ONE)
                    .extend(0.0)
                    .extend(0.0),
            },
        });
        // A room is drawn and taken down again within a frame by the editor
        // and the room shot, so the fire may be gone before this lands.
        let quad = (
            Mesh2d(assets.quad.0.clone()),
            MeshMaterial2d(material),
            Transform::from_scale(flame.size.extend(1.0)),
        );
        commands
            .entity(entity)
            .queue_silenced(move |mut fire: EntityWorldMut| {
                fire.insert(FlameAttached).with_child(quad);
            });
    }
}

/// Carries a `Flame`'s heat to its quad whenever it changes.
pub(in crate::game) fn stoke_the_flames(
    mut materials: ResMut<Assets<FireMaterial>>,
    flames: Query<(&Flame, &Children), Changed<Flame>>,
    quads: Query<&MeshMaterial2d<FireMaterial>>,
) {
    for (flame, children) in &flames {
        for quad in children.iter().filter_map(|child| quads.get(child).ok()) {
            if let Some(mut material) = materials.get_mut(&quad.0) {
                material.settings.state.x = flame.heat;
                material.settings.state.y = f32::from(flame.burning);
            }
        }
    }
}
