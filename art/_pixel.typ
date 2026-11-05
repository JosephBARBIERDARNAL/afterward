// Pixel-art helpers. Each texture page is exactly its size in points and is
// rendered at 72 ppi (`just art`), so 1 pt is one image pixel and every
// pixel is a grid-aligned square: no anti-aliasing, exact colors.

// A page sized `width` x `height` pixels, with a transparent background.
#let canvas(width, height, body) = {
  set page(width: width * 1pt, height: height * 1pt, margin: 0pt, fill: none)
  body
}

// Draws text art: one string per row, one character per pixel, each
// character looked up in `palette` (a dictionary of colors). Characters
// missing from the palette (such as ".") stay transparent.
#let pixels(rows, palette) = canvas(rows.first().len(), rows.len(), {
  for (y, row) in rows.enumerate() {
    for (x, key) in row.clusters().enumerate() {
      if key in palette {
        place(dx: x * 1pt, dy: y * 1pt, rect(width: 1pt, height: 1pt, fill: palette.at(key)))
      }
    }
  }
})

// Draws a `width` x `height` image whose color at each pixel is `color-at(x, y)`
// (a palette key, or none to leave the pixel transparent).
#let pixels-by(width, height, palette, color-at) = pixels(
  range(height).map(y => range(width).map(x => {
    let key = color-at(x, y)
    if key == none { "." } else { key }
  }).join()),
  palette,
)
