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

- [x] Add headless integration coverage for rooms 1–10 using real room and enemy YAML, production player/enemy/platform spawning, gun/projectile/damage systems, unique bonus choices, and the Warrior encounter. A separate input-driven room test uses the default movement, sword, and gun controls to defeat the room 1 Swordsman without teleporting or resetting invulnerability. Focused tests also cover fullscreen remapping, repeated gunfire animation, received-damage styling, and the Swordsman's procedural gait. The full-chapter flow pilot still teleports and stays invulnerable; neither headless path verifies visuals.
- [x] Run the native visual walkthrough and inspect screenshots of all ten room combat scenes, bonus screens, door approaches, transitions, the Warrior intro/fight, and Chapter One completion. Captures are saved under `target/visual-walkthrough/`; the walkthrough clears enemies after each room screenshot to exercise the production room and screen flow.
- [ ] Manually play rooms 1–10 using player movement and combat controls. The headless test covers real combat systems, while the visual walkthrough automates room progression and does not verify combat feel or input timing.
- [x] Run formatting, Clippy, the Rust test suite, and the native build; fix all resulting issues.

## Remaining specification gaps

- [x] Give the Swordsman procedural movement animation because the supplied assets have no walk sheet.
- [x] Keep room decoration lists empty and use the supplied full-width parallax background layers as the room scenery.
- [x] Add a visible door-opening / room-transition animation. The door narrows as the screen fades to black, the next room loads under the overlay, and the new room fades in.
- [x] Add distinct generated placeholder cues for each Chapter 1 enemy's idle, movement, attack warning/execution, hurt, and death events.
- [x] Fade gameplay sound effects while paused or in menus while keeping interface and terminal cues audible.
- [ ] Review audio against the full sound specification. Chapter 1 now has a forest-specific generated placeholder loop; replace it with authored music when available, and add surface-specific player footsteps after room surface types are defined.
