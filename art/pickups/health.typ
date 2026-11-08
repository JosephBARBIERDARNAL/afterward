// The health drop an enemy sometimes leaves behind: a plain medical cross,
// bevelled so it catches the light from the upper left, in the same green its
// glow pools on the ground. Two bars of constant thickness, with the four
// outer corners of each bar cut back by one pixel to soften the silhouette.
#import "../_pickups.typ": outlined

#let palette = (
  "#": rgb("#0b2416"), // rim
  "d": rgb("#1b8f4d"), // shadowed edge
  "m": rgb("#35c96a"), // face
  "l": rgb("#6ef394"), // lit edge
  "h": rgb("#d6ffe4"), // glint
)

// The cross's own grid, 20 x 20 set in the 24 x 24 frame.
#let span = 20
#let offset = 2
#let thickness = 8
#let edge = (span - thickness) / 2

// Whether (a, b) is one of the two pixels cutting an arm's outer corner:
// `a` runs along the bar, `b` across it.
#let cut(a, b) = (
  (a == 0 or a == span - 1) and (b == edge or b == edge + thickness - 1)
)

#let cross(u, v) = {
  if u < 0 or v < 0 or u >= span or v >= span {
    false
  } else if not (
    (u >= edge and u < edge + thickness) or (v >= edge and v < edge + thickness)
  ) {
    false
  } else {
    not cut(v, u) and not cut(u, v)
  }
}

#outlined(palette, (x, y) => {
  let u = x - offset
  let v = y - offset
  if not cross(u, v) {
    none
  } else if not cross(u, v - 1) or not cross(u - 1, v) {
    "l"
  } else if not cross(u, v + 1) or not cross(u + 1, v) {
    "d"
  } else if u >= 7 and u <= 8 and v >= 2 and v <= 3 {
    "h"
  } else {
    "m"
  }
})
