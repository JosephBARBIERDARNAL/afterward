// The aura a `Glowing` thing wears: a soft round pool of light that breathes,
// drawn on a quad behind whatever it belongs to. It reaches the edge of its
// quad at zero strength, so it melts into the room instead of ending on a rim.

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

struct Glow {
    // rgb: the colour of the light; a: how bright its core is.
    color: vec4<f32>,
    // x: how far the brightness swings
    // y: how long one pulse takes, in seconds
    // z: this aura's phase, so two auras never pulse in step
    pulse: vec4<f32>,
}

@group(2) @binding(0)
var<uniform> glow: Glow;

const TAU: f32 = 6.2831853;
// How sharply the pool of light falls off towards the edge of its quad.
const FALLOFF: f32 = 2.5;

@fragment
fn fragment(input: VertexOutput) -> @location(0) vec4<f32> {
    let offset = (input.uv - vec2<f32>(0.5)) * 2.0;
    let falloff = pow(max(1.0 - length(offset), 0.0), FALLOFF);
    let breath = 1.0 + glow.pulse.x * sin(TAU * (globals.time / glow.pulse.y + glow.pulse.z));
    return vec4<f32>(
        glow.color.rgb,
        clamp(glow.color.a * falloff * breath, 0.0, 1.0),
    );
}
