# Afterward implementation tracker

## Data foundation

- [x] Author room YAML for rooms 1–10 with fixed layouts and enemy rosters.
- [x] Add centralized Chapter 1 enemy defaults and the shared forest background profile.
- [x] Centralize player, viewport, room-flow, and pickup tuning.
- [x] Define the 20 unique bonus offers and their category presentation data.
- [x] Add typed loading and validation for all game data.
- [x] Verify every configured sprite atlas frame count and height against its PNG header.

## Playable game

- [x] Create the Bevy 0.20 app, fixed 16:9 viewport, fullscreen/window settings, and main menu.
- [x] Implement player movement, animations, sword/gun combat, health, damage feedback, and pickups.
- [x] Implement Chapter 1 enemy behavior and the Warrior boss encounter.
- [x] Implement room transitions, clear animation, bonus offers/rerolls, and run/death flow.
- [x] Add pause/restart/settings UI, fullscreen switching, and persistent music/effects volume preferences.
- [x] Add replaceable placeholder sound events and connect volume preferences to audio playback.

## Completion checks

- [ ] Play through rooms 1–10 in a running build and visually verify each room, bonus choice, transition, and the Warrior encounter. So far, startup, game-data loading, and sprite-atlas dimensions have been verified.
- [x] Run formatting, Clippy, the Rust test suite, and the native build; fix all resulting issues.

## Remaining specification gaps

- [ ] Decide how the Swordsman should move. The supplied assets have no walk sheet, so the current animation falls back to idle while it moves.
- [ ] Decide how to add room-specific decorative props. All ten decoration lists are empty, and the supplied environment art is made of full-width parallax layers rather than standalone props.
- [x] Add a visible door-opening / room-transition animation. The door narrows as the screen fades to black, the next room loads under the overlay, and the new room fades in.
- [ ] Review audio against the full sound specification. Placeholder cues are in place, but most enemy types still share generic action sounds and footsteps do not vary by surface.
