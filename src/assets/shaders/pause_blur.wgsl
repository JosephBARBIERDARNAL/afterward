struct FullscreenVertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

struct PauseBlur {
    radius_pixels: f32,
};

@group(0) @binding(0) var scene_texture: texture_2d<f32>;
@group(0) @binding(1) var scene_sampler: sampler;
@group(0) @binding(2) var<uniform> settings: PauseBlur;

@fragment
fn fragment(input: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let texel_size = 1.0 / vec2<f32>(textureDimensions(scene_texture));
    let offset = texel_size * settings.radius_pixels;
    let diagonal = offset * 0.7;
    var color = textureSample(scene_texture, scene_sampler, input.uv) * 0.2;
    color += textureSample(scene_texture, scene_sampler, input.uv + vec2<f32>(offset.x, 0.0)) * 0.1;
    color += textureSample(scene_texture, scene_sampler, input.uv - vec2<f32>(offset.x, 0.0)) * 0.1;
    color += textureSample(scene_texture, scene_sampler, input.uv + vec2<f32>(0.0, offset.y)) * 0.1;
    color += textureSample(scene_texture, scene_sampler, input.uv - vec2<f32>(0.0, offset.y)) * 0.1;
    color += textureSample(scene_texture, scene_sampler, input.uv + diagonal) * 0.1;
    color += textureSample(scene_texture, scene_sampler, input.uv - diagonal) * 0.1;
    color += textureSample(scene_texture, scene_sampler, input.uv + vec2<f32>(diagonal.x, -diagonal.y)) * 0.1;
    color += textureSample(scene_texture, scene_sampler, input.uv + vec2<f32>(-diagonal.x, diagonal.y)) * 0.1;
    return color;
}
