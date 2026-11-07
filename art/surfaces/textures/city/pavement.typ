// Street pavement: one poured concrete slab per tile, lit along its top and
// left, joined by a dark seam, with aggregate speckled through the fill.
#import "../../../_pixel.typ": pixels-by
#import "_city.typ": concrete

#pixels-by(
  16,
  16,
  concrete,
  (x, y) => {
    let chip = (x, y) in ((5, 4), (11, 9), (3, 12))
    if x == 15 or y == 15 { "#" } else if x == 0 or y == 0 {
      "l"
    } else if chip { "r" } else if calc.rem(x * 5 + y * 3, 7) == 0 {
      "d"
    } else { "m" }
  },
)
