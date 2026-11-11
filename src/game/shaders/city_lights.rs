//! The background layers' material: lit windows and stars breathe, each at its
//! own pace, while the rest of the layer is drawn as the sprite drew it.

use super::*;

pub(super) const SHADER_PATH: &str = "shaders/city_lights.wgsl";

#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub(in crate::game) struct CityLightsMaterial {
    #[uniform(0)]
    settings: CityLightsSettings,
    #[texture(1)]
    #[sampler(2)]
    pub(in crate::game) layer: Handle<Image>,
}

#[derive(Clone, ShaderType)]
struct CityLightsSettings {
    tint: Vec4,
    /// How far above its surroundings a pixel must stand to count as a light,
    /// the margin over which that fades in, how far its brightness swings, and
    /// how long one breath takes.
    light: Vec4,
    /// How bright a light's surroundings may be, and the layer's phase, so
    /// the layers never breathe in step.
    surround: Vec4,
}

impl CityLightsMaterial {
    /// The material for one background layer: `depth` is 0 for the backdrop
    /// and grows with each layer in front of it, spreading their phases.
    pub(in crate::game) fn new(
        layer: Handle<Image>,
        tint: Color,
        depth: usize,
        config: &crate::data::CityLightsConfig,
    ) -> Self {
        Self {
            settings: CityLightsSettings {
                tint: LinearRgba::from(tint).to_vec4(),
                light: Vec4::new(
                    config.light_threshold,
                    config.light_softness,
                    config.swing,
                    config.period_s,
                ),
                surround: Vec4::new(config.surround_ceiling, depth as f32 * 0.37, 0.0, 0.0),
            },
            layer,
        }
    }
}

impl Material2d for CityLightsMaterial {
    fn fragment_shader() -> ShaderRef {
        SHADER_PATH.into()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}
