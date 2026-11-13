## The game

Afterward is a single-player, offline, 2D action roguelike pixel-art platformer. The core design:

- A run is 100 hand-authored rooms played in fixed order, split into 10 chapters of 10 rooms. Each chapter has its own environment and music, and ends with a boss in its tenth room; room 100 holds the final boss. Rooms are not procedurally generated.
- A run opens in room 0, before the chapter proper: a stretch with no enemies, crossed to its door on the right, where the run's first bonus waits. It is the default start room (`just run` opens it, `just run 1` skips it). A room authored without enemies has nothing to clear: its bonus item is there from the first frame and there is no room-clear flash, but its door is locked like every other, until that bonus is taken.
- Rooms are multi-level arenas (Dead Cells style) built from solid platforms at several heights, reachable by the player and by enemies. A platform is solid on every side: it holds up a body landing on its top, stops one walking into its side, and stops one jumping into its underside, so a level is reached by going around it. Platforms stop shots too, so they are also the only cover. Everything else a room shows is a decoration panel or the background. The player walks in from the left; the exit is on the right.
- Clearing a room lays a bonus item on the room's `bonus` spot, in one of the five bonus categories, picked at random. Pressing Interact (E) beside it opens a choice of two random bonuses of that category (never one already owned, with a few rerolls per run), and choosing one unlocks the exit door, which is used with Interact too: nothing in a room is taken or left by walking into it, except a health drop. Bonuses build a temporary run build. Death ends the run and nothing carries over: no currency, unlocks, or meta-progression.
- Health persists between rooms and healing is scarce, so taking damage matters.
- Combat is fast, mobile, and skill-based, and fought at arm's length: the player's only weapon is a melee chain, thrown with Attack, whose blows follow one another while the player keeps asking for the next. Attack held rather than tapped gathers a heavier blow, and Attack with Down held in mid-air slams the floor. There is no gun, no ammunition and no ranged attack of any kind, so closing the distance under fire is the whole skill.
- Movement includes jump, crouch, cover, a dodge roll (Down with Jump, invulnerable through its middle — the melee player's one way through a shot), and the one thing a platform's side allows: a body pressed into a wall in mid-air clings to it and slides, and Jump pushes off it, Mario style. There is still no ledge mantle, and no swimming — the flood is scenery, never entered. The player never receives knockback; a body with no armor up is shoved back by every blow that wounds it.
- Enemies stay idle until they see the player — near enough, at their own height, with no platform in between; they see far on the side they face (the `facing` of their room spawn, left by default) and only a short way at their back — or until a blow wakes them: the one it lands on, and every idle one near enough to hear it. Then they chase across levels, jumping up and across gaps, but keep to the floor they stand on: one only steps off a ledge after a player standing below it. A ranged enemy fires as soon as its shot is ready, however close the player stands, and only backs away while it cools down. They deal damage only through attack hitboxes or projectiles, never by contact, and attacks are telegraphed by a readable wind-up.
- `H`, the height of the player's collision box, is the unit for every distance and speed. The visible world is a fixed 16:9 viewport, 13.5 H tall (24 H wide), scaled to fit any window without revealing more of the room.
- Every player action has a dedicated animation.

The details (tuning values, rooms, enemies, bonuses, materials, backgrounds, animations, controls) live in the YAML files under `src/assets/` and in the code, which are the reference. Don't duplicate them here.

The game is in development, focused on Chapter 1 (rooms 1–10). Content and systems needed only for later chapters (rooms 11–100) are low priority unless explicitly requested.

The project is at an early, experimental stage. Many choices (assets, room layouts and materials, tuning values, editor features, even parts of the core design above) are trials that may be reworked or dropped soon. Expect them to change: prefer simple, easy-to-replace solutions over heavy abstractions or polish, don't treat the current state as a settled design, and when something looks like a placeholder or a half-made decision, ask rather than build on it as final.

## Run and check

Once you finish making a change (fixing a bug or adding a feature), make sure all code is well formatted, linted and tested. Everything should be green. This will run in CI too. Common commands:

- `cargo run` — start the Bevy build.
- `cargo fmt --all -- --check` — check Rust formatting.
- `cargo clippy --locked --all-targets -- -D warnings` — check Rust lint.
- `cargo test --locked` — run Rust tests.
- `cargo build --locked --bin afterward` — compile the native game.

Git hooks are managed by [prek](https://prek.j178.dev) from `prek.toml`: file hygiene, YAML/TOML syntax, `cargo fmt`, Clippy, and typstyle on commit, `cargo test` on push. Install them once with `prek install --install-hooks`; `prek run --all-files` runs them on the whole repo. Fixers (trailing whitespace, final newline) edit files in place: re-add and commit again.

## Rules

- When something is unclear, needs decision, is inconsistent or contradictory with something else, ask explicitly what should be done.
- When defining the number of enemies, room organization, etc., always do it in a consistent and centralized way in yaml files so that making changes in a specific room is easy. Those files should be the source of truth on the construction of the game and make the game a true data-driven game.
- All constant parameters (game dimensions, player stats, etc.) should be clearly defined in such way and easy to change.
- Always feel free to move, rename and change asset files for consistency and simplicity.
- Keep source files focused and reasonably sized. Split code into separate modules by responsibility instead of letting individual files grow too long.
- Follow established software engineering practices: keep responsibilities clear, reuse shared logic where appropriate, and keep configuration data in its designated source-of-truth files.
- Keep Clippy rules intact. Fix lint findings in code instead of lowering lint levels or adding lint suppressions to make checks pass.
- No free functions for validation or conversion. Define a newtype and implement `Deserialize`, `FromStr`, or `TryFrom` on it.
- A helper called once is inlined. This applies to test fixtures as much as to production code; a fixture used by two tests is still inlined into both.
- Combine `use` statements from one crate into a single `crate::{...}`.
- No accessor that only returns a field. Widen the field to `pub(crate)` instead of writing a getter around it.
- Prefer a tailored API over `std::mem::take`. `Option::take`, `Vec::split_off` and `HashMap::remove` name what moves and leave the container usable, where `mem::take` silently swaps in a `Default`.
- Treat every git change you did not explicitly make as belonging to someone else working on the project simultaneously; preserve it exactly and do not revert, restore, overwrite, format, or otherwise “clean up” that file. When this happens, only git add the files you modified. You don't have to mention that there are some unrelated changes.
- Don't add content to README.md; relevant development knowledge lives in AGENTS.md.
- When you finished implementing a feature or fixing a bug, make sure to commit it with a clear and concise commit message. Don't push to GitHub.

## Bevy

The game is built using Bevy 0.20, in Rust. Here are relevant sources to look into:

- https://bevy.org/
- https://docs.rs/bevy/latest/bevy/
- https://bevy.org/learn/quick-start/getting-started/setup/
- https://bevy.org/learn/quick-start/getting-started/ecs/
- https://bevy.org/learn/quick-start/getting-started/plugins/
- https://bevy.org/learn/quick-start/getting-started/resources/

The game targets are MacOS, Windows and Linux (CI runs tests for all of them).

## Development

Common tasks are in the `justfile` (run `just` to list them):

- `just run` — play from room 1; `just run 7` starts new runs (and pause-menu restarts) in room 7.
- `just boss` — start at the Chapter 1 boss (room 10, with its intro).
- `just check` — formatting, Clippy, tests, and the native build, as in CI.

Without `just`, pass the flag directly: `cargo run -- --room 7`.

### Seeing a room

`just shot 3` (`scripts/room_shot.sh`) writes a picture of room 3 to `target/room-shots/room_003.png` and prints its path, in about a second and without opening a window. Use it after every edit to a room's YAML, and read the PNG, rather than guessing from the numbers.

- `just shot --grid 3` rules the picture in H (a line per H, a number every 5, origin bottom left, as in the YAML) and writes `room_003_grid.png`: read a spot off the grid and it is the `x_h`/`y_h` to write. `just shot 3 4` takes several rooms, `just shot` every room.
- It is the real game (`cargo run -- --room 3 --shot [--grid]`, `src/game/room_shot.rs`): the room is built by `world::spawn_room` on the `Playing` screen with the clock paused, and drawn into an image instead of a window, so platforms, decorations, enemies with their armor, the player on their spawn and the exit door are what a run shows on its first frame. The bonus item is not there: it is only laid once the room is cleared.
- It shows the whole room, where the player only ever sees a 24 H by 13.5 H window of it (the editor's F overlay frames that). The picture's long side is 1920 px at most, so a wide room is drawn smaller; the backdrop, which in play is one view tall and follows the camera, is laid flat and enlarged to cover a taller room; there is no HUD.
- It needs a GPU but no display. A failed shot prints the game's log and exits non-zero.

### End-to-end tests

`src/game/tests/e2e.rs` runs the real `GamePlugin` inside Bevy's default plugins, headless (no window, no GPU, no audio device), at a fixed 60 FPS with a fixed random seed. Scenarios in `src/game/tests/e2e/` play it through keyboard events (`tap`, `key`, `typed_key` for other layouts) and check the screen and run state. Run them with `just e2e`; CI runs them on Linux, macOS, and Windows as their own step.

- Add a scenario for every player-facing flow or input bug: menus, pause, death, the bonus screen, the editor's shortcuts.
- Tests must survive room edits. Gameplay tests build their own small room inline (see `enemy_jumping.rs`) instead of using an authored one. The full Chapter 1 playthrough (`chapter_one_run.rs`) depends on every room's layout, so it is `#[ignore]`d while rooms are being designed: run it with `just playthrough`.
- Scenarios must survive room edits: `defeat_enemies`, `take_bonus` and `step_toward_exit` are the only shortcuts, for steps whose real input depends on a room's layout. Don't assert on specific spawn positions.
- A headless test has no window and so no real pointer: `point_mouse_at` and `click_mouse_on` write the events the picking backend would have written, at a button `menu_button` looks up by its `MenuAction`. Both run a second frame, as `tap` does, so a screen the touch asked for has landed.
- `PreferencesDir(None)` keeps tests away from the player's saved settings and bindings. `HeadlessGame::with_copied_data` points the YAML data (and editor saves) at a temporary copy, so tests never write to `src/assets`.

### Debug room editor

Debug builds (`cargo run`, `just run`) add `D  Debug editor` to the main menu (opens the start room) and the pause menu (opens the room being played). The editor lives in `src/game/editor/` and is compiled out of release builds (`#[cfg(debug_assertions)]`).

- Shortcuts take Cmd on macOS and Ctrl elsewhere (both work everywhere; the help shows the platform's). They read `KeyboardInput` events rather than `ButtonInput`, because macOS sends no key-up for keys released while Cmd is held. Letter shortcuts match the character the key types on the active layout (`KeyboardInput::logical_key`), not its physical position, so Cmd+Z is Z on AZERTY too.
- It edits the rooms in `GameData` in place: Cmd+R playtests the edited room (and makes it the start room, so pause-menu restarts replay it), and the edits stay in memory until the game quits.
- Cmd+R first opens the playtest settings (Up/Down, Left/Right, Enter or Cmd+R again to play, Esc to go back): infinite health, whether the rooms have enemies, and the bonuses the run opens with (none, 5 at random, all). They are `RunSession::playtest` (`session.rs`), read by the game itself, and like the start room they stay until the editor changes them — a run begun from the main menu afterwards is played the same way.
- Cmd+S validates all game data with `GameData::validate` (the same checks as at startup) and writes each changed room back to `src/assets/rooms/room_NNN.yaml` through the `Display` impl in `src/data/room_yaml.rs`. Saving is refused while the data is invalid; the inspector shows the problem as soon as an edit causes it.
- Saved files use one canonical layout. Keep notes about blocks and enemies in `label:` fields; per-entry YAML comments would be lost on the next save. When a room field is added, update `room_yaml.rs` too (its round-trip test catches fields it drops).
- The bonus spot is an item like the player spawn and the exit: it can only be moved, never deleted or copied.
- A platform can be turned: `rotation_deg` on its entry, in degrees counter-clockwise about the middle of its `rect_h`, left out of the file while it stands square. It is the `Rotation (deg)` row of the inspector (Left/Right turn it 5 degrees, C stands it square again), which also says whether the result is a slope to walk or a wall. A turned platform is outlined and clicked where it stands, but its edges are not dragged: resize it with Width and Height. `collision.rs` gives it the shape it is drawn with — the gentler of its two upper faces is walked by the player and the enemies when it leans no more than `player.walkable_slope_deg`, and every other face stops a body like a side, so one landing on a steep face slides down it. Enemies walk a slope but never leap at one: the jump planner only aims at platforms standing square.
- Holding H shows the shortcuts panel (one line per shortcut; obvious ones like copy, paste, undo, save, and click-to-select are left out). The controls: click to select, Shift+click to add or remove, drag empty space to box-select, drag to move the whole selection (Alt for the fine grid), drag a selected block's edge to resize, two-finger trackpad scroll or right/middle drag to pan, pinch or mouse wheel to zoom, P/D/E to add a platform/decoration/enemy at the cursor, Cmd+C/X/V to copy, cut, and paste (into any room, at the cursor), Cmd+A to select all, Cmd+D to duplicate, Tab to move a single selection to the closest item of the same kind (enemy, platform or decoration), arrows for the inspector (room size, the two backgrounds, fill, a platform's rotation, decoration layer, enemy type, the way an enemy faces, its layers of armor, labels; Fill and X/Y apply to the whole selection).
- Two layout overlays in `src/game/editor/overlays.rs` answer, in the editor, what otherwise takes a playtest; F and V toggle them and the inspector names the ones that are on (the keys themselves are only in the H panel). Each runs the rules the game plays by rather than its own: `RoomCamera::clamp_center_h` for F, which frames the 16:9 viewport where the camera sits when the room opens and where it would sit with the player under the cursor; and `blocks_sight` for V, which fills every idle enemy's detection band — the long one in front of it and the short one at its back — with sight lines that stop against the platforms hiding the far side, and draws a pale line to the player spawn when the enemy already sees it standing there.
- The fill palette (top right) shows every material of `surfaces/materials.yaml`, then a color picker: a hue strip over a grid of shades mixed from the picked hue. Clicking a material or a shade paints the selected platforms and decorations and becomes the fill new ones use; clicking a hue only remixes the shades. Clicks over a panel never reach the room.
- A block names either a material (`material: city_pavement`) or a plain color (`color: "#4C5A74"`), never both: that is the `Fill` enum, flattened into the YAML entry. A color block takes the `color_footstep` of `surfaces/materials.yaml`.
- The key that opens the editor from a menu (D) is ignored on the frame it opens, so it doesn't also add a decoration.

### The melee chain

The player's whole offence is `player.melee` of `config/gameplay.yaml`: a list of `steps`, each naming the clip it plays (`strike`), its `damage`, its `hitbox_h`, how far into the clip it connects (`contact_s`) and the forward shove it gives (`lunge_h_s`). Attack throws the next step; once a blow ends there is `combo_window_s` to ask for the next, and letting that lapse drops the chain back to its first step. The steps with no `requires_bonus` are the chain every run throws. Those with one are its rival finishers — Power Strike, Fire Kick, Explosive Strike — and a run throws only one of them, whichever it owns that is listed last, so the order of those steps is the order they outrank each other in. `MeleeConfig::chain` is the one place that decides it.

An Attack pressed while the blow in hand is still playing (or a roll still under way) is remembered for `buffer_s` and thrown the moment it can be, so mashing never drops a press; Jump is remembered the same way for `player.jump_buffer_s`. A blow on its way in carries the player forward at its `lunge_h_s`, never slower than they were already running, and stops short as soon as it can reach a body: it closes the distance without running through anyone. `player/mod.rs` leaves the player's horizontal speed alone until the blow connects.

A blow that lands is felt (`src/game/impact.rs`, tuned from `melee.impact`): `combat/blows.rs` spawns an `Impact`, and `feel_the_impacts` holds the whole game still for a few frames — longer on a kill — and throws the camera off its mark. The freeze pauses the virtual clock, so every system reading `Time` stops together while input is still heard and buffered; the freeze itself and the camera's `Shake` run on the real clock. The body that was wounded flares for `flash_s` and is shoved away by `knockback_h_s`, stopping against platform sides like a walk.

Attack held rather than tapped gathers, once the blow in hand has ended, and letting go after `charge.ready_s` throws one heavy blow that goes through a shield instead of being soaked by it. Attack with Down held in mid-air drives the player at the floor: `player/mod.rs` owns that drop and reports the landing, and the floor breaks to both sides of them (`melee.slam`). Down with Jump on the ground rolls instead of hopping, and nothing can reach the player between `roll.invulnerable_from_s` and `roll.invulnerable_to_s` of it.

`src/game/combat/player_melee.rs` runs all three ways a blow arrives; every one of them lands through `combat/blows.rs`, so a hit costs the same and sounds the same however it was thrown, and a blow hurts everything its hitbox reaches — one kick can catch two bodies standing together. A layer of armor soaks an ordinary blow whole, but not the charged one. `blow_area` in `combat/hitboxes.rs` is the one place that turns a hit area in H into a world rectangle, for the chain, the charged blow, the slam and the debug overlay alike.

### What the body shows

The player's state is meant to be read off the body rather than the HUD, so several things that used to happen silently now play a clip. Taking a health drop, or a bonus that heals a cleared room (Second Wind, Clean Sweep), plays the healing glow; Recovery's per-second trickle, and a pickup lying on the floor, wear the drifting plusses of `effects/health_regeneration.png` (`HealEffect`). Taking a bonus wraps the player in that category's glow for a moment (`BonusAura`, one clip per category, the defensive one being the guard's own sheet). And Last Stand is real now: the hit that would end the run instead drops the player, who gets up again at 1 health, untouchable and unable to act until they have.

Decoy puts a second body in the room: a roll plants a standing double of the hero on the spot it set off from (`combat/decoy.rs`, the `player_summon` clip played once and held), and only one stands at a time. It has health and nothing else — it never moves and never strikes. `Mark` is what an engaged enemy goes after, the player or whichever double stands nearer to it, and it is what decides where a blow or a shot lands: a hit on the double comes out of the double's own health and costs the run nothing. `plant_decoy` runs between `player::update_player`, which notes where the roll began, and `update_enemy_ai`, which marks what it left standing.

### Backgrounds

A room lists its scenery in `background_profiles`, each naming a profile of `background/background_profiles.yaml`, and `world::draw_room` mixes them into one backdrop rather than laying one over the other. Their skies (each profile's `backdrop`) are blended in equal parts. Their layers are then dealt together from the furthest to the nearest, by `parallax`, so with two city profiles one skyline's far towers stand behind the other's near ones; each layer keeps its own profile's tile size and scrolling. The order of the list only breaks ties between layers that scroll alike. The inspector's `Background 1` and `Background 2` rows set the first two, the second through `None`; a longer list is written by hand and left alone by the editor.

### Hitboxes

Every hit area is data, in H: the player's hurtbox and the hitbox of each melee step in `config/gameplay.yaml` (`player.hurtbox_h`, `player.melee`), and each enemy type's hurtbox, attack hitboxes, and projectile sizes in `enemies/enemy_types.yaml` (its header explains the coordinates). `src/game/combat/hitboxes.rs` turns them into world rectangles for combat. The debug editor edits them too: select an enemy (its type's hurtbox, and the hitbox of the attack the Attack field picks) or the player spawn (the player's hurtbox); edits apply to every enemy of that type, undo with Cmd+Z, and Cmd+S rewrites only their lines in the YAML files, keeping the comments. The size the player is drawn at rides the same path: `Body height (H)` on the player spawn is `body_height_h` of `player/animations.yaml`, outlined pale over the spawn while it is selected. It scales the sprite (and Decoy's double) only — the collision box stays 1 H and the hurtbox keeps its own fields. In debug builds, B toggles an overlay (`src/game/hitbox_overlay.rs`) that draws them with that same code: green player hurtbox, yellow enemy hurtboxes, red strikes (faint for an enemy's next attack, and for the blow the player's next press would throw), orange projectiles.

### Menus

Every row of every menu — main, difficulty, pause, settings, controls, and the
two end-of-run screens — is a button, and so are the bonus screen's boxes. The
mouse points at one and clicks it, the arrows walk to it, the confirm key
presses the one the focus stands on, and the letter on a row's badge still
presses it from anywhere on that screen.

- The prologue's way in is on that same list (`MenuAction::of(GameScreen::Prologue)`),
  though the prologue draws the button itself, under the words it writes: it
  carries `MenuButton` like a menu row, so the arrows, the pointer, the ripple
  and the focus highlight all treat it as one.
- `MenuAction::of(screen)` in `src/game/menus/actions.rs` is the one list of a
  menu's buttons, in the order they are drawn. `menus.rs` draws that list and
  `menus/input.rs` walks it, so what is on show and what the arrows reach can
  never disagree. Adding a button means adding it there and giving it a row in
  `MenuState::row`.
- `MenuFocus` holds the button the player is on. A menu opens on the button
  `menus/input.rs` hands `MenuFocus::settle`, which is its first — why Enter
  still resumes a paused run and still begins one from the main menu — except
  the difficulty screen, which opens on the choice already made.
- Begin does not start a run: it asks how hard it should be. The difficulty
  screen (`GameScreen::Difficulty`) offers Easy, Normal and Hard, each box
  edged in its own colour from `difficulty:` of `config/gameplay.yaml` and
  dimmed until the focus stands on it (`MenuEdge`). Choosing one writes
  `RunSession::difficulty` and opens the run. Nothing reads that choice yet: it
  outlives a run so the next one opens on it, and changes nothing else.
- `menus/input.rs` decides _which_ button and _that_ it was pressed; it leaves
  a `MenuPressed`, which `screens::handle_screen_input` carries out beside the
  letter shortcuts, so a click and a shortcut run the same code.
- The focus and the volume levels are dressed onto the standing buttons each
  frame (`highlight_menu_buttons`, `update_volume_bars`) rather than being part
  of the `MenuView` the entities are rebuilt from: pointing at a row must never
  pull it out from under the pointer.
- A pointer reports itself in window pixels while the UI is laid out inside the
  letterboxed viewport. `ripple::UiPointer` is what puts the two in the same
  coordinates; anything comparing a pointer to a UI node goes through it.

### Shaders

Two kinds of shader, both WGSL under `src/assets/shaders/`, both tuned from `effects:` in `config/gameplay.yaml`:

- `src/game/shaders/` holds the 2D materials, drawn on `Mesh2d` quads sized by their transform (one shared unit square, `ShadedQuad`). `flood.wgsl` is the water the main menu sits on: a cellular field whose borders light up where two cells meet, bent by slow waves and quantized to a pixel grid and to four tones, so it reads as pixel art. Its quad is laid at startup, kept under the camera and shown on the screens that stand over no room at all — the main menu, the difficulty it opens, the settings and controls reached from it, and the screen a run ends on (`SettingsReturn::over_the_flood`). Those are the menus drawn without the blur, since the water is pixel art and must stay crisp. `city_lights.wgsl` draws every background layer: a pixel that stands brighter than the four around it _and_ sits on a dark surround is a lit window or a star, and breathes at a pace its own position seeds — the dark-surround test is what keeps a dithered bright sky out of it. `glow.wgsl` is the pool of light behind anything the player should walk towards; give an entity `Glowing { color, radius_h }` and `attach_glows` hangs the quad behind it as a child, so it follows and dies with it.
- `src/game/pause_blur.rs` is the full-screen pass behind a menu, a `Core2d` post-process rather than a material.
- `src/game/ripple.rs` is the droplet that falls on a button the moment it is touched — hovered, reached with an arrow, or clicked — and the one shader drawn through the UI pipeline (`UiMaterial` on a `MaterialNode`) rather than on a `Mesh2d` quad, which is why it sits beside `src/game/shaders/` instead of inside it. `ripple.wgsl` drops the bead in, flashes where it lands and sends rings out, quantized to a pixel grid and to two tones like the flood. Anything that wants one writes a `Splash`: `Spot::Pointer` for exactly where the mouse touched, `Spot::Middle` for the middle of a button an arrow walked to. They stack in their own layer above the menus and die once they have played out.

Shaders are plain `.wgsl`, not Bevy's own `.wesl`: a material's fragment shader redeclares the mesh2d `VertexOutput` and the `globals` binding itself instead of importing them, so nothing depends on Bevy's shader module paths. Effects are meant to be felt and not noticed — turn a value up to see what it does, then put it back.

### Audio

Nothing is generated: every sound in the game is a real recording. A `SoundCue` plays the file `config/gameplay.yaml` lists for it under `audio.recorded_effects` and stays silent when there is none, and the music is one of the tracks in `audio.music_tracks`. Most cues have no sound yet; leave them silent until a recording exists rather than synthesizing a placeholder. `src/game/audio.rs` plays it all through `ManagedAudio`, whose class decides which volume setting applies (music or sound effects) and how `audio/mixing.rs` fades it per screen.

- The game opens on `audio.intro_music`: it plays from launch, carries the main menu and the opening room, and fades out over `audio.intro_music_fade_out_s` as the run leaves room 0 — the chapter track takes over when room 1 opens. Returning to the main menu starts it again, and so does dying: the screen a run ends on takes down what the run was playing and opens the game's own music again under the same water.
- `audio.ambience` is the city under the whole run: it rises over `audio.ambience_fade_in_s` on the first room a run enters and plays to the end. Its gain is what to turn to hear more or less of the street.
- Those two, the chapter track and the typing under the prologue are layers rather than cues: one entity each, carrying `ManagedAudio` and, while they come or go, an `AudioFade` the mixer multiplies in and despawns the layer with.
- One track, picked at random when the run enters its first room with a chapter, plays until the run ends. It is a `MusicStream` (`audio/music.rs`): it loops itself and runs through a low-pass filter the mixer opens and closes, so the music sounds muffled, as if played in another room, while the game is paused (`audio.paused_music_muffle`). It must play with `PlaybackSettings::ONCE` — Bevy's own looping buffers the decoded source, which would freeze the filter after the first play-through.
- A blow of the chain that lands plays the punch of its own step, from the list `audio.recorded_effects.punch_hits` (a finisher, the charged blow and the slam play the last of them), over the `damage_*` of what it landed on. One that reaches nobody plays `punch_missed` instead, at the moment it would have connected; the slam never does, since it lands on the floor either way. `combat/blows.rs` decides both.
- Recordings are cropped to what is audible, faded at both ends and peak-normalized, and stored as mono WAV in `audio/sfx/` so a cue sounds the frame it fires rather than after an MP3 decoder's padding.

### Interact

The bonus item and the unlocked door are used with Interact (E by default), never by walking into them (`src/game/interact.rs`, tuned from `room_flow.interact`). Both carry `Interactable`, their reach in H; `mark_what_is_in_reach` writes the nearest one the player stands within reach of to `InReach`, and `update_bonus_pickup` and `handle_door_touch` act on it when the key goes down. A locked door is never in reach. Health drops are not part of this: they are still taken on contact.

The thing in reach says so itself: `show_the_interact_badge` stands a badge over it naming the key Interact is bound to (read from `ControlBindings`, so a rebind shows), as its child, grown in and hovering, and `brighten_what_is_in_reach` turns its aura up by `effects.glow.reach_boost`. Giving something else the same treatment means giving it `Interactable` and reading `InReach` where it is used.

### Between rooms

A door is crossed in the open rather than behind a fade (`src/game/room_pan.rs`). The player uses the unlocked door (Interact, within reach of it) and dissolves into the portal turning over it, coming apart into its swirls rather than fading out; that room is then stood to the left of the next one, across a dark gap of `room_flow.room_gap_h`, and the camera slides over to where the new room opens, with the room behind let go the moment it lands. The slide runs at `room_flow.room_pan_speed_h_s` rather than for a fixed time, so every door moves at the same pace however far the camera has to travel. Rooms keep their own coordinates: only the one on its way out is displaced, with the camera moved with it so nothing stirs on screen. The last door of the chapter, which leads out of the run, still fades to black.

A portal turns over both doorways of a crossing: `room_pan::hang_a_portal` hangs one as a child of the door, so it stands where the door stands, rides the room off with it and goes the moment the door does. `flow::handle_door_touch` turns the one over the way out as it is used, and `room_intro::open_the_entry_door` the one over the door on the far side. Nobody stands in the new room until that portal gives them back: the hero forms out of its swirls over `room_flow.room_intro.player_fade_in_s` (`player_materialize`, its frames spread over that window rather than played at the sheet's own pace) and walks in once they are whole.

Every room stands the door the player came in through (`EntryDoor`) where their walk in begins: `room_intro` swings it open for them, then fades it away once they are in. The entrance starts `room_flow.room_intro.early_start_s` before the camera lands, so the door is already open and the player already stepping out of it as the room slides into frame; `room_intro::entrance_elapsed_s` is the one clock both halves read, and `start_room_intro` winds the intro timer forward by that much when the room was walked into through a door (`RoomPan::came_through_a_door`), so the walk carries on rather than starting over. The HUD stays up across the whole crossing (`GameScreen::shows_the_hud`), and the music and the city are heard through the wall while it lasts (`audio.crossing_music_muffle`).

The door out of room 0 leads through the prologue (`src/game/prologue.rs`): the flood rises over the room the run is leaving — the same water the main menu sits on, drawn in front of the room instead of behind it — the lines of `prologue.lines` are written out over it with `prologue.typing_sound` under them, and the water drains away over the slide into room 1. The words stand on a card of their own, read from the left, and every character holds its place from the first frame (the untyped ones are drawn as spaces in the monospace font), so the block never shifts under itself as it fills in. Nothing moves on by itself: the Enter button under the words is a `MenuAction::EnterTheCity` like any other button, so Enter or a click presses it — the first press finishes the writing, the next walks into the city. The words and their timings are all in `prologue:` of `config/gameplay.yaml`.

### Enemy material and armor

Each type in `enemies/enemy_types.yaml` declares a `material`, which decides the sound a hit on its body makes (`audio.recorded_effects.damage_*`), and its `health`, speed and damage, which every enemy of the type shares: a room cannot change them. Armor is the one thing a room gives an enemy. No type walks in armored; `armor_layers` on a room spawn (none when left out) is how many layers that one enemy has, and it is the `Armor layers` row of the debug editor's inspector. An armored enemy's health bar is gold with one shield icon per layer above it.

- Each layer soaks one ordinary blow whole: the layer breaks, with no health lost and no damage number, though the blow still wakes the enemy. The charged blow costs a layer too, and wounds the body behind it as well.
- Armor decides whether a hit interrupts. While any layer is up when the blow lands — even its last, even under the charged blow — the enemy carries on: no flinch, no knockback, and a wind-up runs to its strike. Without armor every hit that wounds interrupts: the wind-up is dropped and the enemy is held in its flinch. `combat/blows.rs` decides both.
- `EnemyBody::armor`, the layers left, is the whole state. `combat/shield.rs` watches it rather than being told: the frame it drops below an icon's layer, that icon starts its break and despawns once it has played out. The last icon of the row goes first.
- The icon is `art/ui/enemy_shield.typ`, a 6-frame sheet (frame 0 is the shield, 1-5 the break) whose `fps` in `assets/enemy_sheets.rs` sets how fast it shatters. Sprite sheets under `ui/` load with nearest sampling, unlike the character sheets.

### Art direction

`art/README.md` is the reference for how the game looks, and every asset — surface,
backdrop, character, UI piece, shader, effect — answers to it. Read it before
drawing anything or picking a colour, and change it first when the direction
itself changes. The short version:

- The city drowned after a collapse that left the power on. The flood is
  wine-dark, stained by what the city leaked into it, and the run is a climb out
  of it over ten chapters. The enemies are the machines nobody switched off.
- **The world is cold; only danger and help are warm.** Orange, red and warm
  white are reserved for attack wind-ups, projectiles, damage and pickups. Never
  spend them on decoration.
- **No world pixel goes brighter than `#6A6288`.** The exceptions are declared:
  light sources, the flood's crests, the player, and anything warm under the
  first rule.
- **The flood, or its light, is in every frame.**

Colour belongs in the YAML and in the Typst palettes, not in `art/README.md`:
the master palette there names the roles, and `art/_palettes.typ`,
`art/surfaces/textures/*/_*.typ`, `surfaces/materials.yaml` and
`config/gameplay.yaml` hold the values the game reads.

### Typst pixel art

Every surface texture is drawn in code with [Typst](https://typst.app) instead of a paint program, as are the HUD pieces in `src/assets/ui/`, the health drop in `src/assets/pickups/`, and the backdrop bands in `src/assets/background/drowned-city/`: the surface families are `city`, `metal`, `carved`, and `wood`. Sources live in `art/`, mirroring their output path under `src/assets/` (`art/surfaces/textures/wood/planks.typ` renders `src/assets/surfaces/textures/wood/planks.png`).

- `just art` renders every source (needs `typst` 0.15.1). The PNGs are committed, so building the game never needs Typst; rerun `just art` and commit both after editing a source. CI (`.github/workflows/art.yml`) re-renders with the same pinned Typst version and fails if any PNG differs from its source; bump `TYPST_VERSION` there when upgrading Typst locally.
- `art/_pixel.typ` makes a page exactly the image's size and draws 1 pt squares, rendered at 72 ppi: one square per pixel, no anti-aliasing. `pixels` draws text art (one character per pixel, looked up in a palette); `pixels-by` computes each pixel from a function.
- Files starting with `_` are shared libraries, not images: `art/_palettes.typ` for palettes used across families, `_<family>.typ` for a family's own palettes and patterns, `art/_pickups.typ` for the size of a pickup drawn in Typst and the rim that `outlined` draws around it (the health drop; the bonus item is the obelisk sheet `pickups/flying_obelisk.png`, the same for every category and told apart by its glow). A palette swap is a second short source reusing the same pattern (`dark_planks.typ`, `rusty_plates.typ`).
- Import libraries with relative paths.
- Format sources with typstyle 0.15.2 (`just art-fmt`, or an editor set to the same version): CI runs `typstyle --diff art` and fails on any unformatted file. Bump `TYPSTYLE_VERSION` in `art.yml` when upgrading.
- UI images load through `ui_style::load_pixel_art` (nearest sampling, so they stay crisp scaled up). Gray images can be tinted with `ImageNode::color`, like the ammo round.
- Keep to axis-aligned squares on whole points: curves, diagonal strokes, and text come out anti-aliased.
- Preview animated Typst sprite sheets as GIFs with `scripts/preview_animated_art.sh` (requires Typst and ImageMagick). It writes enlarged previews under `art/previews/`; update the script's frame sizes and delays when adding another animated sheet.

### Code layout

- `src/data.rs` loads and validates every YAML file under `src/assets/` into the `GameData` resource. `src/data/` holds one module per YAML source: `gameplay.rs` (`config/gameplay.yaml`), `enemies.rs`, `backgrounds.rs`, `bonuses.rs`, `rooms.rs`, `surfaces.rs` (`surfaces/materials.yaml`), `color.rs`, `player_animations.rs`.
- `src/game/combat/player_melee.rs` runs the melee chain; `src/game/impact.rs` holds the freeze and the camera shake a landed blow gives; `src/game/bonus_pickup.rs` lays out a cleared room's bonus item and opens the bonus screen when it is taken; `src/game/interact.rs` decides what Interact acts on and stands the key over it; `src/game/flow.rs` holds the rest of the room flow, `src/game/room_pan.rs` the slide from one room into the next, and `src/game/prologue.rs` the words the city is entered on.
- `src/game/menus.rs` draws the menus and `src/game/menus/` holds what their buttons do (`actions.rs`) and how they are walked and pressed (`input.rs`); `src/game/ripple.rs` holds the droplet that falls on a touched button.
- `src/game/shaders/` holds the 2D material shaders; `src/game/collision.rs` holds room collision; `src/game/room_art.rs` draws platforms and decorations (world-aligned tiled textures via a repeat sampler and `Sprite::rect`, or a plain color, and outlines of uncovered edges).
- `src/game/mod.rs` wires the `GamePlugin` (systems and states) and holds only shared items: `GameScreen`, `StartRoom`, `H_WORLD_UNITS`, small helpers. Components and resources live in the module that owns them (`session.rs` for `RunSession`, `sprites.rs` for sprite-sheet animation, `player/body.rs` for `PlayerBody`, `combat/enemy_body.rs` and `combat/components.rs` for combat entities, and so on) and are re-imported in `mod.rs` when other modules need them.
- Game modules reach shared items through `use super::*;`; add a `use` in `mod.rs` for a type that several modules need, or a direct `use crate::game::...` for a type only one other module needs.
- Tests live in `src/game/tests/` (gameplay, driven through a headless `App`) and in `#[cfg(test)]` modules next to the code they cover.

### Important note

Right now the focus in on making the first room feels very good before starting to implement the other ones.
