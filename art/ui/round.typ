// One ammo round of the HUD, in grays: the game tints it (loaded, reloading,
// spent) and draws it at 2x (ammo_hud.rs).
#import "../_pixel.typ": pixels

#pixels(
  (
    ".w.",
    "wwg",
    "wwg",
    "sss",
    "wwg",
    "wwg",
    "wgg",
  ),
  ("w": rgb("#ffffff"), "g": rgb("#b4b4b4"), "s": rgb("#6e6e6e")),
)
