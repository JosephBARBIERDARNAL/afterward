// A carved band: moldings at the top and bottom and a row of sunken
// diamonds, one per tile, with a drill hole between neighbors. Meant for
// blocks 0.25 H tall (one tile), such as a lintel or a cornice.
#import "../../../_pixel.typ": pixels-by
#import "_carved.typ": sandstone

#pixels-by(
  16,
  16,
  sandstone,
  (x, y) => {
    // Twice the distance to the tile center, so the center falls between pixels.
    let diamond = calc.abs(2 * x - 15) + calc.abs(2 * y - 15)
    if y == 0 or y == 15 { "#" } else if y == 1 or y == 13 { "l" } else if (
      y == 2 or y == 14
    ) { "d" } else if diamond == 12 { "d" } else if diamond == 10 {
      "l"
    } else if diamond <= 2 { "d" } else if (
      (x == 0 or x == 15) and (y == 7 or y == 8)
    ) { "d" } else { "m" }
  },
)
