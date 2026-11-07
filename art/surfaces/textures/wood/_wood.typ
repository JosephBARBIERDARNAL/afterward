// Shared palettes and patterns of the wood texture family.
// Keys: "#" seam/outline, "d" dark grain, "m" body, "l" top light.

#let oak = (
  "#": rgb("#3b1f2b"),
  "d": rgb("#6b3a32"),
  "m": rgb("#93573f"),
  "l": rgb("#b8794f"),
)

#let ebony = (
  "#": rgb("#1a1424"),
  "d": rgb("#33232f"),
  "m": rgb("#4a3138"),
  "l": rgb("#664443"),
)

// Four horizontal planks with staggered butt joints. Seamless: every row
// wraps cleanly left to right and the last seam meets the first plank.
#let planks = (
  "llllllllll#lllll",
  "mmmmmmmmmd#mmmmm",
  "mmmddmmmmd#mmmmm",
  "################",
  "lll#llllllllllll",
  "mmd#mmmmmmmmdddm",
  "mmd#mmmmmmmmmmmm",
  "################",
  "lllllllllllll#ll",
  "mmmmmdmmmmmmd#mm",
  "mmmmddmmmmmmd#mm",
  "################",
  "lllllll#llllllll",
  "mmmmmmd#mmmmmmmm",
  "mddmmmd#mmmmmddm",
  "################",
)
