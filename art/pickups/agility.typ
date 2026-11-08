// The agility bonus pickup: a wing, its shoulder at the upper right and four
// feathers fanning down to the left. A feather is a wedge of the fan around
// the shoulder: the angle picks which feather a pixel is on, the distance says
// how far along it, and each feather narrows to a point at its tip. The
// shoulder itself is a rounded blob covering where the feathers are rooted.
#import "../_pickups.typ": outlined

#let palette = (
  "#": rgb("#0f2316"), // rim
  "d": rgb("#2d6b3f"), // feather, trailing side
  "m": rgb("#4e9c5c"), // feather
  "l": rgb("#7ec97a"), // feather, leading side
  "r": rgb("#d2f2ae"), // leading edge of the wing
)

#let fan-x = 18.5
#let fan-y = 5.0
#let shoulder = (x: 17.5, y: 5.0, radius-x: 4.2, radius-y: 3.8)

// The fan starts `first-angle` degrees below straight left; each feather takes
// `feather-span` degrees, the last `feather-gap` of them being the gap to the
// next feather. The feathers reach `feather-length`, outermost first.
#let first-angle = 7.0
#let feather-span = 27.5
#let feather-gap = 6.0
#let feather-length = (18.5, 19.5, 17.5, 13.5)

#let feather(x, y) = {
  let left = fan-x - (x + 0.5)
  let down = (y + 0.5) - fan-y
  let angle = calc.atan2(left, down).deg() - first-angle
  let index = calc.floor(angle / feather-span)
  if angle < 0 or index >= feather-length.len() {
    return none
  }
  let length = feather-length.at(index)
  let distance = calc.sqrt(calc.pow(left, 2) + calc.pow(down, 2))
  // Feathers lie flush against each other near the shoulder and only part
  // over the outer half, where each also narrows to its point.
  let reach = distance / length
  let opened = calc.max(0.0, (reach - 0.5) / 0.5)
  let half-width = feather-span / 2 - 1.1 * opened - 1.6 * calc.pow(reach, 3)
  let offset = angle - (index + 0.5) * feather-span
  if distance > length or calc.abs(offset) > half-width {
    none
  } else if offset > half-width - 2.4 {
    "d"
  } else if index == 0 and offset < -half-width + 2.2 {
    "r"
  } else if offset < -half-width + 4.5 {
    "l"
  } else {
    "m"
  }
}

#outlined(palette, (x, y) => {
  let key = feather(x, y)
  if key != none {
    key
  } else if (
    calc.pow((x + 0.5 - shoulder.x) / shoulder.radius-x, 2)
      + calc.pow((y + 0.5 - shoulder.y) / shoulder.radius-y, 2)
      <= 1.0
  ) {
    if y + 1 < shoulder.y { "r" } else if x + y < 22 { "l" } else { "m" }
  } else {
    none
  }
})
