// Shared drawing for the pickup items drawn in Typst (`art/pickups/`): 24 x 24
// objects lying on the ground, which today is the health drop alone. The bonus
// item is not one of them: it is the obelisk sheet `pickups/flying_obelisk.png`.
//
// A source only describes the object's own pixels; `outlined` adds the dark
// 1 px rim that keeps it readable over any background.
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
