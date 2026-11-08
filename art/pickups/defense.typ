// The defense bonus pickup: a heraldic shield, a steel rim around a deep blue
// face with a chevron across it and a glint in its upper-left corner.
#import "../_pickups.typ": outlined

#let palette = (
  "#": rgb("#0d1522"), // rim
  "y": rgb("#9fb0c6"), // steel edge
  "d": rgb("#23457a"), // face, shadowed side
  "m": rgb("#3a6fb5"), // face
  "l": rgb("#5d9ae0"), // face, lit side
  "r": rgb("#bcdcff"), // glint and chevron
)

// The shield's own grid: 20 wide, 22 tall, set in the 24 x 24 frame.
#let offset-x = 2
#let offset-y = 1
#let shield-width = 20

// Half the shield's width at row `v`, in half pixels: square shoulders down to
// the waist, then a taper to the point.
#let half-width = (
  20,
  20,
  20,
  20,
  20,
  20,
  20,
  20,
  20,
  20,
  20,
  20,
  20,
  20,
  18,
  16,
  14,
  12,
  10,
  8,
  6,
  3,
)

#let face(u, v) = {
  if v < 0 or v >= half-width.len() {
    false
  } else {
    calc.abs(2 * u - (shield-width - 1)) < half-width.at(v)
  }
}

#outlined(palette, (x, y) => {
  let u = x - offset-x
  let v = y - offset-y
  if not face(u, v) {
    none
  } else if (
    not face(u - 1, v)
      or not face(u + 1, v)
      or not face(u, v - 1)
      or not face(u, v + 1)
  ) {
    "y"
  } else if v >= 2 and v <= 4 and u - v >= 1 and u - v <= 2 {
    "r"
  } else if (
    v + calc.abs(2 * u - 19) / 2 >= 13 and v + calc.abs(2 * u - 19) / 2 <= 15
  ) {
    "r"
  } else if u <= 5 {
    "l"
  } else if u <= 12 {
    "m"
  } else {
    "d"
  }
})
