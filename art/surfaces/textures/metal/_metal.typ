// Patterns of the metal texture family, as `(x, y) => key` functions.

// One riveted plate per tile: lit top and left edges, a dark seam on the
// right and bottom, and a rivet with a shadow in each corner.
#let plate(x, y) = {
  let rivet = (x == 2 or x == 12) and (y == 2 or y == 12)
  let rivet-shadow = (x == 3 or x == 13) and (y == 3 or y == 13)
  if x == 15 or y == 15 { "#" } else if rivet { "r" } else if rivet-shadow {
    "d"
  } else if x == 0 or y == 0 { "l" } else { "m" }
}
