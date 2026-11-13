// The hero, shared by every sheet of art/player/sheets/. Each sheet is a row
// of 128 x 128 frames drawn as text art, one character per pixel. A sheet only
// says which part of the body a pixel belongs to; how that part is coloured
// and lit is decided here, so redesigning the hero is changing `body`, and
// every clip follows.

#import "../_pixel.typ": canvas

#let frame-width = 128
#let frame-height = 128

// The hero's body, one ramp per part: the tone of an edge that faces the
// light (up, then left), the tone of the part itself, and the tone of an edge
// that faces away. A hooded figure in crimson over violet limbs, the far arm
// and leg sunk into the dark so the near ones read in front of them.
#let body = (
  "h": (rgb("#ff7a86"), rgb("#d42a44"), rgb("#8c1838")), // head and chest
  "m": (rgb("#c66ad8"), rgb("#8a34a8"), rgb("#541c74")), // near arm
  "l": (rgb("#e0508c"), rgb("#a82668"), rgb("#6a1650")), // near leg
  "d": (rgb("#8a3c9c"), rgb("#5c2278"), rgb("#381252")), // far leg
  "s": (rgb("#6e2c84"), rgb("#441a60"), rgb("#280c40")), // far arm
)

// What the hero's moves throw off: sparks, flames, glows and dust. These are
// the colours several sheets share; one a single sheet uses is declared in
// that sheet.
#let effects = (
  "W": rgb("#ffffff"), // white flash
  "w": rgb("#f1f2ff"), // cold white
  "c": rgb("#fcf7be"), // cream
  "y": rgb("#fff089"), // pale yellow
  "g": rgb("#f8f644"), // lemon
  "Y": rgb("#f8c53a"), // gold
  "o": rgb("#e88a36"), // amber
  "O": rgb("#f1641f"), // orange flame
  "a": rgb("#73efe8"), // aqua
  "b": rgb("#42bfe8"), // sky blue
  "n": rgb("#2b4e95"), // navy
)

// A sprite sheet: `frames` side by side, left to right. A frame is a raw
// block of text art, and every frame of a sheet is drawn in the same window
// of its 128 x 128 cell, whose top left corner is `at` (x, y). A character is
// a part of `body`, an effect colour, one of the sheet's `own` colours, or
// (such as ".") nothing at all. To add a frame, copy a block and redraw it;
// the sheet grows by one cell, and the clip's `frames` in
// src/assets/player/animations.yaml must follow.
#let sheet(at: (0, 0), own: (:), frames) = canvas(
  frame-width * frames.len(),
  frame-height,
  {
    let flat = effects + own
    for (index, frame) in frames.enumerate() {
      let rows = frame.text.split("\n").map(row => row.clusters())
      // Whether the body covers a pixel: its silhouette is what gets lit.
      let solid = rows.map(row => row.map(key => key in body))
      let empty = solid.first().map(_ => false)
      for (y, row) in rows.enumerate() {
        let above = if y == 0 { empty } else { solid.at(y - 1) }
        let below = solid.at(y + 1, default: empty)
        let here = solid.at(y)
        let last = row.len() - 1
        let fills = row
          .enumerate()
          .map(((x, key)) => {
            if here.at(x) {
              let tone = if not above.at(x) { 0 } else if not below.at(x) {
                2
              } else if x == 0 or not here.at(x - 1) { 0 } else if (
                x == last or not here.at(x + 1)
              ) { 2 } else { 1 }
              body.at(key).at(tone)
            } else { flat.at(key, default: none) }
          })
        // One rectangle per run of the same colour rather than per pixel.
        let start = 0
        for x in range(1, fills.len() + 1) {
          if x == fills.len() or fills.at(x) != fills.at(start) {
            if fills.at(start) != none {
              place(
                dx: (index * frame-width + at.at(0) + start) * 1pt,
                dy: (at.at(1) + y) * 1pt,
                rect(
                  width: (x - start) * 1pt,
                  height: 1pt,
                  fill: fills.at(start),
                ),
              )
            }
            start = x
          }
        }
      }
    }
  },
)
