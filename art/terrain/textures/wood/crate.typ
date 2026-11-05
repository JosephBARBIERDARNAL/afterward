// A crate: dark outline, a two-pixel frame lit on its top and left, three
// vertical boards inside, and a diagonal brace with a shadow under it.
// Each tile is one whole crate, so a block shows a stack of crates.
#import "../../../_pixel.typ": pixels-by
#import "_wood.typ": oak

#pixels-by(16, 16, oak, (x, y) => {
  let edge = calc.max(calc.abs(2 * x - 15), calc.abs(2 * y - 15))
  if edge == 15 { "#" } else if edge >= 11 {
    if x == 14 or y == 14 { "m" } else { "l" }
  } else if x + y == 15 or x + y == 16 { "l" } else if x + y == 17 { "d" } else if (
    x == 6 or x == 9
  ) { "d" } else { "m" }
})
