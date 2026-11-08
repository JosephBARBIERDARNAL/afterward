// A droplet falling into water, drawn over a button the moment it is touched.
// The drop falls in from above, the surface flashes where it lands, and rings
// spread out of the impact and fade as they widen.
//
// One square quad, centred on the spot that was touched, quantized to a pixel
// grid and to two tones so the splash reads as the same pixel-art water the
// main menu sits on rather than as a smooth gradient.

// The UI material vertex stage's output. The border fields are unused here,
// but the locations must line up with what that stage writes.
struct UiVertexOutput {
    @location(0) uv: vec2<f32>,
    @location(1) border_widths: vec4<f32>,
    @location(2) border_radius_x: vec4<f32>,
    @location(3) border_radius_y: vec4<f32>,
    @location(4) @interpolate(flat) size: vec2<f32>,
    @builtin(position) position: vec4<f32>,
}

struct Drop {
    // The two tones the splash is drawn in, their alpha being how much of the
    // card below they cover: the water gathering in a ring, and its lit crest.
    crest: vec4<f32>,
    foam: vec4<f32>,
    // x: how far the splash has run, 0 to 1
    // y: how many water pixels the quad is across
    // z: how much of that run the droplet spends falling
    // w: how far the ordered dither spreads one tone into the next
    shape: vec4<f32>,
    // x: how thick a ring is, in quad radii
    // y: how long after one ring the next is born, as a fraction of the spread
    rings: vec4<f32>,
}

@group(1) @binding(0)
var<uniform> drop: Drop;

// How many rings the impact sends out.
const RINGS: i32 = 3;
// How far above the water the droplet starts, in quad radii, how wide it is
// when it lands, and how far its fall stretches it out.
const FALL_HEIGHT: f32 = 0.78;
const DROP_WIDTH: f32 = 0.15;
const FALL_STRETCH: f32 = 1.4;
// How bright the flash where the droplet lands is, how wide, and over how much
// of the spread it dies away.
const FLASH_LIGHT: f32 = 1.2;
const FLASH_WIDTH: f32 = 0.2;
const FLASH_FADE: f32 = 0.33;
// How bright a ring is where it is born, and how much fainter each ring is
// than the one before it.
const RING_LIGHT: f32 = 1.05;
const RING_FALLOFF: f32 = 0.24;
// How much a ring narrows as it widens, and how far out of round the water
// pulls it, so a splash never reads as a drawn target.
const RING_NARROWING: f32 = 0.45;
const RING_WOBBLE: f32 = 0.07;
// Where the splash gives way to the card below, in quad radii, so the rings
// never end on a square rim.
const EDGE_START: f32 = 0.85;
// Where one tone gives way to the next.
const CREST_LEVEL: f32 = 0.26;
const FOAM_LEVEL: f32 = 0.62;

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
fn fragment(input: UiVertexOutput) -> @location(0) vec4<f32> {
    // One flat tone per water pixel, however large the quad is drawn.
    let grid = drop.shape.y;
    let pixel = floor(input.uv * grid);
    let middle = (pixel + vec2<f32>(0.5)) / grid;
    // Where this pixel lies from the spot that was touched: 0 at the spot, 1
    // at the quad's edge. UV y grows downwards, which is the way the drop falls.
    let offset = (middle - vec2<f32>(0.5)) * 2.0;
    let reach = length(offset);

    let age = drop.shape.x;
    let falling = drop.shape.z;
    var light = 0.0;

    if age < falling {
        // The droplet on its way down: a bead stretched out by the fall and
        // drawing itself back together as it nears the water.
        let fall = age / falling;
        let bead = vec2<f32>(offset.x, offset.y + FALL_HEIGHT * (1.0 - fall));
        let stretch = 1.0 + FALL_STRETCH * (1.0 - fall);
        let shaped = vec2<f32>(bead.x, bead.y / stretch);
        let width = DROP_WIDTH * (0.7 + 0.5 * fall);
        light += 1.0 - smoothstep(width * 0.55, width, length(shaped));
    } else {
        let spread = (age - falling) / max(1.0 - falling, 0.0001);
        // The water thrown up where the droplet went in, falling back into the
        // hollow it left.
        let flash = 1.0 - smoothstep(0.0, FLASH_FADE, spread);
        light += FLASH_LIGHT * flash
            * (1.0 - smoothstep(0.0, FLASH_WIDTH + 0.5 * spread, reach));
        // The rings, each born a little after the one before it, narrowing and
        // fading as it widens.
        for (var index = 0; index < RINGS; index++) {
            let born = f32(index) * drop.rings.y;
            let life = (spread - born) / max(1.0 - born, 0.0001);
            if life <= 0.0 {
                continue;
            }
            let width = drop.rings.x * (1.0 - RING_NARROWING * life);
            let turn = atan2(offset.y, offset.x);
            let wobble = 1.0 + RING_WOBBLE
                * sin(3.0 * turn + f32(index) * 2.1)
                * cos(5.0 * turn - 1.3 + f32(index));
            let ring = 1.0 - smoothstep(0.0, width, abs(reach - life * wobble));
            light += RING_LIGHT * ring * (1.0 - life)
                * (1.0 - RING_FALLOFF * f32(index));
        }
    }

    light *= 1.0 - smoothstep(EDGE_START, 1.0, reach);
    light += (dither4(pixel) - 0.5) * drop.shape.w;

    if light > FOAM_LEVEL {
        return drop.foam;
    }
    if light > CREST_LEVEL {
        return drop.crest;
    }
    return vec4<f32>(0.0);
}
