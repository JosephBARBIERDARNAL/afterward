// Large dressed stone blocks, two 8 px courses per tile with offset joints.
// Each block is lit on its top and left and shaded on its bottom and right.
#import "../../../_pixel.typ": pixels-by
#import "_carved.typ": sandstone

#pixels-by(
  16,
  16,
  sandstone,
  (x, y) => {
    let v = calc.rem(y, 8)
    let joint = if y < 8 { 15 } else { 7 }
    let chisel = (x, y) in ((4, 3), (5, 3), (11, 12), (12, 12), (2, 10))
    if v == 7 or x == joint { "#" } else if (
      v == 0 or x == calc.rem(joint + 1, 16)
    ) { "l" } else if v == 6 or x == calc.rem(joint + 15, 16) or chisel {
      "d"
    } else { "m" }
  },
)
