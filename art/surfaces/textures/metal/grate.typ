// A grate: vertical bars every 4 px over a dark void, held by a crossbar.
#import "../../../_pixel.typ": pixels-by
#import "../../../_palettes.typ": steel

#pixels-by(
  16,
  16,
  steel,
  (x, y) => {
    let u = calc.rem(x, 4)
    if y == 7 { "l" } else if y == 8 { "d" } else if u == 0 { "l" } else if (
      u == 1
    ) { "m" } else if u == 2 and y == 9 { "d" } else { "#" }
  },
)
