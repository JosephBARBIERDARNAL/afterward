//! The aura a `Glowing` thing wears. Anything the player should walk towards
//! — the bonus item, a health pickup, the door once it unlocks — carries
//! `Glowing`, and `attach_glows` hangs a pool of light behind it.

use super::*;

pub(super) const SHADER_PATH: &str = "shaders/glow.wgsl";
/// Just behind whatever glows, so the light pools around it rather than over it.
const GLOW_Z_OFFSET: f32 = -0.1;

/// A soft pool of light behind the entity carrying this, in its own colour and
/// reaching `radius_h` past it. `attach_glows` turns it into a drawn quad.
#[derive(Component)]
pub(in crate::game) struct Glowing {
    pub(in crate::game) color: Color,
    /// How far the light reaches past the thing it belongs to, in H.
    pub(in crate::game) radius_h: f32,
}

/// Marks a `Glowing` whose aura has been hung, so it is only built once.
#[derive(Component)]
pub(in crate::game) struct GlowAttached;

#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub(super) struct GlowMaterial {
    #[uniform(0)]
    settings: GlowSettings,
}

#[derive(Clone, ShaderType)]
struct GlowSettings {
    /// The colour of the light, its alpha being how bright the core is.
    color: Vec4,
    /// How far the brightness swings, how long one pulse takes, and this
    /// aura's phase, so two auras never pulse in step.
    pulse: Vec4,
}

impl Material2d for GlowMaterial {
    fn fragment_shader() -> ShaderRef {
        SHADER_PATH.into()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}

/// Hangs the aura of every `Glowing` that does not have one yet, as a child,
/// so it follows what it belongs to and dies with it.
pub(in crate::game) fn attach_glows(
    mut commands: Commands,
    data: Res<GameData>,
    mut assets: ShaderAssets,
    glowing: Query<(Entity, &Glowing), Without<GlowAttached>>,
) {
    let config = &data.gameplay.effects.glow;
    for (entity, glow) in &glowing {
        let span = glow.radius_h * 2.0 * H_WORLD_UNITS;
        // Each aura's phase comes from the entity it belongs to, so the ones
        // in a room are never caught pulsing together.
        let phase = (entity.index_u32() % 97) as f32 / 97.0;
        let material = assets.glows.add(GlowMaterial {
            settings: GlowSettings {
                color: LinearRgba::from(glow.color.with_alpha(config.strength)).to_vec4(),
                pulse: Vec4::new(config.pulse, config.pulse_period_s, phase, 0.0),
            },
        });
        commands.entity(entity).insert(GlowAttached).with_child((
            Mesh2d(assets.quad.0.clone()),
            MeshMaterial2d(material),
            Transform::from_xyz(0.0, 0.0, GLOW_Z_OFFSET).with_scale(Vec3::new(span, span, 1.0)),
        ));
    }
}

/// Turns up the aura of the thing within reach of Interact, and back down
/// once the player has stepped away from it.
pub(super) fn brighten_what_is_in_reach(
    data: Res<GameData>,
    in_reach: Res<InReach>,
    mut glows: ResMut<Assets<GlowMaterial>>,
    glowing: Query<(Entity, &Children), With<GlowAttached>>,
    auras: Query<&MeshMaterial2d<GlowMaterial>>,
) {
    let config = &data.gameplay.effects.glow;
    for (entity, children) in &glowing {
        let strength = if in_reach.0 == Some(entity) {
            config.strength * config.reach_boost
        } else {
            config.strength
        };
        for aura in children.iter().filter_map(|child| auras.get(child).ok()) {
            // Only an aura whose brightness has to move is touched, so the
            // others are not uploaded again every frame.
            if glows
                .get(&aura.0)
                .is_some_and(|glow| glow.settings.color.w != strength)
                && let Some(mut glow) = glows.get_mut(&aura.0)
            {
                glow.settings.color.w = strength;
            }
        }
    }
}

/// Lights the exit door the moment it unlocks, in the colour of the bonus
/// category the room behind it will pay out — or in the door's own once no
/// category has a bonus left to offer.
pub(in crate::game) fn light_the_unlocked_door(
    mut commands: Commands,
    data: Res<GameData>,
    session: Res<RunSession>,
    doors: Query<Entity, (With<ExitDoor>, Without<Glowing>)>,
) {
    if !session.door_open {
        return;
    }
    for entity in &doors {
        commands.entity(entity).insert(Glowing {
            color: Color::from(
                session
                    .next_bonus_category
                    .as_ref()
                    .and_then(|category| data.bonuses.categories.get(category))
                    .map_or(data.gameplay.effects.glow.door_color, |category| {
                        category.color
                    }),
            ),
            radius_h: data.gameplay.effects.glow.door_radius_h,
        });
    }
}
