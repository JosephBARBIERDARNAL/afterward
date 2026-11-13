// Fire, for everything in a room that burns: a vent's jet, the patch a mage
// sets alight, a rust monster's last blast, the wave a brute sends along the
// floor. Drawn on a quad the size of what hurts, on a grid of art pixels and
// in three tones, so it reads as pixel art beside the sprites.
//
// At rest a vent is only its grate, dark with dim slots. As the burn comes
// (`heat` from 0 to 1) the slots light and embers climb out of it, more of
// them and higher; then the jet stands for as long as it burns.

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

struct Vent {
    // The colour of the warning and of a flame's edge.
    warning: vec4<f32>,
    // The colour of a flame's core.
    burn: vec4<f32>,
    // x: heat, from 0 at rest to 1 as the burn comes
    // y: 1 while it burns
    // z: 1 for a vent, which has a grate in the floor
    // w: this fire's seed, so two never flicker in step
    state: vec4<f32>,
    // xy: the quad's size in art pixels
    grid: vec4<f32>,
}

@group(2) @binding(0)
var<uniform> vent: Vent;

// How many times a second the fire moves: few, like a hand-drawn loop.
const STEPS_PER_S: f32 = 12.0;
// How tall the grate is, in art pixels, and how far apart its slots are.
const GRATE_ROWS: f32 = 3.0;
const SLOT_EVERY: f32 = 3.0;
const GRATE_COLOR: vec3<f32> = vec3<f32>(0.012, 0.011, 0.03);
const SLOT_COLD: vec3<f32> = vec3<f32>(0.22, 0.05, 0.025);

fn hash(at: vec2<f32>) -> f32 {
    return fract(sin(dot(at, vec2<f32>(127.1, 311.7))) * 43758.5453);
}

// Smooth noise between hashed grid points.
fn noise(at: vec2<f32>) -> f32 {
    let cell = floor(at);
    let within = fract(at);
    let blend = within * within * (3.0 - 2.0 * within);
    return mix(
        mix(hash(cell), hash(cell + vec2<f32>(1.0, 0.0)), blend.x),
        mix(hash(cell + vec2<f32>(0.0, 1.0)), hash(cell + vec2<f32>(1.0, 1.0)), blend.x),
        blend.y,
    );
}

@fragment
fn fragment(input: VertexOutput) -> @location(0) vec4<f32> {
    let cells = vent.grid.xy;
    let cell = min(floor(input.uv * cells), cells - vec2<f32>(1.0));
    // The art pixel, counted from the floor up.
    let pixel = vec2<f32>(cell.x, cells.y - 1.0 - cell.y);
    // Across the quad from -1 to 1, and up it from 0 to 1.
    let across = (pixel.x + 0.5) / cells.x * 2.0 - 1.0;
    let up = (pixel.y + 0.5) / cells.y;
    let time = floor(globals.time * STEPS_PER_S) / STEPS_PER_S;
    let heat = vent.state.x;
    let burning = vent.state.y > 0.5;
    let seed = vent.state.w * 61.0;

    var color = vec3<f32>(0.0);
    var alpha = 0.0;

    if burning {
        // A jet, wide at the floor and ragged toward its top, its body
        // climbing as the noise scrolls down through it.
        let flicker = noise(vec2<f32>(pixel.x * 0.33 + seed, pixel.y * 0.3 - time * 10.0));
        let top = 0.88 + 0.12 * noise(vec2<f32>(pixel.x * 0.5 + seed, time * 6.0));
        // It fills most of its quad, so what is seen is what hurts.
        let width = mix(1.1, 0.6, up);
        let body = 1.0 - abs(across) / width - up / top * 0.4 + (flicker - 0.5) * 0.5;
        if up < top && body > 0.0 {
            alpha = 1.0;
            if body > 0.5 {
                color = vent.burn.rgb;
            } else if body > 0.22 {
                color = mix(vent.warning.rgb, vent.burn.rgb, 0.45);
            } else {
                color = vent.warning.rgb;
            }
        }
    } else if heat > 0.0 {
        // Embers: one to a column, in more columns and climbing higher as
        // the burn comes, each trailing a dimmer pixel under it.
        let column = hash(vec2<f32>(pixel.x, seed));
        let rise = fract(hash(vec2<f32>(pixel.x * 1.7, seed + 3.0)) + time * (0.7 + column));
        let ember = floor(rise * cells.y * (0.25 + 0.75 * heat));
        if column < heat * 0.7 && pixel.y >= GRATE_ROWS * vent.state.z {
            if pixel.y == ember {
                color = mix(vent.warning.rgb, vent.burn.rgb, heat * 0.5);
                alpha = 1.0 - rise * 0.5;
            } else if pixel.y == ember - 1.0 {
                color = vent.warning.rgb;
                alpha = 0.45 * (1.0 - rise);
            }
        }
        // And a glow hugging the floor that swells with them.
        let glow = heat * max(1.0 - pixel.y / (2.0 + heat * 4.0), 0.0) * (1.0 - abs(across) * 0.6);
        if alpha == 0.0 && glow > 0.0 {
            color = vent.warning.rgb;
            alpha = glow * 0.5;
        }
    }

    // The grate a vent burns through, over everything else.
    if vent.state.z > 0.5 && pixel.y < GRATE_ROWS {
        alpha = 1.0;
        color = GRATE_COLOR;
        if pixel.y == GRATE_ROWS - 1.0 && pixel.x % SLOT_EVERY == 1.0 {
            color = mix(SLOT_COLD, vent.warning.rgb, select(heat, 1.0, burning));
        }
    }
    return vec4<f32>(color, alpha);
}
