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
- The protagonist uses "The Dude" Premium v0.51 pack. Its sheets live in `src/assets/player/sheets/` (the pack's Aseprite sources in `src/assets/player/source/`), and `src/assets/player/animations.yaml` lists every clip (sheet, grid row and first frame, frame count, frame rate) and how the 128×128 frames map onto the collision box: the 45 px body is drawn 0.75 H tall (`body_height_h`), feet on the collision box's baseline. The player always holds the gun, so gun versions of the pack's animations are used wherever they exist; the pack's sword, sword-and-shield, push, pull, roll, slide, interact, and unarmed idle/walk/wounded animations are not used.
- Every player action, including idle, has a dedicated animation:
  - Idle: the alert idle while enemies are alive, the calm gun idle once the room is clear, and the wounded idle below 25% health (`wounded_health_fraction`). After `idle_fidget_after_s` (4 s) standing still, a left- or right-hand fidget plays (alternating), then the idle resumes.
  - Ground movement: walk while accelerating or below 60% of maximum speed (`jog_speed_fraction`), jog above it, and sprint while the Sprinter bonus sprint is active. Reversing direction on the ground plays a turn-around step (the running variant above jog speed). Holding Down plays the crouch (stealth) idle and walk.
  - Air: jump take-off then a rising loop, a falling loop, a landing crouch when touching down without horizontal input, and for walls the latch cling, the landing against the wall, the wall slide, and the wall jump. Mantling onto a ledge plays the grab.
  - Actions: dash, cover, reload (stretched over the whole reload while standing; moving keeps the movement animation), hit reaction when taking damage, and death.
  - Shooting: one shot clip per aim direction (straight, diagonal down, diagonal up), each with two variants (two-handed, one-handed) that consecutive shots alternate between. After a shot, the matching aim pose holds for `aim_hold_s` (0.4 s) and as long as Shoot is held. Firing straight while moving on the ground plays the run-shooting cycle instead, restarted on its shot frame.

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
| `PLAYER_WALL_LATCH_DURATION`       | 0.15  | s clinging to a wall before sliding down it                       |
| `PLAYER_CROUCH_SPEED`              | 3     | H/s while crouching                                               |
| `PLAYER_COVER_DAMAGE_MULTIPLIER`   | 0.5   | Damage taken from the front while in cover                        |
| `PLAYER_DEATH_HOLD`                | 0.6   | s the last death frame holds before the run ends                  |

### Walls, Ledges, and Platforms

- Rooms are built from solid terrain (floors, walls, ceilings, pillars) that blocks the player from every side, and one-way platforms that the player lands on, jumps through from below, and drops through with Down + Jump.
- Wall slide: while falling and pressing into a solid wall (or the room edge), the player first clings to it for `PLAYER_WALL_LATCH_DURATION` (no falling), then slides down no faster than `PLAYER_WALL_SLIDE_MAX_FALL_SPEED`. On a wall the player's back is against it: the player faces (and shoots) away from the wall, and keeps facing away through the wall jump's animation even while the direction toward the wall is held.
- Wall jump: pressing jump while airborne against a wall kicks the player up and away from it. Horizontal input is ignored for `PLAYER_WALL_JUMP_CONTROL_LOCK`. A wall jump uses up the take-off jump, so Double Jump still grants its mid-air jump afterwards. Releasing jump early cuts the rise, as for any jump.
- Ledge mantle: while airborne and moving toward a solid block whose top is above the feet and at most `PLAYER_LEDGE_GRAB_REACH` above the head, the player climbs onto it automatically over `PLAYER_LEDGE_CLIMB_DURATION`, provided there is room to stand on it.

### Weapon Specifications

The gun is the player's only weapon. Holding Shoot (J) fires a bullet every `PLAYER_GUN_FIRE_INTERVAL` while the magazine has rounds; there is no combo. Each shot fires horizontally in the facing direction, or diagonally (`PLAYER_GUN_DIAGONAL_ANGLE` above or below horizontal) while Up or Down is held. A crouching player stands up for the shot (the diagonal-down aim) and crouches again afterwards.

- Bullets are visible projectiles that leave the gun's muzzle (per-aim offsets in `player.gun.muzzle_offset_h`), fly straight at `PLAYER_BULLET_SPEED`, and stop at the first enemy they touch, at solid terrain, at the room's edges, or after `PLAYER_BULLET_RANGE`. They fly through one-way platforms. A bullet hits only one enemy. An enemy pressed against the player is still hit point-blank.
- Every shot bursts into a small fiery explosion at the muzzle, and every bullet hitting an enemy bursts into a larger one at the impact: a hot core that swells and cools from yellow to red while embers fly outward, then fades (`muzzle_flash` and `impact_burst` in `player.gun`). A bullet hitting terrain makes a small burst.
- The magazine holds `PLAYER_GUN_MAGAZINE_SIZE` rounds and starts full in every room. Emptying it reloads automatically; Reload (R) refills a partial magazine early. Reloading takes `PLAYER_GUN_RELOAD_DURATION`, during which the gun cannot fire. Reserve ammunition is unlimited.
- Ammo HUD: the bottom-right corner shows the rounds left over the magazine size (for example `7 / 10`) above one small pip per round. Spent rounds are dimmed; while reloading the text reads `RELOADING` and the pips refill one by one with the reload's progress.
- Cover (L, held on the ground): the player raises a guard and can neither move, jump, dash, shoot, nor start a reload. Damage from enemies in front of the player is multiplied by `PLAYER_COVER_DAMAGE_MULTIPLIER`; hits from behind are not reduced.

| Constant                                 | Value | Unit or notes                                         |
| ---------------------------------------- | ----- | ----------------------------------------------------- |
| `PLAYER_GUN_DAMAGE`                      | 20    | Damage per bullet                                     |
| `PLAYER_GUN_FIRE_INTERVAL`               | 0.2   | s between shots (5 shots per second)                  |
| `PLAYER_GUN_MAGAZINE_SIZE`               | 10    | Rounds                                                |
| `PLAYER_GUN_RELOAD_DURATION`             | 0.9   | s                                                     |
| `PLAYER_BULLET_SPEED`                    | 18    | H/s                                                   |
| `PLAYER_BULLET_RANGE`                    | 12    | H                                                     |
| `PLAYER_GUN_DIAGONAL_ANGLE`              | 45    | Degrees above or below horizontal                     |
| `BOSS_KNOCKBACK_FROM_BULLETS`            | 2.5   | H/s horizontally away from the player                 |
| `PLAYER_DAMAGE_INVULNERABILITY_DURATION` | 0.55  | s after taking damage (33 frames at 60 FPS)           |
| `PLAYER_DASH_INVULNERABILITY_DURATION`   | 0.12  | s at the start of the dash                            |

### Combat Feedback and Room Flow

- Combat is fast, mobility-heavy, and skill-based.
- The player never receives knockback. Damage may trigger the usual feedback and hit reaction, but must not displace the player.
- Bosses are the only enemies affected by knockback. Bullet hits may push bosses horizontally away from the player using the boss knockback constant above. Ordinary enemies are never displaced by knockback.
- Whenever the player deals or receives damage, show a brief floating damage number. Damage dealt appears over the enemy; damage received appears over the player in bold red. Numbers pop up, drift upward, and fade quickly so they remain legible without obscuring combat.
- Each player move, including shooting, reloading, and dashing, has a dedicated animation. Use placeholder sounds only for now (see Sound Design).
- Every room, the first included, opens with an entrance sequence before combat starts. The player walks in from the left at walking pace for 0.5 s, fading in, and stops on the room's player spawn, then stays idle; input is locked for the whole sequence. Once the player stands idle, the enemies materialize one after another (each rises out of its spawn point from a shadow tint into full color, with a slight overshoot) and their health bars fade in with them, with a placeholder cue. The fight starts right after the last enemy has appeared; in the boss room the boss introduction follows the entrance. Timings live in `room_flow.room_intro` in `src/assets/config/gameplay.yaml`.
- After all enemies in a room are defeated, play a 0.5-second, full-screen room-clear animation, then show the bonus choices.
- The game is not paused during the room-clear animation and the bonus screen: the room keeps running behind them, and the player keeps living without taking movement or combat input (an airborne player lands, then plays the calm idle and its fidgets).
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
- Each room file describes its ID, chapter, an optional description of the layout's idea, dimensions, player spawn, background profile, solid terrain, one-way platforms, decorative props and their positions, enemy spawns and their configured attributes, and exit position. Terrain blocks, platforms, and enemy spawns may carry an optional `label` naming their role ("floor", "hanging rock"); labels and the description are data fields rather than YAML comments, so tools that rewrite the files keep them.
- Player and enemy spawn `position_h.y_h` values mark the feet baseline. The player and every enemy stand on the configured platform top, and sprites are bottom-aligned to that shared baseline even when their source frames have different heights.
- Terrain blocks and one-way platforms are rectangles given by their lower-left corner and size (`rect_h`). Solid `terrain` blocks stop bodies and enemy projectiles from every side; one-way `platforms` (at most 0.5 H thick) only hold up bodies landing on them, and projectiles fly through them.
- Layouts should form reachable routes. The ground floor runs continuously from the entrance doorway to the exit, with steps no higher than a jump. Upper levels step up 2 H at a time through planks and ledges so enemies can follow the player; higher ledges may require the player's mantle or a wall jump. Upper floors keep at least one open edge so enemies can come back down. Place some enemy spawn feet on raised tops and the rest on the ground.
- Each room has a doorway clear of solid terrain in its left wall where the player walks in, and the exit stands on the ground floor near the right wall.
- A room is 2 to 4 times the fixed gameplay viewport's width (32 to 64 H) and 1.5 to 3 times its height (13.5 to 27 H). These limits live in `room_size_limits` in `src/assets/config/gameplay.yaml`. Room geometry positions and dimensions use `H`, the player's collision-box height, as the reference unit.
- Each background profile defines a backdrop and parallax layers, and each room in a chapter has its own profile. Layers scroll horizontally at their parallax rate and stay centered on the camera vertically, behind the room's drawn terrain. Chapter 1 drifts from forest daylight to a stormy boss: room 1 `chapter_01_forest`, 2 `chapter_01_hills`, 3 `clouds_bright_blue`, 4 `clouds_golden_tower`, 5 `clouds_pastel_tower`, 6 `chapter_01_autumn_canopy`, 7 `clouds_sunset_fire`, 8 `clouds_dusk_mauve`, 9 `clouds_night_blue`, and 10 (the boss) `clouds_thunderstorm`. Room layouts, platform placement, enemy placement, and decorative props vary so rooms in a chapter do not all look alike. Further profiles are available for rooms and later chapters: 28 cloud skies (`clouds_*`, from `background/clouds/`) and 8 city skylines (`city_*`, from `background/city/`); in both, the opaque sky never scrolls and nearer layers scroll faster (clouds up to 0.35, the front building row 0.5). Each background profile also declares its texture `filtering`: `nearest` keeps upscaled pixel art crisp on large displays instead of blurring it, `linear` smooths it.
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
description: null # optional note on the layout's idea
size:
  width_viewports: null # 2 to 4
  height_h: null # 1.5 to 3 viewport heights
background_profile: null # unique to this room within its chapter
player_spawn: # y_h marks the player's feet baseline; a doorway leads in from the left
  x_h: null
  y_h: null
terrain: [] # solid blocks: optional label, surface (soil or stone), and rect_h { x, y, width, height }, lower-left corner
platforms: [] # one-way platforms: optional label, surface (wood), and rect_h, at most 0.5 H thick
decorations: [] # entries define asset, position_h, and visual layer
enemy_spawns: [] # position_h.y_h marks enemy feet; entries define an optional label, type_id, and optional stat_overrides (health, movement_speed_h_s, attack_damage)
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
- Initial bonus pool (21 bonuses):
  - **Hollow Points** (_Combat_): Increase bullet damage by 20%.
  - **Last Round** (_Combat_): The last bullet of each magazine deals 50% more damage.
  - **Hair Trigger** (_Combat_): Increase fire rate by 15%.
  - **Extended Magazine** (_Combat_): Increase magazine size by 4 rounds.
  - **Thick Skin** (_Defense_): Increase defense by 10%, causing attacks to deal 10% less damage.
  - **Room Ward** (_Defense_): Negate the first damage event in each room.
  - **Vitality** (_Defense_): Increase maximum health by 15 and restore 15 health when acquired.
  - **Scavenger** (_Defense_): Increase the chance of an enemy dropping a 10-HP health pickup by 25% relative to the base chance.
  - **Fleet Feet** (_Agility_): Increase movement speed by 10%.
  - **High Jump** (_Agility_): Increase jump height by 15%.
  - **Dash Recharge** (_Agility_): Reduce dash cooldown by 20% (from 0.8 s to 0.64 s at the current baseline).
  - **Double Jump** (_Agility_): Gain one additional mid-air jump at 80% of the normal jump impulse.
  - **Sprinter** (_Agility_): Ending a grounded dash while still holding its direction keeps the player sprinting at 8.5 H/s (Sprint animation on the ground; the speed carries through jumps) until they stop, turn, crouch, take cover, or run into a wall.
  - **Marked Prey** (_Intelligence_): A bullet hit marks an enemy for 2 s; further bullets deal 15% more damage to that enemy while marked.
  - **Rhythm Training** (_Intelligence_): Each hit within 1.5 s of the previous hit increases fire rate by 4%, up to 16%; the bonus resets after 1.5 s without a hit.
  - **Chain Reaction** (_Intelligence_): Killing an enemy increases damage by 15% for 2 s.
  - **Speed Loader** (_Intelligence_): Reload 30% faster.
  - **Vampiric Rounds** (_Combined: Combat + Defense_): Each bullet hit restores 1 health, up to 5 health per magazine.
  - **Dash Shot** (_Combined: Combat + Agility_): The first shot within 1 s after dashing deals 25% more damage.
  - **Aerial Gunner** (_Combined: Combat + Agility_): Deal 20% more bullet damage while airborne.
  - **Sated Fury** (_Combined: Combat + Defense_): After collecting a healing pickup, increase fire rate by 15% for 3 s.

## Assets

- Game assets are stored in `src/assets/`. Add more assets as development proceeds.
- Player assets are stored in `src/assets/player/`.
- Enemy assets are stored in `src/assets/enemies/`. Each enemy type has its own subfolder (for example, `src/assets/enemies/type1/` and `src/assets/enemies/type2/`). Boss assets are stored in corresponding boss folders (for example, `src/assets/enemies/boss1/` holds the Chapter 1 Evil Wizard).
- Background assets are stored in `src/assets/background/`.
- Assets may be provided as PNG, PSD, or Aseprite files.

## Controls and Settings

Default controls:

- ZQSD (AZERTY) or arrow keys: move. Down (S or Down arrow) crouches on the ground and aims diagonally down while shooting; Down + Space drops through a one-way platform. Up (Z on AZERTY, W on QWERTY, or Up arrow) aims diagonally up while shooting.
- Space: jump; against a wall in the air, wall jump. Jump height depends on how long Space is held: holding it through the rise gives the full `PLAYER_JUMP_HEIGHT`, releasing it early cuts the rise (`PLAYER_JUMP_RELEASE_MULTIPLIER`). The same applies to mid-air jumps.
- Left Shift: dash.
- J: shoot (hold to keep firing).
- R: reload.
- L: cover (hold).
- E: interact with an element.
- 1 / 2: choose the first / second bonus. R: reroll the bonus offers. Up/Down + Enter, or a mouse click, also work on the bonus screen.
- Death: the death animation plays and holds for `PLAYER_DEATH_HOLD` with every control ignored before the run-ended screen appears.
- Escape: open or close the pause menu. The menu offers Continue, Restart, Settings, and Leave. Leave returns to the main menu shown before a run.
- Every menu (main, pause, settings, controls, run ended, chapter cleared) shares the bonus screen's look over the blurred game: a centered title, then one dark bordered card per option with its key in a badge. Settings show each volume as a bar; the controls list fills two columns, and the selected row is highlighted.

Settings:

- Start in a maximized window: it fills the screen but keeps its frame and the system menu bar or taskbar (`window.start_mode` in `config/gameplay.yaml`; `borderless_fullscreen` and `windowed` are the alternatives). Settings can switch to borderless fullscreen or back; both preserve the fixed gameplay viewport.
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

- Player actions have sound effects synchronized with their animations. Include movement and surface-specific footsteps, jumping, landing, gunshots and bullet hits (with a distinct, heavier hit for the Last Round bullet), reloading, cover blocking a hit, enemy projectiles striking the environment, dashing, taking damage, healing pickups, and death. Idle sounds may be subtle and should not become repetitive or distracting.
- Gunshots, reloading, and the exit door opening use recorded effects from `src/assets/audio/sfx/` (`gun-shot.wav`, `reload.wav`, `door-open.wav`), configured with their gain under `audio.recorded_effects` in `config/gameplay.yaml`. The other effects are still synthesized placeholder tones.
- A bullet hitting an enemy must sound different from one hitting terrain. Damage and healing sounds must be easy to recognize.
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

- Store audio assets in `src/assets/audio/`, organized by category, with descriptive, consistent filenames. Name numbered variations with suffixes such as `bullet_hit_01.wav` and `bullet_hit_02.wav`.
- Choose audio formats appropriate to playback latency and quality. Preload frequently used sound effects, and ensure missing optional audio assets do not crash the game.

Important for audio production: use temporary placeholders (simple generated tones/noise or silence) for required events. Keep placeholder calls easy to replace, and make sure missing audio files never prevent the game from running.
