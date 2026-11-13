// The hero, shared by every sheet of art/player/sheets/. Each sheet is a row
// of 128 x 128 frames drawn as text art, one character per pixel, looked up
// here: recolour the hero by changing `body`, and every clip follows.

#import "../_pixel.typ": canvas

#let frame-width = 128
#let frame-height = 128

// The hero's body: a pale head and chest over four blues, lightest to darkest.
#let body = (
  "h": rgb("#e6ecf5"), // head and chest
  "l": rgb("#4660a8"), // lit limbs
  "m": rgb("#3a509a"), // limbs
  "d": rgb("#2a3a78"), // shaded limbs
  "s": rgb("#24306a"), // deepest shade
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

#let palette = body + effects

// A sprite sheet: `frames` side by side, left to right. A frame is a raw
// block of text art, and every frame of a sheet is drawn in the same window
// of its 128 x 128 cell, whose top left corner is `at` (x, y). Characters
// missing from the palette (such as ".") stay transparent. To add a frame,
// copy a block and redraw it; the sheet grows by one cell, and the clip's
// `frames` in src/assets/player/animations.yaml must follow.
#let sheet(at: (0, 0), palette: palette, frames) = canvas(
  frame-width * frames.len(),
  frame-height,
  {
    for (index, frame) in frames.enumerate() {
      for (y, row) in frame.text.split("\n").enumerate() {
        // One rectangle per run of the same colour rather than per pixel.
        let keys = row.clusters()
        let start = 0
        for x in range(1, keys.len() + 1) {
          if x == keys.len() or keys.at(x) != keys.at(start) {
            if keys.at(start) in palette {
              place(
                dx: (index * frame-width + at.at(0) + start) * 1pt,
                dy: (at.at(1) + y) * 1pt,
                rect(
                  width: (x - start) * 1pt,
                  height: 1pt,
                  fill: palette.at(keys.at(start)),
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
