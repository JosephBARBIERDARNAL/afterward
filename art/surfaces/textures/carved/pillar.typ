// A fluted column surface: a flute every 4 px (lit edge, body, shadowed
// groove) and a faint drum joint at the bottom of each tile.
#import "../../../_pixel.typ": pixels-by
#import "_carved.typ": sandstone

#pixels-by(
  16,
  16,
  sandstone,
  (x, y) => {
    let u = calc.rem(x, 4)
    if y == 15 and u != 3 { "d" } else if u == 0 { "l" } else if u == 3 {
      "#"
    } else if u == 2 { "d" } else { "m" }
  },
)
