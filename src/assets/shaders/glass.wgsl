// The hidden player as liquid glass: the frame their sprite is on, read texel
// by texel so it stays pixel art, drawn as a clear body edged in light. Its
// rows ripple sideways like water, and a glint sweeps across it.

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

struct Glass {
    // The frame drawn, as texel corners in its sheet.
    frame: vec4<f32>,
    // The colour the body is filled with, and its rim's.
    fill: vec4<f32>,
    rim: vec4<f32>,
    // x: mirrored; y: how much of the glass shows; z: ripple, in texels.
    look: vec4<f32>,
}

@group(2) @binding(0)
var<uniform> glass: Glass;
@group(2) @binding(1)
var sheet: texture_2d<f32>;

// How many times a second the ripple moves on: stepped, as pixel art moves.
const STEPS_PER_S: f32 = 12.0;

fn solid(at: vec2<f32>) -> bool {
    if any(at < glass.frame.xy) || any(at >= glass.frame.zw) {
        return false;
    }
    return textureLoad(sheet, vec2<i32>(at), 0).a > 0.5;
}

@fragment
fn fragment(input: VertexOutput) -> @location(0) vec4<f32> {
    var uv = input.uv;
    if glass.look.x > 0.5 {
        uv.x = 1.0 - uv.x;
    }
    let time = floor(globals.time * STEPS_PER_S) / STEPS_PER_S;
    var texel = floor(glass.frame.xy + uv * (glass.frame.zw - glass.frame.xy));
    // Each row is pushed sideways by a slow wave running up the body.
    texel.x += round(sin(texel.y * 0.45 - time * 6.0) * glass.look.z);
    if !solid(texel) {
        return vec4<f32>(0.0);
    }
    let rim = !solid(texel + vec2<f32>(1.0, 0.0)) || !solid(texel - vec2<f32>(1.0, 0.0))
        || !solid(texel + vec2<f32>(0.0, 1.0)) || !solid(texel - vec2<f32>(0.0, 1.0));
    var color = glass.fill;
    if rim {
        color = glass.rim;
    } else {
        // What shading the body has shows through faintly, as light bent by it.
        let shade = textureLoad(sheet, vec2<i32>(texel), 0).rgb;
        let light = dot(shade, vec3<f32>(0.3, 0.59, 0.11));
        color = vec4<f32>(mix(glass.fill.rgb, glass.rim.rgb, light * 0.35), glass.fill.a);
        // A glint sweeps across the body, top left to bottom right.
        let band = fract((texel.x + texel.y) / 48.0 - time * 0.7);
        if band < 0.06 {
            color = vec4<f32>(glass.rim.rgb, max(glass.fill.a, 0.6));
        }
    }
    return vec4<f32>(color.rgb, color.a * glass.look.y);
}
