// The drowned city's backdrop: flat silhouette bands of the master palette,
// each one nearer and darker, half-sunk in the flood. Shared by the band
// sources under background/, which differ only in palette and distance.

#import "../_pixel.typ": pixels-by

// A deterministic hash, so the same column always gives the same building.
#let rand(n, m) = calc.rem(
  calc.quo(calc.rem(n * 1103515245 + 12345, 2147483648), 65536),
  m,
)

// The height of the building standing at each column, 0 where there is none.
// The first and last columns are left open so the band tiles horizontally.
#let skyline(width, seed, min-width, max-width, min-height, max-height) = {
  let tops = ()
  let i = 0
  while tops.len() < width {
    let w = min-width + rand(seed + i * 17, max-width - min-width + 1)
    let h = min-height + rand(seed + i * 29 + 7, max-height - min-height + 1)
    for _ in range(w) { tops.push(h) }
    i += 1
  }
  tops = tops.slice(0, width)
  for x in range(6) {
    tops.at(x) = 0
    tops.at(width - 1 - x) = 0
  }
  tops
}

// One band: towers standing on `waterline`, lit windows at `window_chance`
// per cent of the window grid, and a dashed reflection smeared below them.
// Palette keys: "m" body, "l" lit edge, "d" shaded edge, "c" strip window,
// "g" sign window, "e" reflection.
#let band(
  width,
  height,
  palette,
  seed: 0,
  waterline: 236,
  window-chance: 20,
  min-width: 8,
  max-width: 16,
  min-height: 40,
  max-height: 120,
) = {
  let tops = skyline(width, seed, min-width, max-width, min-height, max-height)
  pixels-by(width, height, palette, (x, y) => {
    let own = tops.at(x)
    let top = waterline - own
    if own == 0 {
      none
    } else if y > waterline {
      // Wet ground, cheaply: a dashed smear of the tower on the flood under
      // it, fading out as it runs toward the viewer.
      let reflects = rand(x * 5 + seed, 100) < 45
      if (
        reflects and y <= waterline + 14 and calc.rem(y, 2) == 0
      ) { "e" } else { none }
    } else if y < top {
      none
    } else if y == top or (x > 0 and tops.at(x - 1) < own) {
      "l"
    } else if x + 1 < width and tops.at(x + 1) < own {
      "d"
    } else if (
      calc.rem(x, 4) == 1
        and calc.rem(y, 5) == 2
        and y > top + 2
        and rand(x * 131 + y * 7 + seed, 100) < window-chance
    ) {
      if rand(x * 7 + y * 3, 10) < 2 { "g" } else { "c" }
    } else {
      "m"
    }
  })
}
