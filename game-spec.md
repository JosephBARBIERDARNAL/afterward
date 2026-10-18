# Afterward game specifications

## Overview

- Afterward is a single-player action 2D, roguelike pixel-art platformer game where player has to clean 100 rooms full of enemies
- Each room is harder and harder because of 3 reasons: more enemies, more powerful and different kind of enemies
- After cleaning a room, player can choose one bonus among two
- Every 10th room, there is a boss and the environment changes on the next room
- The player enters a surreal world, fights through a 100-room run in ten themed chapters, creates a temporary build, and attempts to defeat the boss at the end of each chapter. The final boss waits in room 100.
- Death ends the run and destroys the temporary build. Nothing earned or chosen in one run carries into another: there is no currency, permanent upgrade tree, unlock, or account progression.
- Early runs will often end before the player reaches later chapters because the player is weak and inexperienced.
- Death immediately ends the run.

## Player

- There is one protagonist
- Character creation is not included
- All player assets (png and aseprite files) live in `src/assets/player/`

## Combats

- Player has a sword that does range damage (all enemies in a 150° range in front) and a gun that shoots
- Default sword damage is 5, default gun damage is 15
- Combat must be fast, mobility-heavy, skill-based
- Hitting an enemy should show a brief floating damage number whenever the player damages an enemy or takes damage. Damage dealt appears over the enemy; damage received appears over the player in bold red. It uses quick pop, upward drift, and fade so the numbers are easy to notice without lingering over the fight.
- Every move (sword attack, gun, dash, etc) should have its own dedicated animation

## Health

- Health persists between rooms during a run.
- Health is an horizontal bar at the top left of the screen, starting with 100/100
- Enemies health appear above its head in red, disappearing when it dies. An enemy with 1/3 of its life means 1/3 of the bar is red and the rest is black
- Healing is scarce enough that taking damage matters.

## Rooms

## Bonuses

## Assets

- All game assets live in `src/assets/`. Current asset directory should contain enough material to get started, but more should be added throught the development.
- Player assets are in `src/assets/player/`
- Ennemies assets are in `src/assets/enemies/`. Each ennemy kind has its own subfolder: `src/assets/enemies/type1/`, `src/assets/enemies/type2`, etc. Boss assets live in `src/assets/enemies/boss1`, etc.
- Background assets are in `src/assets/background/`
- Assets are available as PNG files, PSD files or aseprite.

## Controls

Default:

- arrows or ZQSD — movement;
- J — sword;
- K — shoot a bullet;
- Space — dash;
- E — interact with an element (get bonus, open door, etc).
- Escape — open or close the pause menu, which offers Continue (stop the pause and continue the run), Restart (restart the run from the beginning), Settings (see below) and Leave (go to the main menu that shows before a run).

The settings should let player control

- sound level of music and game sounds (both default to 100, which is max value)
- customize controls: every control can be remmapable to a different one

## Technical requirements

- Bevy game engine (Rust): https://bevy.org/
- Targets are MacOS, Windows and Linux (CI runs tests for all of them)
