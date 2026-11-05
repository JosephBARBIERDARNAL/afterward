// The player's health bar frame, drawn nine-sliced: the outer 3 px are the
// frame (corners kept, edges stretched) and the dark middle is the trough
// the red fill covers. The game draws it at 2x (interface.rs).
#import "../_pixel.typ": pixels
#import "../_palettes.typ": steel

#pixels(
  (
    ".##########.",
    "#rllllllllr#",
    "#lm######md#",
    "#l#xxxxxx#d#",
    "#l#xxxxxx#d#",
    "#l#xxxxxx#d#",
    "#l#xxxxxx#d#",
    "#l#xxxxxx#d#",
    "#l#xxxxxx#d#",
    "#lm######md#",
    "#rddddddddr#",
    ".##########.",
  ),
  steel + ("x": rgb("#0d0f17")),
)
