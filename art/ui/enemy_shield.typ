// The shield that protects a metal enemy from its first hit, as a sprite
// sheet of 6 frames of 21 x 16 px, left to right. Frame 0 is the intact
// shield shown above the enemy's health bar; frames 1-5 are the break, played
// once when a shot lands (combat/shield.rs): the shield flashes and cracks,
// then its two halves fly apart and crumble away.
#import "../_pixel.typ": pixels-by

#let frame-width = 21
#let frame-height = 16
#let frames = 6

#let palette = (
  "#": rgb("#2b2007"), // outline
  "d": rgb("#8a6a12"), // shadowed side
  "m": rgb("#d8aa22"), // body
  "l": rgb("#f4d75f"), // lit side
  "r": rgb("#fff6c0"), // highlight
)

// The intact shield, 11 x 12, lit from the top left.
#let rows = (
  "###########",
  "#rrllllmmm#",
  "#rrlllmmmd#",
  "#rllmmmmmd#",
  "#lllmmmmdd#",
  "#llmmmmmdd#",
  "#lmmmmmmdd#",
  ".#lmmmmmd#.",
  "..#lmmmd#..",
  "...#lmd#...",
  "....#m#....",
  ".....#.....",
)

#let shield-width = 11
#let shield-height = 12
// Where the shield sits in a frame, leaving room for the halves to fly out.
#let offset-x = 5
#let offset-y = 3

// One step brighter, for the flash on the frame the shield cracks.
#let brighter = ("#": "#", "d": "m", "m": "l", "l": "r", "r": "r")

// The column the crack runs down, per shield row: it wanders so the two
// halves break apart along a jagged edge rather than a straight cut.
#let crack-column = (5, 6, 5, 4, 5, 6, 5, 5, 5, 5, 5, 5)

// Per break frame: how far each half has flown sideways, how far it has
// risen, and how many of the twelve wear steps have crumbled away.
#let spread = (0, 0, 1, 2, 3, 4)
#let lift = (0, 0, 0, 1, 2, 3)
#let crumble = (0, 0, 0, 3, 7, 11)

// The order in which a pixel crumbles: the splinters along the crack go
// first and the outer rim last, with a little noise so the edge is ragged.
#let wear(u, v) = calc.min(
  11,
  calc.abs(u - 5) * 2 + calc.rem(u * 5 + v * 7, 3),
)

// The palette key of the intact shield at (u, v) of its own grid, or none
// outside it.
#let face(u, v) = {
  if u < 0 or v < 0 or u >= shield-width or v >= shield-height {
    none
  } else {
    let key = rows.at(v).clusters().at(u)
    if key == "." { none } else { key }
  }
}

// The half of the shield that covers (u, v) once the halves have flown apart,
// `side` being -1 for the left one and 1 for the right.
#let half(u, v, frame, side) = {
  let su = u - side * spread.at(frame)
  let sv = v + lift.at(frame)
  let past-crack = if sv >= 0 and sv < shield-height {
    if side == -1 { su < crack-column.at(sv) } else { su > crack-column.at(sv) }
  } else {
    false
  }
  if past-crack and wear(su, sv) >= crumble.at(frame) { face(su, sv) } else {
    none
  }
}

#pixels-by(frame-width * frames, frame-height, palette, (x, y) => {
  let frame = calc.div-euclid(x, frame-width)
  let u = calc.rem(x, frame-width) - offset-x
  let v = y - offset-y
  if frame == 0 {
    face(u, v)
  } else if frame == 1 {
    let key = face(u, v)
    if key == none { none } else if u == crack-column.at(v) { "#" } else {
      brighter.at(key)
    }
  } else {
    let left = half(u, v, frame, -1)
    if left != none { left } else { half(u, v, frame, 1) }
  }
})
