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
- The protagonist has no defined gender or sex
- Character creation is not included
- Cosmetic customization is not required

## Combats

- Player has a sword that does range damage (all enemies in a 150° range in front) and a bow that shoots an arrow
- Default sword damage is 5, default bow damage is 15
- Combat must be fast, mobility-heavy, skill-based
- Hitting an enemy should show a brief floating damage number whenever the player damages an enemy or takes damage. Damage dealt appears over the enemy; damage received appears over the player in bold red. It uses quick pop, upward drift, and fade so the numbers are easy to notice without lingering over the fight.
- Every move (sword attack, arrow, dash, etc) should have its own dedicated animation

## Health

- Health persists between rooms during a run.
- Health is an horizontal bar at the top left of the screen, starting with 100/100
- Enemies health appear above its head in red, disappearing when it dies. An enemy with 1/3 of its life means 1/3 of the bar is red and the rest is black
- Healing is scarce enough that taking damage matters.

## Rooms

## Bonuses

## Controls

- arrows or ZQSD — movement;
- J — sword;
- K — shoot an arrow;
- Space — dash;
- E — interact with an element (get bonus, open door, etc).
- Escape — open or close the pause menu, which offers Continue (stop the pause and continue the run), Restart (restart the run from the beginning), and Leave (go to the main menu that shows before a run).

## Technical requirements

- Bevy game engine (Rust): https://bevy.org/
- Targets are MacOS, Windows and Linux (CI runs tests for all of them)
