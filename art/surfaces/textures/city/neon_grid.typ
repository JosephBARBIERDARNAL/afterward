// Floor panels lit from their seams: one dark plate per tile, a cyan line
// along its top and left, a brighter node where the lines cross, a halo
// fading into the plate, and a sunken bolt at its center.
#import "../../../_pixel.typ": pixels-by
#import "_city.typ": neon

#pixels-by(
  16,
  16,
  neon,
  (x, y) => {
    if (x, y) == (0, 0) { "r" } else if x == 0 or y == 0 {
      "l"
    } else if x == 1 or y == 1 or x == 15 or y == 15 { "m" } else if (
      x >= 7 and x <= 8 and y >= 7 and y <= 8
    ) { "#" } else { "d" }
  },
)
