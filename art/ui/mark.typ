// The reticle Marked Prey hangs over a marked enemy, 13 x 13 px: a ring
// with four ticks pointing in, drawn white with a dark rim so the game can
// tint it in the colour of the bonus while it stays readable on any wall.
#import "../_pixel.typ": pixels

#pixels(
  (
    "....##.##....",
    "...#ww#ww#...",
    "..#w#.w.#w#..",
    ".#w#..w..#w#.",
    "#w#.......#w#",
    "#w#.......#w#",
    "#wwww...wwww#",
    "#w#.......#w#",
    "#w#.......#w#",
    ".#w#..w..#w#.",
    "..#w#.w.#w#..",
    "...#ww#ww#...",
    "....##.##....",
  ),
  (
    "w": rgb("#ffffff"),
    "#": rgb("#1a1420"),
  ),
)
