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

const RINGS: i32 = 4;
const SAMPLES_PER_RING: i32 = 12;
const TAU: f32 = 6.2831853;

@fragment
fn fragment(input: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let texel_size = 1.0 / vec2<f32>(textureDimensions(scene_texture));
    var color = textureSample(scene_texture, scene_sampler, input.uv);
    var total_weight = 1.0;
    for (var ring = 1; ring <= RINGS; ring += 1) {
        let distance = f32(ring) / f32(RINGS);
        let weight = exp(-2.0 * distance * distance);
        let offset = texel_size * settings.radius_pixels * distance;
        for (var sample = 0; sample < SAMPLES_PER_RING; sample += 1) {
            let angle = TAU * (f32(sample) + 0.5 * f32(ring % 2)) / f32(SAMPLES_PER_RING);
            let direction = vec2<f32>(cos(angle), sin(angle));
            color += textureSample(scene_texture, scene_sampler, input.uv + direction * offset) * weight;
            total_weight += weight;
        }
    }
    return color / total_weight;
}
