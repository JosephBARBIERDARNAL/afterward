// A ring opening out and thinning as it goes, as a sprite sheet of 6 frames
// of 32 x 32 px, left to right. It is drawn white so the game can tint it in
// the colour of whatever it tells (`tells/`): the reach of Undertow's drag,
// Room Ward and Deflector spent, the charged blow ready with Overcharge.
#import "../_pixel.typ": pixels-by

#let size = 32
#let frames = 6
#let palette = (
  "w": rgb("#ffffff"), // the ring
  "g": rgb("#b8b8b8"), // its inner edge, a shade darker
)

#let frame(index) = {
  let radius = 4 + index * 2.3
  let thickness = 3.2 - index * 0.45
  (x, y) => {
    let dx = x + 0.5 - size / 2
    let dy = y + 0.5 - size / 2
    let distance = calc.sqrt(dx * dx + dy * dy)
    if distance > radius or distance < radius - thickness {
      none
    } else if distance < radius - thickness + 1 {
      "g"
    } else {
      "w"
    }
  }
}

#pixels-by(size * frames, size, palette, (x, y) => {
  frame(calc.quo(x, size))(calc.rem(x, size), y)
})
