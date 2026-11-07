// Diamond tread plate: short raised bars in a checkerboard of 8 px cells,
// alternating "/" and "\", each with a shadow on its lower right.
#import "../../../_pixel.typ": pixels-by
#import "../../../_palettes.typ": steel

#pixels-by(
  16,
  16,
  steel,
  (x, y) => {
    let (u, v) = (calc.rem(x, 8), calc.rem(y, 8))
    let rising = calc.rem(calc.quo(x, 8) + calc.quo(y, 8), 2) == 0
    let (bar, shadow) = if rising {
      (u + v == 7 and u >= 2 and u <= 5, u + v == 8 and u >= 3 and u <= 6)
    } else {
      (u == v and u >= 2 and u <= 5, u == v + 1 and u >= 3 and u <= 6)
    }
    if bar { "l" } else if shadow { "d" } else { "m" }
  },
)
