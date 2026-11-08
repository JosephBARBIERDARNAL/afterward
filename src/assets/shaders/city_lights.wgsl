// Background layer shading: the lit windows and stars of a parallax layer
// breathe, each at its own pace, while everything else is drawn untouched.
//
// A window is found rather than authored: it is a pixel that stands clearly
// brighter than the four around it *and* sits on a dark surround. The first
// test alone would also catch the dither of a bright sky, which is just as
// speckled; the second keeps the search to lights burning against the dark.
// The pixel's position seeds its rhythm, so the same window always breathes
// the same way and the pattern never crawls.

// The mesh2d vertex stage's output. Locations 3 and 4 (tangents and vertex
// colors) are conditional and unused here.
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) world_position: vec4<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(5) @interpolate(flat) instance_index: u32,
}

struct Globals {
    time: f32,
    delta_time: f32,
    frame_count: u32,
}

@group(0) @binding(1)
var<uniform> globals: Globals;

struct CityLights {
    // The layer's own tint, as the sprite's color was.
    tint: vec4<f32>,
    // x: how far above its surroundings a pixel must stand to be a light
    // y: the margin over which that judgement fades in
    // z: how far a light's brightness swings
    // w: how long one breath takes, in seconds
    light: vec4<f32>,
    // x: how bright a pixel's surroundings may be for it to still be a light
    // y: this layer's phase, so the layers never breathe in step
    surround: vec4<f32>,
}

@group(2) @binding(0)
var<uniform> material: CityLights;
@group(2) @binding(1)
var layer_texture: texture_2d<f32>;
@group(2) @binding(2)
var layer_sampler: sampler;

const TAU: f32 = 6.2831853;
// How far out the neighbours that stand for "the wall around this window" are.
const NEIGHBOUR_TEXELS: f32 = 2.0;

fn luminance(color: vec3<f32>) -> f32 {
    return dot(color, vec3<f32>(0.2126, 0.7152, 0.0722));
}

// A stable value in 0..1 for a texel, so every window keeps its own rhythm
// from frame to frame.
fn hash(cell: vec2<f32>, salt: f32) -> f32 {
    return fract(sin(dot(cell, vec2<f32>(127.1, 311.7)) + salt) * 43758.5453);
}

@fragment
fn fragment(input: VertexOutput) -> @location(0) vec4<f32> {
    let color = textureSample(layer_texture, layer_sampler, input.uv);
    let size = vec2<f32>(textureDimensions(layer_texture));
    let step = NEIGHBOUR_TEXELS / size;

    // What the layer looks like just around this pixel.
    var around = 0.0;
    around += luminance(textureSample(layer_texture, layer_sampler, input.uv + vec2<f32>(step.x, 0.0)).rgb);
    around += luminance(textureSample(layer_texture, layer_sampler, input.uv - vec2<f32>(step.x, 0.0)).rgb);
    around += luminance(textureSample(layer_texture, layer_sampler, input.uv + vec2<f32>(0.0, step.y)).rgb);
    around += luminance(textureSample(layer_texture, layer_sampler, input.uv - vec2<f32>(0.0, step.y)).rgb);
    let surround = around * 0.25;
    let excess = luminance(color.rgb) - surround;
    // Bright against the dark around it, and only where that dark is dark.
    let lit = smoothstep(material.light.x, material.light.x + material.light.y, excess)
        * (1.0 - smoothstep(
            material.surround.x,
            material.surround.x + material.light.y,
            surround,
        ));

    // Whole texels breathe together, so the pixel grid stays intact however
    // far the layer is scaled up. Most windows barely move; a few swing.
    let cell = floor(input.uv * size);
    let phase = hash(cell, 0.0) + material.surround.y;
    let swing = material.light.z * hash(cell, 17.0);
    let breath = 1.0 + swing * sin(TAU * (globals.time / material.light.w + phase));

    return vec4<f32>(color.rgb * mix(1.0, breath, lit), color.a) * material.tint;
}
