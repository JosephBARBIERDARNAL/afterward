// The hero, shared by every sheet of art/player/sheets/. Each sheet is a row
// of 128 x 128 frames drawn as text art, one character per pixel. A sheet only
// says which part of the body a pixel belongs to; how that part is coloured
// and bordered is decided here, so redesigning the hero is changing `body`, and
// every clip follows.

#import "../_pixel.typ": canvas

#let frame-width = 128
#let frame-height = 128

// The hero's body, one pair of tones per part: the part itself, and the
// shade it takes just inside an edge facing down or right. A figure in white,
// the far arm and leg in grey so the near ones read in front of them.
#let body = (
  "h": (rgb("#ffffff"), rgb("#d9dbe3")), // head and chest
  "m": (rgb("#f6f7fa"), rgb("#d0d2db")), // near arm
  "l": (rgb("#eff0f5"), rgb("#c9cbd5")), // near leg
  "d": (rgb("#b3b6c1"), rgb("#9497a4")), // far leg
  "s": (rgb("#a5a8b4"), rgb("#878a98")), // far arm
)

// The border drawn around the figure, and between a part and one drawn in
// front of it.
#let border = rgb("#34363e")

// Which part stands in front of which, from the furthest: a part is bordered
// where it meets one that stands nearer.
#let depth = ("s": 0, "d": 1, "h": 2, "l": 3, "m": 4)

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
      let key-at(x, y) = if x < 0 or y < 0 { "." } else {
        rows.at(y, default: ()).at(x, default: ".")
      }
      // A body pixel is border where a side of it meets nothing or a nearer
      // part.
      let is-border(x, y) = {
        let key = key-at(x, y)
        ((-1, 0), (1, 0), (0, -1), (0, 1)).any(((dx, dy)) => {
          let next = key-at(x + dx, y + dy)
          not (next in body) or depth.at(next) > depth.at(key)
        })
      }
      let borders = rows
        .enumerate()
        .map(((y, row)) => row
          .enumerate()
          .map(((x, key)) => key in body and is-border(x, y)))
      for (y, row) in rows.enumerate() {
        let fills = row
          .enumerate()
          .map(((x, key)) => {
            if key in body {
              if borders.at(y).at(x) { border } else {
                // Shaded just inside a border below or to the right.
                let shaded = (
                  borders.at(y + 1, default: ()).at(x, default: false)
                    or row.len() > x + 1 and borders.at(y).at(x + 1)
                )
                body.at(key).at(if shaded { 1 } else { 0 })
              }
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
