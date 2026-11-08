// Shared drawing for the bonus pickup items (`art/pickups/`): one 24 x 24
// object per bonus category, lying on the ground once a room is cleared.
//
// Each source only describes the object's own pixels; `outlined` adds the
// dark 1 px rim that keeps it readable over any background, so every pickup
// shares the same silhouette treatment while keeping its own shape.
#import "_pixel.typ": pixels-by

// The size every pickup is drawn at.
#let size = 24

// The palette key `outlined` uses for the rim; sources reserve it.
#let outline = "#"

// Draws `body(x, y)` (a palette key, or none for a transparent pixel) with a
// rim on every transparent pixel that touches it. `body` is called outside
// the image too, so it must handle any coordinates.
#let outlined(palette, body) = pixels-by(size, size, palette, (x, y) => {
  let key = body(x, y)
  if key != none {
    key
  } else if (
    body(x - 1, y) != none
      or body(x + 1, y) != none
      or body(x, y - 1) != none
      or body(x, y + 1) != none
  ) {
    outline
  } else {
    none
  }
})

// Turns text art (one string per row, one character per pixel, "." for a
// transparent pixel) into a `body` function for `outlined`.
#let from-rows(rows) = (x, y) => {
  if y < 0 or y >= rows.len() {
    none
  } else {
    let row = rows.at(y).clusters()
    if x < 0 or x >= row.len() or row.at(x) == "." { none } else { row.at(x) }
  }
}

// Whether (x, y) lies in the ring of radii `inner`..`outer` around
// (center-x, center-y), measured on the pixel centers.
#let in-ring(x, y, center-x, center-y, inner, outer) = {
  let distance = calc.sqrt(
    calc.pow(x + 0.5 - center-x, 2) + calc.pow(y + 0.5 - center-y, 2),
  )
  distance >= inner and distance <= outer
}
