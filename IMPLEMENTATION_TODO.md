# Afterward implementation tracker

## Data foundation

- [x] Author room YAML for rooms 1–10 with fixed layouts and enemy rosters.
- [x] Add centralized Chapter 1 enemy defaults and the shared forest background profile.
- [x] Centralize player, viewport, room-flow, and pickup tuning.
- [x] Define the 20 unique bonus offers and their category presentation data.
- [x] Add typed loading and validation for all game data.

## Playable game

- [x] Create the Bevy 0.20 app, fixed 16:9 viewport, fullscreen/window settings, and main menu.
- [x] Implement player movement, animations, sword/gun combat, health, damage feedback, and pickups.
- [x] Implement Chapter 1 enemy behavior and the Warrior boss encounter.
- [x] Implement room transitions, clear animation, bonus offers/rerolls, and run/death flow.
- [x] Add pause/restart/settings UI, fullscreen switching, and persistent music/effects volume preferences.
- [x] Add replaceable placeholder sound events and connect volume preferences to audio playback.

## Completion checks

- [ ] Verify the first ten rooms in a running build using the supplied assets.
- [x] Run formatting, Clippy, the Rust test suite, and the native build; fix all resulting issues.
