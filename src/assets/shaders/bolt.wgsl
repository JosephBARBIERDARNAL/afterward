// An enemy's shot: a hot head with a tail burning out behind it, drawn on a
// quad that reaches back from the shot along the way it came. On a grid of
// art pixels and in three tones, like the fire of `vent.wgsl`.

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

struct Bolt {
    // The colour of the head, and of the tail behind it.
    core: vec4<f32>,
    tail: vec4<f32>,
    // xy: the quad's size in art pixels
    // z: the way the shot flies, -1 left or 1 right
    // w: how much of the quad's length is the head
    shape: vec4<f32>,
}

@group(2) @binding(0)
var<uniform> bolt: Bolt;

// How many times a second the tail flickers.
const STEPS_PER_S: f32 = 16.0;

fn hash(at: vec2<f32>) -> f32 {
    return fract(sin(dot(at, vec2<f32>(127.1, 311.7))) * 43758.5453);
}

@fragment
fn fragment(input: VertexOutput) -> @location(0) vec4<f32> {
    let cells = bolt.shape.xy;
    let cell = min(floor(input.uv * cells), cells - vec2<f32>(1.0));
    // Along the shot from the end of its tail (0) to its nose (1), and
    // across it from its middle (0) to its edge (1).
    var along = (cell.x + 0.5) / cells.x;
    if bolt.shape.z < 0.0 {
        along = 1.0 - along;
    }
    let across = abs((cell.y + 0.5) / cells.y * 2.0 - 1.0);
    let head_from = 1.0 - bolt.shape.w;
    let time = floor(globals.time * STEPS_PER_S) / STEPS_PER_S;

    if along >= head_from {
        // The head: a hot core in a rim of the tail's colour, rounded off
        // at the nose.
        let nose = (along - head_from) / bolt.shape.w;
        if across > 1.0 - nose * nose * 0.7 {
            return vec4<f32>(0.0);
        }
        if across < 0.5 {
            return vec4<f32>(bolt.core.rgb, 1.0);
        }
        return vec4<f32>(bolt.tail.rgb, 1.0);
    }
    // The tail: thinner and fainter the further back, and broken up by a
    // flicker so it reads as burning out rather than as a streak.
    let back = along / head_from;
    if across > back * 0.8 {
        return vec4<f32>(0.0);
    }
    let lit = hash(vec2<f32>(cell.x, cell.y + time * 7.0));
    if lit > back + 0.25 {
        return vec4<f32>(0.0);
    }
    return vec4<f32>(bolt.tail.rgb, floor(back * 4.0 + 1.0) / 4.0);
}
