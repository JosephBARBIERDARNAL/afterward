# Afterward Game Specifications

## Overview

- Afterward is a single-player, 2D action roguelike pixel-art platformer. The player fights through 100 rooms filled with enemies.
- Difficulty escalates across the run through enemy count, enemy strength, and enemy variety.
- The 100 rooms are divided into 10 chapters of 10 consecutive rooms each. Each chapter ends with a boss encounter in its tenth room (rooms 10, 20, ..., 100). Room 100 contains the final boss.
- Each chapter has its own environment. The environment changes when the player enters the next chapter.
- After clearing a room, the player chooses one of two bonuses.
- The player creates a temporary build during each run. Death ends the run and removes that build. Nothing carries over between runs: there is no currency, permanent upgrade tree, unlock, or account progression.
- Early runs may end before the player reaches later chapters because the player is weak and inexperienced.
- The game runs offline and locally. It has no account system or telemetry.

## Player

- The game has one protagonist. Character creation is not included.
- Player assets (PNG and Aseprite files) are stored in `src/assets/player/`.
- Every player action, including idle, has a dedicated animation.

## Combat

- The player has a sword and a gun. The sword damages all enemies within a 150-degree arc in front of the player. The gun fires bullets.
- Default sword damage is 5; default gun damage is 15.
- Combat is fast, mobility-heavy, and skill-based.
- Whenever the player deals or receives damage, show a brief floating damage number. Damage dealt appears over the enemy; damage received appears over the player in bold red. Numbers pop up, drift upward, and fade quickly so they remain legible without obscuring combat.
- Each player move, including sword attacks, shooting, and dashing, has a dedicated animation and corresponding sound effect.
- After all enemies in a room are defeated, play a 0.5-second, full-screen room-clear animation, then show the bonus choices.
- After the player chooses a bonus, a door appears on the right side of the room. Touching it opens the door and starts the next room's combat.

## Health

- Health persists between rooms during a run.
- The player's health appears as a horizontal bar in the top-left corner, starting at 100/100.
- Enemy health appears above its head as a red bar with a black remainder and disappears when the enemy dies.
- Healing is scarce, so taking damage matters.
- Each enemy has a 1-in-25 chance to drop a 10 HP health pickup.

## Rooms

- Each room is 2 to 4 times the game viewport's width.
- Each room has a distinct background with parallax layers that create a sense of depth.
- Platforms are placed throughout each room to support jumping and movement during combat.
- Moving through the exit door to the next room has its own transition animation.

## Enemies

- Enemy behavior is determined by enemy type; each type has a distinct behavior matching its appearance.
- Each room contains 5 to 15 enemies and at least two different enemy types.
- Individual enemies should be relatively easy to defeat, especially in the early rooms.
- Taking a hit interrupts an enemy's current action.
- Every enemy has idle, movement, hit-reaction, and death animations.

## Bonuses

- A bonus cannot be offered if the player already has it.
- Every bonus has a name, description, and category.
- Categories: combat, defense, agility, intelligence, and combined (a mix of at least two other categories).
- Each category has its own icon and color: red for combat, blue for defense, green for agility, pale yellow for intelligence, and orange for combined.
- The bonus catalog has not yet been defined.

## Assets

- Game assets are stored in `src/assets/`. Add more assets as development proceeds.
- Player assets are stored in `src/assets/player/`.
- Enemy assets are stored in `src/assets/enemies/`. Each enemy type has its own subfolder (for example, `src/assets/enemies/type1/` and `src/assets/enemies/type2/`). Boss assets are stored in corresponding boss folders (for example, `src/assets/enemies/boss1/`).
- Background assets are stored in `src/assets/background/`.
- Assets may be provided as PNG, PSD, or Aseprite files.

## Controls and Settings

Default controls:

- Arrow keys or ZQSD: move.
- J: use the sword.
- K: fire the gun.
- Space: dash.
- E: interact with an element, such as a bonus or door.
- Escape: open or close the pause menu. The menu offers Continue, Restart, Settings, and Leave. Leave returns to the main menu shown before a run.

Settings:

- Music and sound-effect volume have separate sliders, each ranging from 0 to 100 and defaulting to 100.
- Volume changes apply immediately and persist locally between sessions.
- Every control can be remapped.

## Sound Design

### Principles and Music

- The sound identity combines fast-paced combat with a surreal, mysterious atmosphere. It draws on retro pixel-art games while using modern audio effects.
- Sound effects give clear, immediate feedback without overwhelming the player during combat.
- The main soundtrack has exactly 10 chapter tracks: one primary background track for each chapter. Each track matches its chapter's environment, enemies, and atmosphere, and loops seamlessly.
- Chapter themes have energetic rhythms that support fast, mobile combat.
- The chapter track continues without restarting between rooms in the same chapter. Transitions between chapters are smooth.
- After a room is cleared, the music shifts to a calmer state.
- Combat, room-clear, and boss events may use short musical transitions or changes in intensity. Boss encounters, including the final boss, do not require additional full-length chapter tracks.
- Menus and game-state screens use suitable ambience or short audio cues.

### Gameplay Sound Effects

- Player actions have sound effects synchronized with their animations. Include movement and surface-specific footsteps, jumping, landing, sword swings and hits, gunshots and bullet impacts, dashing, taking damage, healing pickups, and death. Idle sounds may be subtle and should not become repetitive or distracting.
- Sword swings must sound different when they hit an enemy. Bullet impacts distinguish enemies from the environment. Damage and healing sounds must be easy to recognize.
- Each enemy type has a recognizable sound identity, with sounds for idle, movement, attack execution, taking damage, and death. Include attack-preparation cues where applicable. Enemy hit sounds clearly confirm a successful hit, dangerous attacks have clear warning cues, and death sounds are short enough to avoid clutter when several enemies die together.
- Stronger attacks have more powerful impact sounds.
- Bosses have distinctive sounds for their movement, attacks, phase changes when applicable, damage, and defeat. Boss attack cues must remain audible during combat.
- Bonuses that introduce new gameplay mechanics have appropriate sound effects.

### Ambience, Events, and Interface

- Each chapter has an ambient soundscape that supports its environment and surreal atmosphere without distracting from gameplay. Ambience loops seamlessly, varies subtly, and transitions smoothly when entering a new chapter.
- Play a distinct, satisfying cue for the 0.5-second room-clear animation. Provide distinct feedback for the exit door appearing and opening, bonus selection and navigation, health pickups, and other interactions.
- Menu navigation, selection confirmation, invalid actions, and opening or closing the pause menu have consistent, appropriately subtle feedback.
- Boss introductions use a short transition that builds tension; phase changes use clear audio cues. Defeating a boss, ending a run, restarting, and winning each have distinct cues. The final victory cue is unique.

### Mixing and Playback

- Keep music and sound effects on separate channels. Important gameplay cues, especially player damage and boss attack warnings, take priority over ambience and decorative sounds.
- Music may briefly lower in volume during important events, such as boss introductions, death, or victory.
- Limit or mix simultaneous identical sounds to prevent excessive volume. Repeated effects may vary slightly in pitch or volume to reduce repetition.
- Avoid clipping and excessive loudness. Stereo positioning may reflect sound location, but must not make important cues difficult to hear.
- Audio playback must remain performant in rooms with up to 15 enemies. Pause or attenuate gameplay audio while paused, keep menu sounds available, and restore gameplay audio smoothly on resume.
- Stop or transition audio correctly when restarting a run, dying, returning to the main menu, or changing chapters. No sound should continue after its associated game state ends.

### Audio Assets

- Store audio assets in `src/assets/audio/`, organized by category, with descriptive, consistent filenames. Name numbered variations with suffixes such as `sword_hit_01.wav` and `sword_hit_02.wav`.
- Choose audio formats appropriate to playback latency and quality. Preload frequently used sound effects, and ensure missing optional audio assets do not crash the game.
