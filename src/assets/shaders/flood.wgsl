// The flood: a web of cold light wandering slowly over deep wine cells,
// quantized to a pixel grid and to four tones, so it reads as pixel art
// rather than as a smooth gradient. The main menu is drawn over it, which is
// the flooded city seen from its surface.
//
// The web is a cellular field: every cell holds one point circling its middle,
// and the water lights up where the two nearest points are equally far — the
// border between two cells. Bending the coordinates with a slow wave first is
// what makes those borders wander like water instead of ruling a polygon mesh.

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

struct Water {
    // The four tones, darkest first: the heart of a cell, its body, the
    // lighter water gathering towards its border, and the lit web itself.
    depth: vec4<f32>,
    cell: vec4<f32>,
    crest: vec4<f32>,
    foam: vec4<f32>,
    // x, y: how many water pixels the quad is across and down
    // z: how many cells fit down it
    // w: how far the ordered dither spreads one tone into the next
    grid: vec4<f32>,
    // x: how many turns a cell's point circles in a second
    // y: how fast the whole field drifts, in cells per second
    // z: how far the slow wave bends the water
    flow: vec4<f32>,
    // x: how thick the lit borders of the web are
    // y: how thick the fainter veins inside a cell are
    lines: vec4<f32>,
}

@group(2) @binding(0)
var<uniform> water: Water;

const TAU: f32 = 6.2831853;
// How far a cell's point wanders from the cell's middle. Under half a cell, so
// the nearest point is always one of the nine cells around the water pixel.
const WANDER: f32 = 0.42;
// How bright the open body of a cell is, and how far one cell's body may stand
// from the next's, so the water is not one flat blue.
const BODY: f32 = 0.4;
const CELL_SPREAD: f32 = 0.12;
// How much darker the heart of a cell is and how far from its point it
// reaches. Each cell draws its own reach, so some have no heart at all.
const HEART_DARKNESS: f32 = 0.3;
const HEART_REACH: f32 = 0.6;
// How much the veins, the water gathering beside a border, and the lit border
// itself lift the water, and how much the swell crossing it does.
const VEIN_LIGHT: f32 = 0.18;
const HALO_LIGHT: f32 = 0.22;
const HALO_WIDTH: f32 = 2.6;
const WEB_LIGHT: f32 = 0.42;
const SWELL_LIGHT: f32 = 0.06;
// Where one tone gives way to the next.
const CELL_LEVEL: f32 = 0.3;
const CREST_LEVEL: f32 = 0.56;
const FOAM_LEVEL: f32 = 0.8;

fn hash2(cell: vec2<f32>) -> vec2<f32> {
    let seed = vec2<f32>(
        dot(cell, vec2<f32>(127.1, 311.7)),
        dot(cell, vec2<f32>(269.5, 183.3)),
    );
    return fract(sin(seed) * 43758.5453);
}

// What the cellular field has at `place`.
struct Cells {
    // How far the nearest and the second nearest cell point lie. Where the two
    // are equally far, `place` sits on a border.
    nearest: f32,
    second: f32,
    // A steady 0..1 of the cell `place` belongs to, which gives that cell its
    // own shade and its own heart.
    seed: f32,
}

fn cell_points(place: vec2<f32>, turn: f32) -> Cells {
    let home = floor(place);
    var nearest = 8.0;
    var second = 8.0;
    var seed_of_nearest = 0.0;
    for (var dy = -1; dy <= 1; dy++) {
        for (var dx = -1; dx <= 1; dx++) {
            let cell = home + vec2<f32>(f32(dx), f32(dy));
            let seed = hash2(cell);
            // Each point circles its cell's middle, from its own phase.
            let point = cell + vec2<f32>(0.5) + WANDER * vec2<f32>(
                sin(TAU * (seed.x + turn)),
                cos(TAU * (seed.y + turn * 0.83)),
            );
            let reach = distance(point, place);
            if reach < nearest {
                second = nearest;
                nearest = reach;
                seed_of_nearest = fract(seed.x + seed.y);
            } else if reach < second {
                second = reach;
            }
        }
    }
    return Cells(nearest, second, seed_of_nearest);
}

// The 2x2 ordered dither, written out rather than looked up in a table.
fn dither2(cell: vec2<f32>) -> f32 {
    return 2.0 * cell.x + 3.0 * cell.y - 4.0 * cell.x * cell.y;
}

// The 4x4 ordered dither, which is the 2x2 one repeated inside itself. It
// scatters the pixels along a tone's edge instead of cutting a clean line.
fn dither4(pixel: vec2<f32>) -> f32 {
    let fine = pixel - 2.0 * floor(pixel * 0.5);
    let coarse = floor(pixel * 0.5) - 2.0 * floor(pixel * 0.25);
    return (4.0 * dither2(coarse) + dither2(fine) + 0.5) / 16.0;
}

@fragment
fn fragment(input: VertexOutput) -> @location(0) vec4<f32> {
    // One flat colour per water pixel, however far the quad is scaled up.
    let pixel = floor(input.uv * water.grid.xy);
    let aspect = water.grid.x / water.grid.y;
    let middle = (pixel + vec2<f32>(0.5)) / water.grid.xy;
    var place = vec2<f32>(middle.x * aspect, middle.y) * water.grid.z;

    // The field drifts, and two slow waves bend it, so the borders wander like
    // water instead of ruling a polygon mesh.
    place += globals.time * water.flow.y * vec2<f32>(1.0, 0.31);
    place += water.flow.z * vec2<f32>(
        sin(place.y * 1.3 - globals.time * 0.21),
        cos(place.x * 1.1 + globals.time * 0.17),
    );
    place += water.flow.z * 0.45 * vec2<f32>(
        sin(place.y * 3.1 + globals.time * 0.33),
        cos(place.x * 2.7 - globals.time * 0.29),
    );

    let turn = globals.time * water.flow.x;
    let near = cell_points(place, turn);
    // A second, finer field lays the veins crossing a cell and frays the edge
    // of its heart.
    let fine = cell_points(place * 2.7 + vec2<f32>(11.3, 5.7), turn * 1.6);
    let vein = 1.0 - smoothstep(0.0, water.lines.y, fine.second - fine.nearest);
    // A swell crossing the whole surface, lifting stretches of it at a time and
    // thickening the borders it passes over.
    let swell = 0.5 + 0.5 * sin(TAU * (place.x * 0.07 + place.y * 0.05 - globals.time * 0.03));

    let border = near.second - near.nearest;
    let width = water.lines.x * (0.65 + 0.7 * swell);
    let web = 1.0 - smoothstep(0.0, width, border);
    let halo = 1.0 - smoothstep(0.0, width * HALO_WIDTH, border);
    // How deep a cell's heart runs is the cell's own: some are a dark hollow,
    // some are barely there, and the finer field frays every edge.
    let reach = HEART_REACH * (0.2 + 1.1 * near.seed);
    let heart = 1.0 - smoothstep(reach * 0.3, reach, near.nearest + 0.25 * fine.nearest);

    var light = BODY + (near.seed - 0.5) * CELL_SPREAD - HEART_DARKNESS * heart;
    light += VEIN_LIGHT * vein + HALO_LIGHT * halo + WEB_LIGHT * web;
    light += SWELL_LIGHT * swell + (dither4(pixel) - 0.5) * water.grid.w;

    var color = water.depth;
    if light > CELL_LEVEL {
        color = water.cell;
    }
    if light > CREST_LEVEL {
        color = water.crest;
    }
    if light > FOAM_LEVEL {
        color = water.foam;
    }
    return color;
}
