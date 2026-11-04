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
- The protagonist uses the Ozzbit Games "Male Hero Template" (free version: personal, non-commercial use only; credit "Ozzbit Games"). Its sheets live in `src/assets/player/sheets/`, and `src/assets/player/animations.yaml` lists every clip (frames, frame rate) and how the 128×128 frames map onto the collision box: the 32 px body is drawn 0.75 H tall (`body_height_h`), feet on the collision box's baseline.
- Every player action, including idle, has a dedicated animation: idle, walk (while accelerating or below 60% of maximum speed), run, jump (rising), fall (apex transition, then a falling loop), dash, wall slide, ledge climb, and one animation per sword-combo hit (the pack's `combo_1` slash, then its `combo_1_end` follow-through as the finisher). The pack has no dash, wall, or ledge art: the dash replays the run cycle at a much higher frame rate, the wall slide slows the falling loop, and the ledge climb replays the jump rise quickly.
- Prototype: `src/assets/player/model.yaml` can replace the sprite sheets with a 3D glTF model (`enabled: true`). An off-screen orthographic camera renders the model into a texture drawn as the player sprite, so it looks high definition against the pixel-art world; each player clip plays a mapped glTF animation, with one-shot actions fitted to the clip's duration. The sample model is "RobotExpressive" by Tomás Laulhé (CC0).

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
| `PLAYER_JUMP_HEIGHT`               | 2.925 | H                                                                 |
| `PLAYER_GRAVITY`                   | 32    | H/s² downward                                                     |
| `PLAYER_JUMP_IMPULSE`              | 13.68 | H/s upward; produces the specified jump height under this gravity |
| `PLAYER_JUMPS_PER_TAKEOFF`         | 1     | No double jump by default                                         |
| `PLAYER_JUMP_RELEASE_MULTIPLIER`   | 0.5   | Upward velocity kept when jump is released while rising           |
| `BONUS_DOUBLE_JUMP_EXTRA_JUMPS`    | 1     | Additional mid-air jump; resets on landing                        |
| `BONUS_DOUBLE_JUMP_IMPULSE_FACTOR` | 0.8   | Multiplier of the normal jump impulse                             |
| `PLAYER_DASH_DISTANCE`             | 5     | H                                                                 |
| `PLAYER_DASH_DURATION`             | 0.2   | s; usable on the ground or in the air                             |
| `PLAYER_DASH_COOLDOWN`             | 0.8   | s from dash start                                                 |
| `PLAYER_WALL_SLIDE_MAX_FALL_SPEED` | 4     | H/s while airborne, pressing into a solid wall                    |
| `PLAYER_WALL_JUMP_IMPULSE`         | 12.5  | H/s upward                                                        |
| `PLAYER_WALL_JUMP_KICK_SPEED`      | 7     | H/s horizontally away from the wall                               |
| `PLAYER_WALL_JUMP_CONTROL_LOCK`    | 0.16  | s of ignored horizontal input after a wall jump                   |
| `PLAYER_LEDGE_GRAB_REACH`          | 0.35  | H above the top of the body                                       |
| `PLAYER_LEDGE_CLIMB_DURATION`      | 0.16  | s                                                                 |
| `PLAYER_DROP_THROUGH_DURATION`     | 0.25  | s one-way platforms are ignored after Down + Jump                 |

### Walls, Ledges, and Platforms

- Rooms are built from solid terrain (floors, walls, ceilings, pillars) that blocks the player from every side, and one-way platforms that the player lands on, jumps through from below, and drops through with Down + Jump.
- Wall slide: while airborne and pressing into a solid wall (or the room edge), the player falls no faster than `PLAYER_WALL_SLIDE_MAX_FALL_SPEED`.
- Wall jump: pressing jump while airborne against a wall kicks the player up and away from it. Horizontal input is ignored for `PLAYER_WALL_JUMP_CONTROL_LOCK`. A wall jump uses up the take-off jump, so Double Jump still grants its mid-air jump afterwards. Releasing jump early cuts the rise, as for any jump.
- Ledge mantle: while airborne and moving toward a solid block whose top is above the feet and at most `PLAYER_LEDGE_GRAB_REACH` above the head, the player climbs onto it automatically over `PLAYER_LEDGE_CLIMB_DURATION`, provided there is room to stand on it.

### Weapon Specifications

The sword is the player's only weapon. It attacks horizontally in the direction the player faces, hitting all enemies in a broad forward-facing area. Its reach should let the player hit enemies without standing directly beside them.

Sword attacks form a combo. Each press of J starts a swing (a press made while the previous swing is still cooling down is buffered and fires as soon as the sword is ready). Combos have no timing window: spamming the sword attack chains it. Any press made before the sword is ready again continues the chain, and each further step multiplies the hit's damage by the next `PLAYER_SWORD_COMBO_DAMAGE_MULTIPLIERS` entry, so longer combos hit harder. The last step is the finisher: it ends the chain, and the next swing starts over at the first step. The chain also resets once the sword is ready with no press waiting, meaning the player stopped attacking. Each hit plays its own combo animation (`sword_combo` in `src/assets/player/animations.yaml`, one clip per multiplier), so the animation shows the player how far the combo has gone; there is no on-screen combo text.

| Constant                                 | Value    | Unit or notes                               |
| ---------------------------------------- | -------- | ------------------------------------------- |
| `PLAYER_SWORD_DAMAGE`                    | 25       | Damage per hit, before combo multipliers    |
| `PLAYER_SWORD_RANGE`                     | 3.0      | H forward from the player's collision-box center |
| `PLAYER_SWORD_ATTACK_INTERVAL`           | 0.5      | s between attacks (2 attacks per second)    |
| `PLAYER_SWORD_COMBO_DAMAGE_MULTIPLIERS`  | 1.0, 1.75 | Damage multiplier per combo hit; the last hit is the finisher |
| `BOSS_KNOCKBACK_FROM_SWORD`              | 2.5      | H/s horizontally away from the player       |
| `PLAYER_DAMAGE_INVULNERABILITY_DURATION` | 0.55     | s after taking damage (33 frames at 60 FPS) |
| `PLAYER_DASH_INVULNERABILITY_DURATION`   | 0.12     | s at the start of the dash                  |

### Combat Feedback and Room Flow

- Combat is fast, mobility-heavy, and skill-based.
- The player never receives knockback. Damage may trigger the usual feedback and hit reaction, but must not displace the player.
- Bosses are the only enemies affected by knockback. Sword hits may push bosses horizontally away from the player using the boss knockback constant above. Ordinary enemies are never displaced by knockback.
- Whenever the player deals or receives damage, show a brief floating damage number. Damage dealt appears over the enemy; damage received appears over the player in bold red. Numbers pop up, drift upward, and fade quickly so they remain legible without obscuring combat.
- Each player move, including sword attacks and dashing, has a dedicated animation. Use placeholder sounds only for now (see Sound Design).
- Every room, the first included, opens with an entrance sequence before combat starts. The player walks in from the left at walking pace for 0.5 s, fading in, and stops on the room's player spawn, then stays idle; input is locked for the whole sequence. Once the player stands idle, the enemies materialize one after another (each rises out of its spawn point from a shadow tint into full color, with a slight overshoot) and their health bars fade in with them, with a placeholder cue. The fight starts right after the last enemy has appeared; in the boss room the boss introduction follows the entrance. Timings live in `room_flow.room_intro` in `src/assets/config/gameplay.yaml`.
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
- Rooms are enclosed, multi-level spaces in the style of Dead Cells rather than open side-scrolling fields: solid floors, walls, and ceilings bound each room, with galleries, ledges, pillars, hanging rock, and plank platforms stacked at several heights.
- Each room file describes its ID, chapter, dimensions, player spawn, background profile, solid terrain, one-way platforms, decorative props and their positions, enemy spawns and their configured attributes, and exit position.
- Player and enemy spawn `position_h.y_h` values mark the feet baseline. The player and every enemy stand on the configured platform top, and sprites are bottom-aligned to that shared baseline even when their source frames have different heights.
- Terrain blocks and one-way platforms are rectangles given by their lower-left corner and size (`rect_h`). Solid `terrain` blocks stop bodies and enemy projectiles from every side; one-way `platforms` (at most 0.5 H thick) only hold up bodies landing on them, and projectiles fly through them.
- Layouts should form reachable routes. The ground floor runs continuously from the entrance doorway to the exit, with steps no higher than a jump. Upper levels step up 2 H at a time through planks and ledges so enemies can follow the player; higher ledges may require the player's mantle or a wall jump. Upper floors keep at least one open edge so enemies can come back down. Place some enemy spawn feet on raised tops and the rest on the ground.
- Each room has a doorway clear of solid terrain in its left wall where the player walks in, and the exit stands on the ground floor near the right wall.
- A room is 2 to 4 times the fixed gameplay viewport's width (32 to 64 H) and 1.5 to 3 times its height (13.5 to 27 H). These limits live in `room_size_limits` in `src/assets/config/gameplay.yaml`. Room geometry positions and dimensions use `H`, the player's collision-box height, as the reference unit.
- Each background profile defines a backdrop and parallax layers, and consecutive rooms share a profile. Layers scroll horizontally at their parallax rate and stay centered on the camera vertically, behind the room's drawn terrain. Chapter 1 uses `chapter_01_forest` for rooms 1–3, `chapter_01_hills` for rooms 4–6, `chapter_01_autumn_canopy` for rooms 7–8, and `chapter_01_autumn_sunbeams` for rooms 9–10. Room layouts, platform placement, enemy placement, and decorative props vary so rooms in a chapter do not all look alike. Each background profile also declares its texture `filtering`: `nearest` keeps upscaled pixel art crisp on large displays instead of blurring it, `linear` smooths it.
- Terrain and platform entries declare a `surface`: solid terrain uses `soil` (earth with a grass edge) or `stone`, and one-way platforms use `wood` planks. The material controls the block's (placeholder) appearance and footsteps. All terrain is drawn by the game; it does not rely on ground painted into the background.
- Keep decorations separate from platforms and other collidable geometry so visual placement does not implicitly make an object climbable.
- Each room's `enemy_spawns` list defines its fixed enemy roster. The number of enemies is the list length; do not duplicate it in a separately edited count. Each entry specifies an enemy type and position and resolves its configured attributes from YAML, using shared enemy-type defaults and any room-specific overrides. Enemy counts, types, positions, and configured attributes do not randomize at runtime; enemy AI still runs during play.
- Each room contains 5 to 15 enemy spawns and at least two enemy types, except boss rooms. Bosses are included in their room's enemy spawn list.
- Room YAML files do not define bonus offers. After a room is cleared, the game randomly selects two eligible bonuses at runtime; a bonus already owned by the player cannot be offered.

### Camera

- The camera follows the player on both axes and is clamped so it never shows anything outside the room.
- Vertically, the camera moves only once the player is more than `camera.vertical_dead_zone_h` (1.5 H) above or below the screen center, so ordinary jumps do not shake the view.

### Room YAML Structure

The following is a schema example; each room file supplies concrete values and entries for its layout:

```yaml
room_id: null
chapter_id: null
size:
  width_viewports: null # 2 to 4
  height_h: null # 1.5 to 3 viewport heights
background_profile: null # shared with neighboring rooms in this chapter
player_spawn: # y_h marks the player's feet baseline; a doorway leads in from the left
  x_h: null
  y_h: null
terrain: [] # solid blocks: surface (soil or stone) and rect_h { x, y, width, height }, lower-left corner
platforms: [] # one-way platforms: surface (wood) and rect_h, at most 0.5 H thick
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

- Chapter 1 boss `evil_wizard`: 300 HP; moves at 3 H/s; jumps 2.75 H high. Its three attacks are a two-hit melee combo (10 damage per hit, Attack 1 animation), a telegraphed lunge (15 damage, Attack 2 animation), and a jump slam: after a crouched wind-up it leaps toward the player and deals 18 damage within 2 H of its landing point on touchdown (Jump and Fall animations). Use 0.45–0.8 s wind-ups and 1.4–2.5 s cooldowns. At 50% HP, reduce cooldowns by 20%. These values are initial and tunable.
- Every enemy has idle, movement, hit-reaction, and death animations.
- Every enemy, bosses included, can jump. Jumps follow the player's gravity and are tuned in `enemy_types.yaml`: a shared `ENEMY_JUMP_HEIGHT` (2.25 H) that each type may override, a 5 H/s horizontal air speed, and a 0.6 s cooldown between jumps. Enemies jump only when it brings them toward the player: the player counts as on another level when their grounded feet differ from the enemy's by more than 1 H; a player in mid-jump does not trigger level changes.
  - When the player stands higher, an enemy targets the reachable platform closest to the player and no higher than them. It walks to a take-off point under or beside that platform, then jumps onto it, repeating step by step.
  - When the player is at its level, an enemy reaching the ledge it walks toward jumps the gap to the next platform if it can make it.
  - When the player is lower, enemies drop through a one-way platform they stand on, or walk off the nearest ledge that is not walled off and fall; they never jump down.
  - Solid terrain blocks enemies like the player. An enemy walking into a wall low enough to jump hops onto it, and an enemy leaping into a ledge within reach climbs onto it like the player's mantle.
  - With no reachable route up, an enemy waits under the player instead of jittering. Enemies attack only when the player is on their level. Enemies without jump art hold their movement frame while airborne.

## Bonuses

- Bonuses are unique per run: a bonus cannot be offered to or acquired by a player who already owns it, even if multiple copies could otherwise be compatible.
- After each room, two distinct eligible bonuses are selected randomly at runtime. Bonus choices are not stored in room YAML files.

### Bonus Rerolls

- Start each run with `BONUS_REROLLS_STARTING_COUNT: 3`. Rerolls are a run-only resource, reset at the start of a new run, and cannot currently be replenished.
- The bonus screen blurs the room behind it. Each offer is shown in its own bordered box (border tinted by the bonus category) with left-aligned category, name, and description, and a key badge on its left showing its shortcut: 1 picks the first bonus, 2 picks the second.
- The offers and the Reroll box can also be clicked with the mouse, or selected with the Up/Down arrow keys and confirmed with Enter (or E). The selected or hovered box is lightly highlighted; the first offer is selected when the screen opens.
- Before selecting a bonus, the player may activate the **Reroll** box, shown separately below the two offers, with its R key badge. Each use consumes one reroll and replaces both current offers with two distinct eligible bonuses different from the pair just replaced. Disable the button when no rerolls remain or fewer than two alternatives are eligible.
- Show the remaining count beside a small, minimal reroll-token icon. Play a short dedicated animation as the old pair is replaced by the new pair.
- Future bonuses may grant rerolls; adding those effects is out of scope for now.

- Every bonus has a name, description, and category.
- Categories: combat, defense, agility, intelligence, and combined (a mix of at least two other categories).
- Each category has its own icon and color: red for combat, blue for defense, green for agility, pale yellow for intelligence, and orange for combined.
- Initial bonus pool (20 bonuses):
  - **Honed Edge** (_Combat_): Increase sword damage by 20%.
  - **Keen Finisher** (_Combat_): The final hit of a sword combo deals 30% more damage.
  - **Quick Cut** (_Combat_): Increase sword attack speed by 15%.
  - **Long Reach** (_Combat_): Increase sword range by 20%.
  - **Thick Skin** (_Defense_): Increase defense by 10%, causing attacks to deal 10% less damage.
  - **Room Ward** (_Defense_): Negate the first damage event in each room.
  - **Vitality** (_Defense_): Increase maximum health by 15 and restore 15 health when acquired.
  - **Scavenger** (_Defense_): Increase the chance of an enemy dropping a 10-HP health pickup by 25% relative to the base chance.
  - **Fleet Feet** (_Agility_): Increase movement speed by 10%.
  - **High Jump** (_Agility_): Increase jump height by 15%.
  - **Dash Recharge** (_Agility_): Reduce dash cooldown by 20% (from 0.8 s to 0.64 s at the current baseline).
  - **Double Jump** (_Agility_): Gain one additional mid-air jump at 80% of the normal jump impulse.
  - **Marked Prey** (_Intelligence_): A sword hit marks an enemy for 2 s; further sword hits deal 15% more damage to that enemy while marked.
  - **Rhythm Training** (_Intelligence_): Each hit within 1.5 s of the previous hit increases sword attack speed by 4%, up to 16%; the bonus resets after 1.5 s without a hit.
  - **Chain Reaction** (_Intelligence_): Killing an enemy increases damage by 15% for 2 s.
  - **Combo Flow** (_Intelligence_): Follow-up sword combo swings come 25% faster.
  - **Vampiric Edge** (_Combined: Combat + Defense_): Each enemy hit by the sword restores 1 health, up to 5 health per swing.
  - **Dash Strike** (_Combined: Combat + Agility_): A sword attack within 1 s after dashing deals 25% more damage.
  - **Aerial Edge** (_Combined: Combat + Agility_): Deal 20% more sword damage while airborne.
  - **Sated Fury** (_Combined: Combat + Defense_): After collecting a healing pickup, increase sword attack speed by 15% for 3 s.

## Assets

- Game assets are stored in `src/assets/`. Add more assets as development proceeds.
- Player assets are stored in `src/assets/player/`.
- Enemy assets are stored in `src/assets/enemies/`. Each enemy type has its own subfolder (for example, `src/assets/enemies/type1/` and `src/assets/enemies/type2/`). Boss assets are stored in corresponding boss folders (for example, `src/assets/enemies/boss1/` holds the Chapter 1 Evil Wizard).
- Background assets are stored in `src/assets/background/`.
- Assets may be provided as PNG, PSD, or Aseprite files.

## Controls and Settings

Default controls:

- ZQSD (AZERTY) or arrow keys: move. Down (S or Down arrow) + Space drops through a one-way platform.
- Space: jump; against a wall in the air, wall jump. Jump height depends on how long Space is held: holding it through the rise gives the full `PLAYER_JUMP_HEIGHT`, releasing it early cuts the rise (`PLAYER_JUMP_RELEASE_MULTIPLIER`). The same applies to mid-air jumps.
- Left Shift: dash.
- J: swing the sword (press repeatedly to chain a combo).
- E: interact with an element.
- 1 / 2: choose the first / second bonus. R: reroll the bonus offers. Up/Down + Enter, or a mouse click, also work on the bonus screen.
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

- Player actions have sound effects synchronized with their animations. Include movement and surface-specific footsteps, jumping, landing, sword swings and hits (with a distinct combo-finisher hit), enemy projectiles striking the environment, dashing, taking damage, healing pickups, and death. Idle sounds may be subtle and should not become repetitive or distracting.
- Sword swings must sound different when they hit an enemy. Damage and healing sounds must be easy to recognize.
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
