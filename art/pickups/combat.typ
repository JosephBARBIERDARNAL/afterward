// The combat bonus pickup: two crossed swords, steel blades over crimson
// grips with gold furniture. Drawn in the blades' own coordinates, where
// `along` runs from the tip to the pommel and `across` measures the width of
// the blade, so the crossguard is simply a band of constant `along`.
#import "../_pickups.typ": outlined, size

#let palette = (
  "#": rgb("#1a0f14"), // rim
  "s": rgb("#64768d"), // blade, shadowed edge
  "b": rgb("#9fb3c9"), // blade
  "h": rgb("#e7f1fb"), // blade, lit edge
  "g": rgb("#7a1f23"), // grip
  "G": rgb("#c2423f"), // grip, lit edge
  "p": rgb("#e8b35a"), // guard and pommel
)

// One sword, pointing up-left: `along` grows from the tip towards the hilt
// and `across` crosses the blade from its lit edge to its shadowed one.
#let sword(along, across) = {
  if along >= 3 and along <= 27 and across >= 0 and across <= 2 {
    // The last few pixels of the point keep to the middle of the blade.
    if along <= 8 and across != 1 { none } else { ("h", "b", "s").at(across) }
  } else if along >= 28 and along <= 30 and across >= -6 and across <= 8 {
    "p"
  } else if along >= 31 and along <= 37 and across >= 0 and across <= 2 {
    if across == 0 { "G" } else { "g" }
  } else if along >= 38 and along <= 41 and across >= -1 and across <= 3 {
    "p"
  } else {
    none
  }
}

#outlined(palette, (x, y) => {
  // The sword laid down-right, drawn over the one laid down-left.
  let front = sword(x + y, x - y)
  if front != none { front } else {
    let mirrored = size - 1 - x
    sword(mirrored + y, mirrored - y)
  }
})
