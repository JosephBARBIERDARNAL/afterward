// The nearest blocks: the strongest silhouette, almost entirely dark, so the
// playable layer always reads in front of them.
#import "../_bands.typ": band

#band(
  576,
  324,
  (
    "m": rgb("#0e0d16"),
    "l": rgb("#13121d"),
    "d": rgb("#0a0910"),
    "c": rgb("#2f7083"),
    "g": rgb("#5e2150"),
    "e": rgb("#331134"),
  ),
  seed: 157,
  waterline: 242,
  window-chance: 5,
  min-width: 16,
  max-width: 36,
  min-height: 36,
  max-height: 92,
)
