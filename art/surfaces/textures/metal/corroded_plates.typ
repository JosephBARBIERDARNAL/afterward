// Riveted plates in corroded colors, with darker streaks running down from the
// top rivets.
#import "../../../_pixel.typ": pixels-by
#import "../../../_palettes.typ": corrosion
#import "_metal.typ": plate

#pixels-by(
  16,
  16,
  corrosion,
  (x, y) => {
    let streak = (
      (x == 2 and y >= 4 and y <= 8)
        or (x == 12 and y >= 4 and y <= 6)
        or (x == 7 and y >= 9 and y <= 11)
    )
    if streak { "d" } else { plate(x, y) }
  },
)
