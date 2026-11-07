// Hull plating: one 16 px alloy plate, lit on its top and left, seamed on its
// bottom and right, with an inset channel across it and a bolt in each corner.
#import "../../../_pixel.typ": pixels-by
#import "_city.typ": alloy

#pixels-by(
  16,
  16,
  alloy,
  (x, y) => {
    let bolt = (x == 2 or x == 13) and (y == 2 or y == 13)
    let channel = (y == 7 or y == 8) and x >= 3 and x <= 12
    if x == 15 or y == 15 { "#" } else if bolt { "r" } else if channel {
      "d"
    } else if x == 0 or y == 0 { "l" } else { "m" }
  },
)
