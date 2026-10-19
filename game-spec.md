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

## Player Movement and Combat

### Movement Specifications

`H` is the height of the player's collision box. Movement distances and speeds use `H` as their reference unit.

These are initial balance constants and should be easy to tune during iteration. Keep each value centralized in the game's player-tuning configuration rather than duplicating it across gameplay systems. The values below are the specification's source of truth.

| Constant                           | Value | Unit or notes                                                     |
| ---------------------------------- | ----- | ----------------------------------------------------------------- |
| `PLAYER_MAX_SPEED`                 | 6     | H/s                                                               |
| `PLAYER_ACCELERATION`              | 36    | H/s²                                                              |
| `PLAYER_DECELERATION`              | 48    | H/s²                                                              |
| `PLAYER_JUMP_HEIGHT`               | 2.25  | H                                                                 |
| `PLAYER_GRAVITY`                   | 32    | H/s² downward                                                     |
| `PLAYER_JUMP_IMPULSE`              | 12    | H/s upward; produces the specified jump height under this gravity |
| `PLAYER_JUMPS_PER_TAKEOFF`         | 1     | No double jump by default                                         |
| `BONUS_DOUBLE_JUMP_EXTRA_JUMPS`    | 1     | Additional mid-air jump; resets on landing                        |
| `BONUS_DOUBLE_JUMP_IMPULSE_FACTOR` | 0.8   | Multiplier of the normal jump impulse                             |
| `PLAYER_DASH_DISTANCE`             | 5     | H                                                                 |
| `PLAYER_DASH_DURATION`             | 0.2   | s; usable on the ground or in the air                             |
| `PLAYER_DASH_COOLDOWN`             | 0.8   | s from dash start                                                 |

### Weapon Specifications

The sword damages all enemies within a 150-degree arc in front of the player. The gun fires projectiles.

| Constant                                 | Value    | Unit or notes                                |
| ---------------------------------------- | -------- | -------------------------------------------- |
| `PLAYER_SWORD_DAMAGE`                    | 5        | Damage per hit                               |
| `PLAYER_SWORD_RANGE`                     | 1.5      | H from the player's collision-box center     |
| `PLAYER_SWORD_ATTACK_INTERVAL`           | 0.5      | s between attacks (2 attacks per second)     |
| `PLAYER_GUN_DAMAGE`                      | 15       | Damage per projectile                        |
| `PLAYER_GUN_FIRE_INTERVAL`               | 0.4      | s between shots (2.5 shots per second)       |
| `PLAYER_GUN_AMMO_MODE`                   | Infinite | No reload                                    |
| `PLAYER_PROJECTILE_SPEED`                | 18       | H/s                                          |
| `ENEMY_KNOCKBACK_FROM_SWORD`             | 2.5      | H/s horizontally away from the player        |
| `ENEMY_KNOCKBACK_FROM_BULLET`            | 1.2      | H/s horizontally away from the player        |
| `PLAYER_KNOCKBACK`                       | 3.5      | H/s horizontally away from the damage source |
| `PLAYER_DAMAGE_INVULNERABILITY_DURATION` | 0.55     | s after taking damage (33 frames at 60 FPS)  |
| `PLAYER_DASH_INVULNERABILITY_DURATION`   | 0.12     | s at the start of the dash                   |

### Combat Feedback and Room Flow

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

### Room Layout and Authoring

- All 100 rooms are authored before a build and played in fixed order. Room layouts and enemy rosters are not procedurally generated.
- Store each room in its own YAML file under `src/assets/rooms/`, named by its zero-padded room number (for example, `src/assets/rooms/room_001.yaml` through `src/assets/rooms/room_100.yaml`). These files are the source of truth for constructing the rooms.
- Each room file describes its ID, chapter, dimensions, player spawn, background profile, camera anchor, platforms and other collidable geometry, decorative props and their positions, enemy spawns and their configured attributes, and exit position.
- A room is 2 to 4 times the game viewport's width. Room geometry positions and dimensions use `H`, the player's collision-box height, as the reference unit.
- Each chapter has one shared background profile, including its backdrop and parallax layers. All ten rooms in that chapter reference the same profile. Room layouts, platform placement, enemy placement, and decorative props vary so rooms in a chapter do not all look alike.
- Keep decorations separate from platforms and other collidable geometry so visual placement does not implicitly make an object climbable.
- Each room's `enemy_spawns` list defines its fixed enemy roster. The number of enemies is the list length; do not duplicate it in a separately edited count. Each entry specifies an enemy type and position and resolves its configured attributes from YAML, using shared enemy-type defaults and any room-specific overrides. Enemy counts, types, positions, and configured attributes do not randomize at runtime; enemy AI still runs during play.
- Each room contains 5 to 15 enemy spawns and at least two enemy types, except boss rooms. Bosses are included in their room's enemy spawn list.
- Room YAML files do not define bonus offers. After a room is cleared, the game randomly selects two eligible bonuses at runtime; a bonus already owned by the player cannot be offered.

### Camera

- The camera follows the player's horizontal position and is clamped to the room's horizontal bounds.
- The camera's vertical position is fixed at the room's configured vertical anchor. It does not follow the player when they jump or fall.

### Room YAML Structure

The following is a schema example; each room file supplies concrete values and entries for its layout:

```yaml
room_id: null
chapter_id: null
size:
  width_viewports: null # 2 to 4
  height_h: null
background_profile: null # shared by all rooms in this chapter
player_spawn:
  x_h: null
  y_h: null
camera:
  follow_player_x: true
  fixed_y_h: null
platforms: [] # entries define position_h, size_h, and collision geometry
decorations: [] # entries define asset, position_h, and visual layer
enemy_spawns: [] # entries define type_id, position_h, and optional stat overrides
exit:
  x_h: null
  y_h: null
next_room_id: null # null only for room 100
```

- Moving through the exit door to the next room has its own transition animation.

## Enemies

- Enemy behavior is determined by enemy type; each type has a distinct behavior matching its appearance.
- Each room contains 5 to 15 enemies and at least two different enemy types.
- Individual enemies should be relatively easy to defeat, especially in the early rooms.
- Taking a hit interrupts an enemy's current action.
- Every enemy has idle, movement, hit-reaction, and death animations.

## Bonuses

- A bonus cannot be offered if the player already has it.
- After each room, two distinct eligible bonuses are selected randomly at runtime. Bonus choices are not stored in room YAML files.
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
- E: interact with an element, such as a bonus.
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
