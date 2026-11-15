// A flat ring kicked out under the feet, seen edge on, as a sprite sheet of
// 6 frames of 32 x 12 px, left to right: the air a mid-air jump pushes off
// (Double Jump, Kick Off, the jump Updraft gives back). Drawn white, to be
// tinted by the game.
#import "../_pixel.typ": pixels-by

#let width = 32
#let height = 12
#let frames = 6
#let palette = (
  "w": rgb("#ffffff"), // the near side of the ring
  "g": rgb("#9a9a9a"), // the far side, behind the feet
)

#let frame(index) = {
  let rx = 4 + index * 2.3
  let ry = 1.6 + index * 0.75
  let thickness = 0.42 - index * 0.04
  (x, y) => {
    let dx = (x + 0.5 - width / 2) / rx
    let dy = (y + 0.5 - height / 2) / ry
    let distance = calc.sqrt(dx * dx + dy * dy)
    if distance > 1 or distance < 1 - thickness {
      none
    } else if y + 0.5 < height / 2 {
      "g"
    } else {
      "w"
    }
  }
}

#pixels-by(width * frames, height, palette, (x, y) => {
  frame(calc.quo(x, width))(calc.rem(x, width), y)
})
