// The intelligence bonus pickup: a lit crystal, a faceted amber stone with
// eight rays of light breaking out of it. The stone is a diamond whose facets
// are told apart by how far a pixel is from its two diagonals; the rays are
// spikes that taper as they leave it.
#import "../_pickups.typ": outlined

#let palette = (
  "#": rgb("#2a1f06"), // rim
  "d": rgb("#9a7a1c"), // facet, shadowed side
  "m": rgb("#d8b231"), // facet
  "l": rgb("#f2dc6c"), // facet, lit side
  "r": rgb("#fdf6c4"), // core and rays
)

#let center = 11.5
// The stone, as a diamond: half its width, and how tall it is for that width.
#let stone-radius = 6.5
#let stone-stretch = 1.25

// A ray every 45 degrees, reaching `ray-reach` from the middle of the stone.
#let ray-reach = 11.5
#let ray-width = 2.6

#let stone(across, down) = {
  // Inside the diamond, the facets are read off the two diagonals.
  let edge = calc.abs(across) + calc.abs(down) / stone-stretch
  if edge > stone-radius {
    none
  } else if edge < stone-radius * 0.4 {
    "r"
  } else if across + down < -stone-radius * 0.55 {
    "l"
  } else if across - down > stone-radius * 0.55 {
    "d"
  } else {
    "m"
  }
}

#let ray(across, down) = {
  // The straight rays run along the axes, the slanted ones along the diagonals.
  let diagonal = (across + down) / calc.sqrt(2)
  let other = (across - down) / calc.sqrt(2)
  let arms = (
    (across, down),
    (down, across),
    (diagonal, other),
    (other, diagonal),
  )
  let lit = arms.any(((along, aside)) => {
    let reach = calc.abs(along)
    (
      reach <= ray-reach
        and calc.abs(aside) <= ray-width * (1 - reach / ray-reach)
    )
  })
  if lit { "r" } else { none }
}

#outlined(palette, (x, y) => {
  let across = x + 0.5 - center
  let down = y + 0.5 - center
  let key = stone(across, down)
  if key != none { key } else { ray(across, down) }
})
