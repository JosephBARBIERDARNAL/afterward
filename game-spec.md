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

Important: the following is the game specifications, focusing on the first chapter (first 10 rooms), not the entire game.

## Player

- The game has one protagonist. Character creation is not included.
- Player assets (PNG and Aseprite files) are stored in `src/assets/player/`.
- Every player action, including idle, has a dedicated animation.

## Player Movement and Combat

### Movement Specifications

`H` is the height of the player's collision box. Movement distances and speeds use `H` as their reference unit.
Ground and airborne horizontal movement share a maximum speed of 6 H/s.

These are initial balance constants and should be easy to tune during iteration. Keep each value centralized in the game's player-tuning configuration rather than duplicating it across gameplay systems. The values below are the specification's source of truth.

| Constant                           | Value | Unit or notes                                                     |
| ---------------------------------- | ----- | ----------------------------------------------------------------- |
| `PLAYER_MAX_SPEED`                 | 6     | H/s while grounded                                                |
| `PLAYER_AIR_MAX_SPEED`             | 6     | H/s while airborne; matches grounded maximum speed                |
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

The sword attacks horizontally in the direction the player faces, hitting all enemies in a broad forward-facing area. Its reach should let the player hit enemies without standing directly beside them. The gun fires projectiles horizontally in the direction the player faces (left or right); it cannot fire vertically. Player projectiles travel at most 10 H from the muzzle, then disappear.

| Constant                                 | Value    | Unit or notes                               |
| ---------------------------------------- | -------- | ------------------------------------------- |
| `PLAYER_SWORD_DAMAGE`                    | 5        | Damage per hit                              |
| `PLAYER_SWORD_RANGE`                     | 2.0      | H forward from the player's collision-box center |
| `PLAYER_SWORD_ATTACK_INTERVAL`           | 0.5      | s between attacks (2 attacks per second)    |
| `PLAYER_GUN_DAMAGE`                      | 15       | Damage per projectile                       |
| `PLAYER_GUN_FIRE_INTERVAL`               | 0.4      | s between shots (2.5 shots per second)      |
| `PLAYER_GUN_AMMO_MODE`                   | Infinite | No reload                                   |
| `PLAYER_PROJECTILE_SPEED`                | 18       | H/s                                         |
| `PLAYER_GUN_RANGE`                       | 10       | H from the muzzle; maximum projectile travel |
| `BOSS_KNOCKBACK_FROM_SWORD`              | 2.5      | H/s horizontally away from the player       |
| `BOSS_KNOCKBACK_FROM_BULLET`             | 1.2      | H/s horizontally away from the player       |
| `PLAYER_DAMAGE_INVULNERABILITY_DURATION` | 0.55     | s after taking damage (33 frames at 60 FPS) |
| `PLAYER_DASH_INVULNERABILITY_DURATION`   | 0.12     | s at the start of the dash                  |

### Combat Feedback and Room Flow

- Combat is fast, mobility-heavy, and skill-based.
- The player never receives knockback. Damage may trigger the usual feedback and hit reaction, but must not displace the player.
- Bosses are the only enemies affected by knockback. Sword hits and bullets may push bosses horizontally away from the player using the boss knockback constants above. Ordinary enemies are never displaced by knockback.
- Whenever the player deals or receives damage, show a brief floating damage number. Damage dealt appears over the enemy; damage received appears over the player in bold red. Numbers pop up, drift upward, and fade quickly so they remain legible without obscuring combat.
- Each player move, including sword attacks, shooting, and dashing, has a dedicated animation. Use placeholder sounds only for now (see Sound Design).
- After all enemies in a room are defeated, play a 0.5-second, full-screen room-clear animation, then show the bonus choices.
- After the player chooses a bonus, a door appears on the right side of the room. Touching it opens the door and starts the next room's combat.

## Health

- Health persists between rooms during a run.
- The player's health appears as a horizontal bar in the top-left corner, starting at 100/100.
- Enemy health appears above its head as a red bar with a black remainder and disappears when the enemy dies.
- Healing is scarce, so taking damage matters.
- Each enemy has a 4% base chance to drop a 10 HP health pickup.

## Rooms

### Display and Gameplay Viewport

- Support a resizable window and borderless fullscreen.
- `GAMEPLAY_VIEWPORT_ASPECT_RATIO`: `16:9`, fixed across displays.
- `GAMEPLAY_VIEWPORT_HEIGHT_H`: `9 H`.
- Derive viewport width from its height and aspect ratio (`16 H`). Resizing the window or entering fullscreen changes only the display scale, never the visible game world.
- Fit the viewport uniformly inside the available window using fractional scaling. Fill unused space with bars; do not stretch the image or reveal additional terrain on wider or taller displays. Integer pixel scaling is not required.
- Keep gameplay and HUD layout within this reference viewport. Physical display resolution and OS pixel scaling do not change world measurements in `H`.

### Room Layout and Authoring

- All 100 rooms are authored before a build and played in fixed order. Room layouts and enemy rosters are not procedurally generated.
- Store each room in its own YAML file under `src/assets/rooms/`, named by its zero-padded room number (for example, `src/assets/rooms/room_001.yaml` through `src/assets/rooms/room_100.yaml`). These files are the source of truth for constructing the rooms.
- Each room file describes its ID, chapter, dimensions, player spawn, background profile, camera anchor, platforms and other collidable geometry, decorative props and their positions, enemy spawns and their configured attributes, and exit position.
- Player and enemy spawn `position_h.y_h` values mark the feet baseline. The player and every enemy stand on the configured platform top, and sprites are bottom-aligned to that shared baseline even when their source frames have different heights.
- Platform layouts should form reachable routes: keep platform tops within the player's jump height of nearby surfaces, use short horizontal gaps, and include enough stepping points for the room's width. Place some enemy spawn feet on raised platform tops and the rest on the ground.
- A room is 2 to 4 times the fixed gameplay viewport's width (32 to 64 H at the initial viewport settings). Room geometry positions and dimensions use `H`, the player's collision-box height, as the reference unit.
- Each chapter has one shared background profile, including its backdrop and parallax layers. All ten rooms in that chapter reference the same profile. Room layouts, platform placement, enemy placement, and decorative props vary so rooms in a chapter do not all look alike.
- Room platform entries declare a `surface`: the `soil` ground collider follows the visible ground in the chapter background and controls footsteps without drawing a second soil slab; raised platforms use `stone` and are drawn separately. The material controls platform appearance and footsteps.
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
player_spawn: # y_h marks the player's feet baseline
  x_h: null
  y_h: null
camera:
  follow_player_x: true
  fixed_y_h: null
platforms: [] # entries define surface (soil or stone), position_h, size_h, and collision geometry
decorations: [] # entries define asset, position_h, and visual layer
enemy_spawns: [] # position_h.y_h marks enemy feet; entries define type_id and optional stat overrides
exit:
  x_h: null
  y_h: null
next_room_id: null # null only for room 100
```

- Moving through the exit door to the next room has its own transition animation.

## Enemies

- These first-pass Chapter 1 values are tuning defaults, not final balance. Store them centrally in `src/assets/enemies/enemy_types.yaml`; room YAML may override them.
- Each non-boss room contains 5 to 15 enemies and at least two different enemy types.
- `ENEMY_DETECTION_RANGE` defaults to 10 H horizontally. Enemies stay idle until the player enters this range, then retain the player as their target for the rest of the room—even if the player leaves detection range. They stay engaged until defeated or the room is cleared. Taking damage from the player also triggers combat immediately. Store this shared setting alongside the enemy profiles.
- In combat, melee enemies pursue the player and attack when in range; ranged enemies reposition to keep their preferred distance, then fire after a readable wind-up. Attacks deal damage only through their hitbox or projectile (no contact damage), and taking a hit interrupts the current action.
- Behaviors follow the existing animations: the Swordsman and Gorgon 1/3 close in for melee attacks, the Infantryman fires regular shots, the Destroyer uses slower charged shots, and Gorgon 2 uses its special ranged attack. The Gorgon variants share a visual style but vary in speed and attack weight.

| Type ID       |  HP | Attack                                                                    | Movement / behavior            |
| ------------- | --: | ------------------------------------------------------------------------- | ------------------------------ |
| `swordsman`   |  30 | 8 damage; melee, 1.2 H reach, 1.2 s cooldown, 0.35 s wind-up              | 2.5 H/s; pursues and strikes   |
| `infantryman` |  24 | 8 damage; projectile at 8 H/s, 1.5 s cooldown, 0.4 s wind-up              | 1.5 H/s; keeps 5–8 H distance  |
| `destroyer`   |  45 | 14 damage; charged projectile at 7 H/s, 2.2 s cooldown, 0.7 s wind-up     | 0.8 H/s; holds 6–10 H distance |
| `gorgon_1`    |  28 | 8 damage; lunge, 1.3 H reach, 1.4 s cooldown, 0.3 s wind-up               | 3.5 H/s; fast pursuer          |
| `gorgon_2`    |  36 | 10 damage; special projectile at 7 H/s, 2 s cooldown, 0.5 s wind-up       | 2.3 H/s; keeps 5–8 H distance  |
| `gorgon_3`    |  46 | 12 damage; heavy melee sweep, 1.6 H reach, 1.8 s cooldown, 0.55 s wind-up | 2.2 H/s; slow pursuer          |

- Chapter 1 boss `warrior`: 300 HP; moves at 3 H/s. Its three attacks are a two-hit melee combo (10 damage per hit), a telegraphed lunge (15 damage), and a jump slam (18 damage within 2 H of the landing point). Use 0.45–0.8 s wind-ups and 1.4–2.5 s cooldowns. At 50% HP, reduce cooldowns by 20%. These values are initial and tunable.
- Every enemy has idle, movement, hit-reaction, and death animations.

## Bonuses

- Bonuses are unique per run: a bonus cannot be offered to or acquired by a player who already owns it, even if multiple copies could otherwise be compatible.
- After each room, two distinct eligible bonuses are selected randomly at runtime. Bonus choices are not stored in room YAML files.

### Bonus Rerolls

- Start each run with `BONUS_REROLLS_STARTING_COUNT: 3`. Rerolls are a run-only resource, reset at the start of a new run, and cannot currently be replenished.
- The bonus screen blurs the room behind it. Each offer is shown in its own bordered box (border tinted by the bonus category) with left-aligned category, name, and description, and a key badge on its left showing its shortcut: A picks the first bonus, B picks the second.
- Before selecting a bonus, the player may activate the **Reroll** box, shown separately below the two offers, with its R key badge. Each use consumes one reroll and replaces both current offers with two distinct eligible bonuses different from the pair just replaced. Disable the button when no rerolls remain or fewer than two alternatives are eligible.
- Show the remaining count beside a small, minimal reroll-token icon. Play a short dedicated animation as the old pair is replaced by the new pair.
- Future bonuses may grant rerolls; adding those effects is out of scope for now.

- Every bonus has a name, description, and category.
- Categories: combat, defense, agility, intelligence, and combined (a mix of at least two other categories).
- Each category has its own icon and color: red for combat, blue for defense, green for agility, pale yellow for intelligence, and orange for combined.
- Initial bonus pool (20 bonuses):
  - **Honed Edge** (_Combat_): Increase sword damage by 20%.
  - **Heavy Caliber** (_Combat_): Increase gun damage by 20%.
  - **Quick Cut** (_Combat_): Increase sword attack speed by 15%.
  - **Velocity Rounds** (_Combat_): Increase projectile speed by 25%.
  - **Thick Skin** (_Defense_): Increase defense by 10%, causing attacks to deal 10% less damage.
  - **Room Ward** (_Defense_): Negate the first damage event in each room.
  - **Vitality** (_Defense_): Increase maximum health by 15 and restore 15 health when acquired.
  - **Scavenger** (_Defense_): Increase the chance of an enemy dropping a 10-HP health pickup by 25% relative to the base chance.
  - **Fleet Feet** (_Agility_): Increase movement speed by 10%.
  - **High Jump** (_Agility_): Increase jump height by 15%.
  - **Dash Recharge** (_Agility_): Reduce dash cooldown by 20% (from 0.8 s to 0.64 s at the current baseline).
  - **Double Jump** (_Agility_): Gain one additional mid-air jump at 80% of the normal jump impulse.
  - **Marked Prey** (_Intelligence_): A sword hit marks an enemy for 2 s; gun hits deal 15% more damage to that enemy while marked.
  - **Rhythm Training** (_Intelligence_): Each hit within 1.5 s of the previous hit increases both weapon attack speeds by 4%, up to 16%; the bonus resets after 1.5 s without a hit.
  - **Chain Reaction** (_Intelligence_): Killing an enemy increases damage by 15% for 2 s.
  - **Patient Aim** (_Intelligence_): After standing still for 0.5 s, gun damage increases by 15% until the player moves.
  - **Vampiric Edge** (_Combined: Combat + Defense_): Each enemy hit by the sword restores 1 health, up to 5 health per swing.
  - **Dash Strike** (_Combined: Combat + Agility_): A sword attack within 1 s after dashing deals 25% more damage.
  - **Aerial Hunter** (_Combined: Combat + Agility_): Deal 20% more gun damage while airborne.
  - **Sated Fury** (_Combined: Combat + Defense_): After collecting a healing pickup, increase both weapon attack speeds by 15% for 3 s.

## Assets

- Game assets are stored in `src/assets/`. Add more assets as development proceeds.
- Player assets are stored in `src/assets/player/`.
- Enemy assets are stored in `src/assets/enemies/`. Each enemy type has its own subfolder (for example, `src/assets/enemies/type1/` and `src/assets/enemies/type2/`). Boss assets are stored in corresponding boss folders (for example, `src/assets/enemies/boss1/`).
- Background assets are stored in `src/assets/background/`.
- Assets may be provided as PNG, PSD, or Aseprite files.

## Controls and Settings

Default controls:

- ZQSD (AZERTY) or arrow keys: move.
- Space: jump.
- Left Shift: dash.
- J: use the sword.
- K: fire the gun.
- E: interact with an element.
- A / B: choose the first / second bonus. R: reroll the bonus offers.
- Escape: open or close the pause menu. The menu offers Continue, Restart, Settings, and Leave. Leave returns to the main menu shown before a run.

Settings:

- Start in borderless fullscreen. Settings can switch to a resizable windowed mode or back; both preserve the fixed gameplay viewport.
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

Important for audio production: use temporary placeholders (simple generated tones/noise or silence) for required events. Keep placeholder calls easy to replace, and make sure missing audio files never prevent the game from running.
