// The drowned city's sky and the flood under it: the opaque backdrop every
// other band is drawn over. The sky darkens upward from a faint violet glow
// at the horizon, dithered between tones so the gradient stays pixel art;
// below the waterline the flood takes over, deepening toward the viewer.
#import "../../_pixel.typ": pixels-by

#let width = 576
#let height = 324
// Where the flood meets the sky. The far towers stand on this line.
#let waterline = 206

#let palette = (
  "0": rgb("#0b0a14"),
  "1": rgb("#0f0d1c"),
  "2": rgb("#141124"),
  "3": rgb("#19152b"),
  "4": rgb("#1e1832"),
  "5": rgb("#221b33"),
  "c": rgb("#4a1f4a"),
  "w": rgb("#2a0e2b"),
  "v": rgb("#220b24"),
  "u": rgb("#1a081c"),
  "t": rgb("#140616"),
)

// An ordered dither, so one tone bleeds into the next instead of banding.
#let bayer = (
  (0, 8, 2, 10),
  (12, 4, 14, 6),
  (3, 11, 1, 9),
  (15, 7, 13, 5),
)

// The tone at `level` (0 to `last`), dithered at (x, y) toward the next one.
#let dithered(x, y, level, last) = {
  let step = calc.min(calc.floor(level), last)
  let fraction = level - step
  let threshold = bayer.at(calc.rem(y, 4)).at(calc.rem(x, 4))
  if fraction * 16 > threshold and step < last { step + 1 } else { step }
}

#pixels-by(width, height, palette, (x, y) => {
  if y < waterline {
    str(dithered(x, y, y / waterline * 5, 5))
  } else if y == waterline {
    "c"
  } else {
    let depth = (y - waterline) / (height - waterline)
    ("w", "v", "u", "t").at(dithered(x, y, depth * 3, 3))
  }
})
