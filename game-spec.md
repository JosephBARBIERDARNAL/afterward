# Afterward game specifications

## Overview

- Afterward is a single-player action 2D, roguelike pixel-art platformer game where player has to clean 100 rooms full of enemies
- Each room is harder and harder because of 3 reasons: more enemies, more powerful and different kind of enemies
- After cleaning a room, player can choose one bonus among two
- Every 10th room, there is a boss and the environment changes on the next room
- The player enters a surreal world, fights through a 100-room run in ten themed chapters, creates a temporary build, and attempts to defeat the boss at the end of each chapter. The final boss waits in room 100.
- Death ends the run and destroys the temporary build. Nothing earned or chosen in one run carries into another: there is no currency, permanent upgrade tree, unlock, or account progression.
- Early runs will often end before the player reaches later chapters because the player is weak and inexperienced.
- The game has no account logic and no telemetry. It runs entirely offline and locally.

## Player

- There is one protagonist
- Character creation is not included
- All player assets (png and aseprite files) live in `src/assets/player/`
- All player actions, including idle, must have its own dedicated animation

## Combats

- Player has a sword that does range damage (all enemies in a 150° range in front) and a gun that shoots a bullet
- Default sword damage is 5, default gun damage is 15
- Combat must be fast, mobility-heavy, skill-based
- Hitting an enemy should show a brief floating damage number whenever the player damages an enemy or takes damage. Damage dealt appears over the enemy; damage received appears over the player in bold red. It uses quick pop, upward drift, and fade so the numbers are easy to notice without lingering over the fight.
- Every move (sword attack, gun, dash, etc) should have its own dedicated animation and sound design
- When combat finishes (all enemies killed in the room), there should be a 0.5 sec animation on the whole screen that makes it explicit for player, as well as display the bonus choice.
- Once bonus choice has been maid, a door on the right of the room appears, user just has to touch it to open it and go in the next room, which starts the next combat.

## Health

- Health persists between rooms during a run.
- Health is an horizontal bar at the top left of the screen, starting with 100/100
- Enemies health appear above its head in red, disappearing when it dies. An enemy with 1/3 of its life means 1/3 of the bar is red and the rest is black
- Healing is scarce enough that taking damage matters.
- Every ennemy has a chance (1 out 25) of dropping a 10 hp heal

## Rooms

- Every room has a background and is between 2 and 4 as big as the game width (in order to make them vary).
- They have a background, with parallax effects that give illusion of deep.
- There are platforms at different places in each room in order to jump on ennemies, move around the room, etc.
- Taking the door to go to the next room has its own transition animation.

## Ennemies

- Ennemies should have different behavior depending on how they look. One ennemy kind = one specific behavior.
- There are between 5 and 15 enemies per room.
- Killing one enemy is quite easy, especially in the first rooms.
- Hitting an ennemy interrupts what it was doing
- Each room has at least 2 different kind of enemies
- All enemies must have an idle, move, hit, being hited and dying animation

## Bonuses

- A bonus cannot be offered if the player already has it
- All bonuses must have a name, a description and a category
- Categories are: combat, defense, agility, intelligence and combined (a mix of a least 2 other categories)
- Each category has its own icon and its own color: red for combat, blue for defense, green for agility, pale yellow for intelligence and orange for combined

Here is the list of all bonuses available:

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

## Sound Design

### Overview

- Afterward must have a distinctive, immersive sound identity that combines fast-paced combat with a surreal, mysterious atmosphere.
- The sound design should be inspired by retro pixel-art games, using modern audio effects while preserving a slightly nostalgic feel.
- Every important player action, enemy action, interaction, and game event must have its own sound effect.
- Sound effects must provide immediate feedback without overwhelming the player, especially during intense combat with many enemies.
- All audio assets must be stored in `src/assets/audio/`, organized by category.
- The game must work entirely offline, including all music and sound effects.
- The game must have a soundtrack that evolves throughout the 100 rooms.
- Each of the 10 chapters must have its own musical identity, matching its environment, enemies, and atmosphere.
- Music should be energetic during combat, with a strong rhythm that supports fast and mobility-heavy gameplay.
- Each chapter must have at least one exploration/ambient track, one combat track, and one boss track.
- Music should transition smoothly between different gameplay states, without abrupt cuts.
- Boss music must feel more intense and threatening than regular combat music.
- The final boss in room 100 must have a unique soundtrack.
- The main menu, pause menu, death screen, and victory screen must have appropriate music or ambient sound.
- Music must loop seamlessly.
- Music should never restart unnecessarily when transitioning between rooms within the same chapter.

### Player Sound Effects

Every player action must have a dedicated sound effect synchronized with its animation.

- **Idle:** subtle sounds when appropriate, without repetitive or distracting audio.
- **Movement:** footsteps synchronized with the walking/running animation, varying depending on the surface.
- **Jump:** short, responsive sound when jumping.
- **Landing:** impact sound depending on the landing intensity.
- **Sword attack:** sharp, fast swoosh synchronized with the sword swing.
- **Sword hit:** satisfying impact sound when hitting an enemy, distinct from swinging without hitting anything.
- **Gunshot:** powerful, punchy shooting sound with a short tail to avoid audio clutter.
- **Bullet impact:** different sounds depending on whether the bullet hits an enemy or the environment.
- **Dash:** quick, distinctive movement sound emphasizing speed.
- **Taking damage:** clear impact sound, with a short character reaction.
- **Healing:** positive, recognizable sound when collecting health.
- **Death:** distinctive sound synchronized with the player's death animation.

Repeated actions such as sword attacks and gunshots should have subtle pitch and volume variations to avoid sounding identical every time.

### Enemy Sound Effects

- Every enemy type must have its own recognizable sound identity, matching its appearance and behavior.
- Each enemy type must have sounds for idle, movement, attack preparation when applicable, attack execution, taking damage, and death.
- Enemy attacks should be recognizable through audio cues, allowing skilled players to anticipate danger.
- Dangerous attacks should have more noticeable sounds than regular attacks.
- Enemy hit sounds must clearly indicate that the player's attack connected.
- Enemy death sounds should be short enough to avoid overwhelming combat when multiple enemies die simultaneously.
- Enemy sounds should vary slightly in pitch and volume to prevent repetition.
- Bosses must have unique sounds for their attacks, movements, phase changes, taking damage, and death.
- Boss attack audio cues must remain audible even during intense combat.

### Combat Feedback

- Combat sounds must feel impactful, fast, and satisfying.
- Successful attacks must sound different from missed attacks.
- Stronger attacks must have more powerful impact sounds.
- When the player takes damage, the sound must be immediately recognizable, even when multiple enemies are attacking.
- Sounds should be synchronized precisely with their corresponding animation frames.
- Multiple identical sounds played simultaneously should be intelligently limited or mixed to prevent excessive volume.
- Each chapter must have its own ambient soundscape matching its visual environment.
- Ambient sounds should reinforce the surreal atmosphere without distracting from gameplay.
- Environmental sounds may include wind, distant echoes, mechanical noises, supernatural whispers, or other effects appropriate to the chapter.
- Ambient audio should loop seamlessly and vary subtly over time.
- Moving between rooms in the same chapter should preserve the overall sound atmosphere.
- Entering a new chapter should introduce its new ambiance through a smooth transition.
- Environmental sounds should remain quieter than important gameplay sounds.
- When all enemies in a room are defeated, the combat music should transition to a calmer state.
- The 0.5-second room-clear animation must have a distinct, satisfying sound effect.
- When the exit door appears, a recognizable sound should indicate that the room is ready to leave.
- Opening the door and transitioning to the next room must have dedicated sound effects synchronized with the transition animation.
- When the bonus selection appears, a short, rewarding sound must play.
- Navigating between bonus choices must produce subtle UI sounds.
- Selecting a bonus must play a satisfying confirmation sound.
- Bonuses that introduce new gameplay mechanics must also introduce appropriate sound effects for those mechanics.
- Collecting a health drop must play a recognizable healing sound.
- Interacting with doors and other interactive elements must produce clear audio feedback.
- Every 10th room must feature a distinct boss encounter audio sequence.
- Entering a boss room should trigger a short musical transition that builds tension.
- Each boss must have unique attack sounds and recognizable audio cues.
- If a boss has multiple phases, each phase transition must be accompanied by a noticeable audio change.
- Defeating a boss must trigger a powerful, satisfying sound sequence.
- After a boss is defeated, the music should transition into a short moment of relief before continuing.
- Defeating the final boss must trigger a unique victory audio sequence.
- All interactive UI elements must have consistent sound effects.
- Navigating menus must produce subtle feedback sounds.
- Confirming a selection must have a distinct confirmation sound.
- Invalid or unavailable actions should produce a soft error sound when appropriate.
- Opening and closing the pause menu must have dedicated sounds.
- UI sounds must remain audible but should never be louder than important gameplay feedback.
- The death screen must have a distinctive sound or musical transition that emphasizes the end of the run.
- Restarting a run must have a short transition sound.
- The victory screen must feature a rewarding audio sequence distinct from regular room completion.
- Audio must be organized into separate music and sound-effect channels.
- The settings menu must provide independent volume sliders for music and game sounds, both ranging from 0 to 100 and defaulting to 100.
- Volume changes must apply immediately without requiring a restart.
- Audio settings must persist locally between game sessions.
- Important gameplay sounds must take priority over ambient and decorative sounds.
- When the player takes damage, the damage sound should remain clearly audible over other effects.
- Boss attack warnings must never be masked by music or other sound effects.
- Music volume may be temporarily reduced during important events such as boss introductions, player death, or victory.
- Audio must avoid clipping, distortion, and excessive loudness when many sounds play simultaneously.
- Sounds should support stereo positioning based on their location relative to the player, without making important audio cues difficult to hear.
- Audio playback must remain performant even in rooms containing 15 enemies.
- All music and sound effects must be loaded from local assets.
- Sound effects should use an appropriate compressed or uncompressed format depending on latency and quality requirements.
- Music should use a compressed format suitable for seamless looping.
- Frequently used sound effects should be preloaded to avoid playback delays.
- Sounds must trigger at the correct moment in their associated animations.
- Repeated sound effects must support slight randomized pitch variations.
- The audio system must support multiple simultaneous sound effects.
- A priority system must prevent excessive overlapping sounds.
- Pausing the game must pause or appropriately attenuate gameplay audio, while keeping menu sounds functional.
- Resuming the game must restore audio smoothly.
- Audio playback must stop or transition correctly when restarting a run, dying, returning to the main menu, or changing chapters.
- No sound should continue playing unintentionally after its associated game state has ended.
- Audio files must use descriptive and consistent filenames.
- Multiple variations of the same sound should use numbered suffixes, such as `sword_hit_01.wav`, `sword_hit_02.wav`, etc.
- Audio assets should be reusable where appropriate, without compromising the unique identity of player actions, enemy types, and chapters.
- Missing optional audio assets must never crash the game.
