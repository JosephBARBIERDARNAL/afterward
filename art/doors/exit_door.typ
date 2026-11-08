// The exit door as a sprite sheet: 14 frames of 48 x 96 px (0.75 x 1.5 H at
// the surfaces' 64 px per H), left to right. Frame 0 is locked (magenta
// lights), frame 1 unlocked (cyan), and frames 2-13 slide the two panels
// apart over a glowing doorway until it is fully open.
#import "../_pixel.typ": pixels-by

#let frame-width = 48
#let frame-height = 96
// How far each panel has slid into the frame, per frame: the panels ease out
// of rest, run at full speed, then settle against the frame.
#let openings = (0, 0, 1, 2, 4, 6, 8, 10, 12, 14, 15, 16, 17, 18)

#let palette = (
  "#": rgb("#08070f"), // outline, grooves
  "f": rgb("#1c1a2e"), // frame
  "F": rgb("#2e2a47"), // frame highlight
  "p": rgb("#221f3b"), // panel
  "P": rgb("#332f4c"), // panel highlight
  "t": rgb("#13303a"), // unlit trim
  "c": rgb("#6fd3e8"), // emergency strip, the way out
  "a": rgb("#c8389b"), // dying signage, still locked
  "v": rgb("#100a16"), // doorway depth
  "s": rgb("#0b0810"), // doorway scanline
  "w": rgb("#3fa8bf"), // doorway glow
  "W": rgb("#9fe4ee"), // doorway core
)

// The doorway behind the panels: a bright core fading out, with scanlines.
#let doorway(x, y) = {
  let from-center = calc.abs(x - 23.5)
  if from-center < 3 { "W" } else if from-center < 7 { "w" } else if (
    calc.rem(y, 4) == 0
  ) { "s" } else { "v" }
}

// One panel pixel at column `u` (0 at its outer edge, 17 at the seam), at
// row `y`; `light` is the neon color of the frame's state.
#let panel(u, y, light) = {
  let inner = u == 17
  if inner { if light == "a" { "t" } else { light } } else if (
    y == 32 or y == 64
  ) { "#" } else if u >= 13 and u <= 14 and y >= 44 and y <= 52 {
    light
  } else if u == 0 { "P" } else { "p" }
}

#let door(x, y, frame) = {
  let open = openings.at(frame)
  let light = if frame == 0 { "a" } else { "c" }
  let trim = if frame == 0 { "t" } else { "c" }
  if x == 0 or x == 47 or y == 0 or y == 95 { "#" } else if (
    y <= 8 or x <= 4 or x >= 43
  ) {
    // Frame, with a status strip in the header.
    if y >= 4 and y <= 5 and x >= 14 and x <= 33 { light } else if (
      x == 1 or y == 1
    ) { "F" } else { "f" }
  } else if x == 5 or x == 42 or y == 9 { trim } else if x <= 23 {
    // Left panel, sliding left: its outer edge stays hidden in the frame.
    let u = x - 6 + open
    if u <= 17 { panel(u, y, light) } else { doorway(x, y) }
  } else {
    // Right panel, mirrored, sliding right.
    let u = 41 - x + open
    if u <= 17 { panel(u, y, light) } else { doorway(x, y) }
  }
}

#pixels-by(
  frame-width * openings.len(),
  frame-height,
  palette,
  (x, y) => door(
    calc.rem(x, frame-width),
    y,
    calc.floor(x / frame-width),
  ),
)
