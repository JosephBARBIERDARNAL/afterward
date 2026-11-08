// The combined bonus pickup: two rings laced through each other, one amber and
// one copper. Each ring is an annulus; where they overlap, the amber one
// passes in front above the middle and behind below it, so they interlock.
#import "../_pickups.typ": in-ring, outlined

#let palette = (
  "#": rgb("#2a1206"), // rim
  "d": rgb("#9c4a18"), // amber ring, shadowed side
  "m": rgb("#e8843f"), // amber ring
  "l": rgb("#ffc078"), // amber ring, lit side
  "D": rgb("#6b4523"), // copper ring, shadowed side
  "M": rgb("#a8743c"), // copper ring
  "L": rgb("#d7a868"), // copper ring, lit side
)

#let inner-radius = 4.2
#let outer-radius = 7.2
#let first = (x: 8.0, y: 10.0)
#let second = (x: 15.5, y: 13.5)

// Which of a ring's three tones a pixel takes: lit at the upper left of the
// band, shadowed at the lower right.
#let tone(x, y, center, tones) = {
  let distance = calc.sqrt(
    calc.pow(x + 0.5 - center.x, 2) + calc.pow(y + 0.5 - center.y, 2),
  )
  // Inside the band, shade across it, then darken the ring's lower right.
  let outside = (x + 0.5 - center.x) - (y + 0.5 - center.y) > 0
  if distance > (inner-radius + outer-radius) / 2 {
    if outside { tones.at(2) } else { tones.at(1) }
  } else if outside { tones.at(1) } else { tones.at(0) }
}

#outlined(palette, (x, y) => {
  let on-first = in-ring(x, y, first.x, first.y, inner-radius, outer-radius)
  let on-second = in-ring(x, y, second.x, second.y, inner-radius, outer-radius)
  // Where the rings cross, the amber one is in front above the crossing and
  // the copper one in front below it.
  let amber-in-front = y + 0.5 < (first.y + second.y) / 2
  if on-first and (not on-second or amber-in-front) {
    tone(x, y, first, ("l", "m", "d"))
  } else if on-second {
    tone(x, y, second, ("L", "M", "D"))
  } else {
    none
  }
})
