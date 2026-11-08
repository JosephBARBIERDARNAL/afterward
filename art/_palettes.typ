// Palettes shared by several families, mixed from the master palette in
// art/README.md. Keys: "#" outline/seam, "d" shadow, "m" body, "l" light,
// "r" rivet or brightest highlight.
//
// Nothing here goes brighter than #6A6288: the world is dark so the player
// and the action can be read against it.

// Cold chassis plate, the metal of a city nobody switched off.
#let steel = (
  "#": rgb("#0d0c16"),
  "d": rgb("#2a2740"),
  "m": rgb("#403b5c"),
  "l": rgb("#5c5680"),
  "r": rgb("#8a84a8"),
)

// What the flood did to metal. Rust is warm, and warm is reserved for danger
// and help, so what eats metal here is the water: the stain is wine, not
// oxide, and it marks how high the water has been.
#let corrosion = (
  "#": rgb("#150a12"),
  "d": rgb("#3a1524"),
  "m": rgb("#5c2436"),
  "l": rgb("#81414b"),
  "r": rgb("#a9646a"),
)
