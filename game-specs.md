# Too Late — Game Specification

## 1. Game

**Too Late** is a single-player action roguelike set in a surreal world whose visual identity changes every ten rooms. Egyptian mythology remains a story influence across the journey rather than the look of every region.

The player enters a surreal world, fights through a 100-room run in ten themed chapters, creates a temporary build, and attempts to defeat the boss at the end of each chapter. The final boss waits in room 100.

Death ends the run and destroys the temporary build. Nothing earned or chosen in one run carries into another: there is no currency, permanent upgrade tree, unlock, or account progression.

The central player feeling is:

> **One more run.**

The game is a hobby/side project designed to be **finished**, not an attempt to implement the full scope of a commercial roguelike.

---

## 2. Core Loop

1. Start in the empty safe room and explicitly open its door to begin a run.
2. Advance through a 100-room run divided into ten chapters of ten rooms each.
3. Enter a combat room; the tenth room of every chapter is a solo boss encounter.
4. Defeat all enemies.
5. Choose a temporary upgrade after eligible rooms.
6. Continue to the next room and chapter.
7. Choose among different temporary bonuses after eligible rooms.
8. Continue until the player dies or defeats the boss in room 100.
9. On death, lose the entire temporary build and return to the safe room.
10. Start another run from the safe room with base stats and no carried resources.

A successful run contains exactly **100 rooms** and ends by defeating the final boss in room 100. Rooms 10, 20, 30, 40, 50, 60, 70, 80, 90, and 100 are boss rooms.

---

## 3. Run Length

The complete run is a 100-room journey divided into ten chapters. Early runs will often end before the player reaches later chapters because the player is weak and inexperienced.

Runs become longer as the player learns enemy patterns and improves their decisions and execution.

A successful late-game run should take approximately **40–60 minutes**, subject to playtesting across all 100 rooms and ten bosses.

Suspending and resuming an active run is not part of the current prototype.

---

## 4. World

The game contains one surreal world divided into **ten chapters**. Each chapter covers ten rooms and has a distinct overall visual style. Example styles include jungle, Hong Kong, and the moon; the final set and order of all ten styles remain to be designed.

Egyptian mythology remains a story influence, but does not restrict the look of each chapter. The visual changes can represent surreal shifts in the same connected world rather than separate playable worlds.

The 100-room journey introduces:

- new enemies;
- stronger enemy combinations;
- elite enemies;
- new room layouts and chapter art styles;
- hazards;
- optional challenges;
- secrets;
- new temporary upgrades that are lost at the end of the run.

---

## 5. World Generation

The world is procedurally generated for every run.

The player physically explores the rooms rather than selecting nodes from an abstract map.

The core run advances through exactly 100 rooms. Optional details inside rooms may reward exploration, but they do not change the fixed position of each tenth-room boss.

Rooms themselves are built from **handcrafted templates and components** combined procedurally.

The generator controls:

- room selection;
- enemy encounters;
- hazards;
- rewards;
- optional in-room paths;
- secrets;
- decorative variation.

The objective is not mathematical uniqueness. It is to make repeated runs rarely feel identical.

Procedural generation must follow authored rules. It must never create unavoidable damage, inaccessible exits, broken encounters, or obviously nonsensical spaces.

---

## 6. Rooms

The world primarily contains:

- combat rooms;
- elite combat rooms;
- resource/reward rooms;
- exploration rooms;
- optional challenge rooms;
- secret rooms;
- the boss arena.

Combat rooms are discrete encounters.

When combat begins, exits lock. Defeating all enemies awards the room reward and makes the east door available. Approach that door and press E to open it; opening the door presents the room's bonus choice. Choosing a bonus lets the player pass through the open exit. Walking into a closed door never opens it by itself.

Exploration occurs between encounters and inside non-combat areas.

---

## 7. Perspective & Visual Style

The game uses a **top-down/isometric camera**.

There is no third-person camera and no important vertical traversal.

The visual target is **stylized pixel art**, with the readability and visual character of games such as _Dead Cells_, adapted to a Hades-like overhead perspective.

Keep UI text crisp and readable at the target window size; avoid rendering or scaling that makes it look blurry.

Existing asset packs are used during development.

All visual and audio assets must remain replaceable without changing gameplay code.

---

## 8. Player Character

There is one protagonist.

The protagonist has no defined gender or sex.

Character creation is not included.

Cosmetic customization is not required.

The protagonist travels through the transformed surreal world seeking **vengeance**.

During the journey, they gradually discover why the event they are avenging actually happened.

---

## 9. Controls

The current prototype and Bevy migration target native desktop play with keyboard and mouse.

Core controls:

- arrows or ZQSD — movement;
- mouse — aim;
- J — primary attack;
- K — heavy attack;
- Space — dash;
- E — interact with a nearby door.
- Escape — open or close the pause menu, which offers Continue, Restart Run, Bonus, and Back Home.

Give each action one clear keyboard path. Avoid adding redundant shortcuts or click alternatives for the same action. Keep the requested arrow and ZQSD movement bindings for keyboard play. Controller input and browser/Wasm builds are outside the Bevy migration scope.

Attacks use explicit directional aiming. The game does not automatically select targets for the player.

---

## 10. Combat

Combat is:

- fast;
- melee-focused;
- mobility-heavy;
- directional;
- skill-based.

Game 1 contains exactly **one combat style**.

The player has three core combat actions.

### Primary Attack

A fast melee combo used as the standard offensive action. Each strike is a broad horizontal sword sweep in the aimed direction, with a visible slash that covers the attack area.

### Heavy Attack

A slower, high-damage attack.

Its fundamental trade-off is:

> **High damage in exchange for vulnerability.**

Using it at the wrong moment should expose the player to damage.

### Dash

The player's primary defensive ability.

The dash is used to:

- avoid attacks;
- reposition;
- close distance;
- escape dangerous situations.

There is **no block or parry system**.

Movement and dash mastery are major components of player skill.

### Hit Feedback

Show a brief floating damage number whenever the player damages an enemy or takes damage. Damage dealt appears over the enemy; damage received appears over the player in bold red. Use a quick pop, upward drift, and fade so the numbers are easy to notice without lingering over the fight.

Keep the player's weapon visible in the direction they are facing, including when they take damage. Give primary attacks, the heavy attack windup and strike, and the dash a brief visual cue. Enemy attack warnings must clearly grow and pulse before the attack lands.

---

## 11. Health

Health persists between rooms during a run.

Keep the player's current and maximum health visible in the run HUD. Show the remaining health of each living enemy in a small bar above it, and remove the bar when that enemy is defeated.

Health does not automatically regenerate after combat.

Healing is scarce enough that taking damage matters.

The player should regularly evaluate whether an optional encounter is worth risking their remaining health.

Healing can appear as a room reward or exploration reward.

---

## 12. Temporary Builds

Every run creates a new temporary build.

The current prototype uses a pool of ten temporary room-clear bonuses. They may improve stats, restore health, or change how attacks work. These bonuses last for one run and are cleared when a new run starts.

Each bonus has a distinct name. Picked bonuses appear in the run HUD on separate lines. The pause menu's Bonus button opens a catalog of all bonuses and their effects, with the current run's picks marked. Both the HUD list and picked status reset when a new run starts.

Later upgrades should primarily **modify existing actions** so that they visibly change how the player fights, instead of relying only on numerical increases.

Examples:

- dash damages enemies;
- dash leaves an effect behind;
- final combo hit creates an explosion;
- heavy attack gains additional range;
- heavy attack hits an area;
- primary attacks inflict a status effect.

These examples are not mandatory individual upgrades; they define the intended design philosophy.

Upgrades should visibly change how the player fights.

---

## 13. Obtaining Temporary Upgrades

Temporary upgrades come from both **combat and exploration**.

Combat provides reliable opportunities to improve the build.

Exploration and optional dangerous paths provide additional or rarer opportunities.

The current prototype includes a bonus cache in The Crossing. After clearing the room, find the cache and press E to open it and receive one additional three-choice run bonus. This reward is separate from the east-door bonus. It has no currency value and does not persist after the run.

When receiving an upgrade, the player chooses from a **small randomized selection**.

Use **three choices** initially.

The player cannot freely construct a predetermined build. Randomness deliberately forces experimentation.

The upgrade-selection system can only become more complicated if playtesting demonstrates a problem with the three-choice system.

---

## 14. Risk and Reward

Some optional paths may be visibly more dangerous than the standard route and can hold additional exploration rewards. These rewards are separate from the room-clear bonus choice.

When a bonus choice appears, its options vary by effect—such as maximum health, attack damage, healing, or a new power—not by high-risk and low-risk reward tiers. Bonuses last only for the current run.

Possible rewards include:

- stronger temporary upgrades;
- healing;
- secrets;
- narrative discoveries.

The game should regularly create situations such as:

> "I'm low on health, but that reward might be worth risking the run."

---

## 15. Death

Death immediately ends the run.

The player loses:

- all temporary upgrades;
- the current build;
- temporary run-specific effects.

Nothing from the run carries over after death. There is no permanent progression between runs. Death should make the player want to understand what went wrong and try again.

---

## 16. Between-Run Progression

There is no persistent progression between runs. Do not add money, a shop, permanent stat upgrades, unlocks, or saved bonuses. A new run always starts with the same base character and a clean temporary build.

---

## 17. Between Runs

The game opens in a safe room with no enemies. Its center stays open for movement, and its door is the sole interaction that starts a run. The safe room has no shop or currency display.

After death, completing room 100, or choosing Back Home in the pause menu, return to the safe room. Starting another run always requires an explicit door interaction and resets health, stats, bonuses, and run-specific effects.

The flow is:

**Game starts / run ends → safe room → move to the door and press E → new run.**

The interface should make restarting extremely quick without carrying power from earlier runs.

---

## 18. Enemies

The finished game contains:

- **6 standard enemy types**;
- **2 elite variants**;
- **10 handcrafted bosses**, one for each ten-room chapter.

Standard enemies have clearly different behaviors rather than primarily different statistics.

The roster should include approximately:

1. aggressive melee pursuer;
2. heavy slow attacker;
3. ranged attacker;
4. mobile attacker;
5. area-denial enemy;
6. support/control enemy.

Their designs fit the visual style of the chapter where they appear, with recurring motifs connecting them to the Egyptian mythology story influence.

Not every enemy appears from the beginning. New enemies enter the encounter pool as progression advances.

Enemy combinations provide much of the late-game combat complexity.

---

## 19. Elite Enemies

Two standard enemies receive elite versions.

Elite enemies modify behavior, not merely health.

They can be:

- faster;
- more aggressive;
- equipped with additional attacks;
- capable of modifying the arena;
- harder to approach safely.

Elite encounters provide increased rewards.

---

## 20. Boss

There are **10 handcrafted solo bosses**, in rooms 10, 20, 30, 40, 50, 60, 70, 80, 90, and 100. Every boss room contains only its boss. Each boss looks distinct from standard enemies and has its own skillset, attack patterns, telegraphs, and mechanics. Bosses are never procedurally generated.

The bosses are connected to the protagonist's vengeance and the mystery behind it. The room 100 boss resolves the central conflict.

The design principle is:

> **Procedural rooms. Handcrafted bosses.**

Defeating the boss in room 100 completes the game's primary objective.

The player may continue playing afterward.

---

## 21. Story

Story is secondary to gameplay.

There are no long mandatory cutscenes and no large dialogue system.

The player discovers the story primarily through:

- environments;
- secret rooms;
- objects;
- visual details;
- short pieces of text;
- rare discoveries;
- boss-related information.

A player ignoring most story content should still enjoy the game.

A player paying attention should gradually understand that the protagonist's initial interpretation of the event behind their vengeance was incomplete.

---

## 22. Content Target

The finished game contains approximately:

- 1 connected world with 10 chapter styles, each covering 10 rooms;
- 1 playable character;
- 1 combat style;
- 6 standard enemies;
- 2 elite variants;
- 10 handcrafted solo bosses, with the room 100 boss as the final boss;
- 15–25 reusable room templates;
- 20–30 temporary upgrades;
- several optional challenge/secret room variants.

These numbers are **scope limits**, not minimum quotas.

Do not add content simply to reach them.

---

## 23. Platform

The project is migrating to **Bevy with Rust**. The current migration target is a native desktop build with keyboard and mouse controls.

---

## 24. Technology Requirements

Bevy must support:

- strong 2D or 2.5D workflows;
- visual scene/room editing;
- procedural generation through code;
- animation;
- particles/effects;
- straightforward collision/physics;
- data-driven content;
- replaceable placeholder assets;
- good iteration speed.

Use Bevy's built-in 2D shapes as temporary visuals. The [Bevy learning index](https://bevy.org/assets/#learning) and [Bevy 2D assets index](https://bevy.org/assets/#2d) are starting references. The 2D assets index is a community plugin directory, not a downloadable art pack; evaluate plugins only when a concrete need appears. Current placeholders follow the official [2D Shapes example](https://github.com/bevyengine/bevy/blob/latest/examples/2d/2d_shapes.rs). Keep visuals and audio replaceable without changing combat rules.

---

## 25. MVP

The first milestone is **not the full 100-room game**.

It is one combat arena.

It contains:

- player movement;
- directional aiming;
- primary melee combo;
- heavy attack;
- dash;
- health/damage;
- one melee enemy;
- one ranged enemy;
- death/restart;
- placeholder graphics;
- basic sound effects.

No procedural generation.

No upgrades.

No permanent progression.

No story.

No boss.

No Egyptian content is required.

The MVP has exactly one question to answer:

> **Is fighting these enemies fun enough that I voluntarily restart after dying?**

Do not build the rest of the game until the answer is reasonably close to **yes**.

---

## 26. Development Order

### Milestone 1 — Combat

Build the MVP above.

Polish movement, attacks, dash, hit feedback and enemies until combat is fun.

### Milestone 2 — Rooms

Create several combat rooms and 3 enemy types.

Add room completion and transitions.

### Milestone 3 — Runs

The current prototype uses a fixed sequence of five handcrafted rooms that gets harder from room to room. This is a prototype slice only. The end goal is a 100-room run divided into ten chapters, with a solo boss in every tenth room and a distinct world style for each chapter. Do not build all 100 rooms as part of the current prototype milestone.

After eligible rooms, offer three different temporary bonuses such as increased max health, increased base damage, healing, or a new combat power. Show chosen bonuses in the run HUD and clear them when a new run starts. Do not add money or any other progression that persists between runs.

Add death/reset and basic rewards.

### Milestone 4 — Builds

Expand the randomized three-choice temporary-upgrade system to approximately ten options, mixing direct bonuses with upgrades that change attacks.

Use upgrades such as a wider heavy swing, a burning final combo hit, a final-hit burst, and life-stealing attacks to make builds feel different.

Playtest whether different choices meaningfully change combat and tune from feedback.

### Current combat tuning note

The ranged enemy currently aims at the player's earlier position before firing. Its attack feels too predictable; revisit its aim timing or variation in a later combat-tuning pass.

### Milestone 5 — Bevy Migration

Port the current five-room prototype to Bevy for native desktop keyboard and mouse play while preserving its combat, authored room sequence and obstacles, six enemy types, run-only bonuses, Crossing bonus cache, safe-room door, pause flow, and hit feedback. Obstacle geometry must block player and enemy movement, projectile travel, and attacks without clear line of sight. Remove currency, the shop, permanent upgrades, and persistence between runs. Use replaceable Bevy 2D shape placeholders while assets are pending. Browser/Wasm builds and controller input are outside this migration. The Phaser/Vite/Bun implementation and its dependencies have been removed.

The current Bevy build includes the five authored rooms and their obstacle geometry, basic collision and line-of-sight, the Crossing cache, generated versions of the prototype's gameplay sound cues, and a reset that clears run bonuses when returning home or starting another run. User playtesting remains outstanding before balance tuning.

### Milestone 6 — Full Enemy Roster

The current prototype has six enemy types: melee, ranged, heavy, charger, skirmisher, and bomber. It introduces two elite behaviors in later rooms: a faster, tougher Vanguard charger and a Volley ranged enemy that fires a three-shot spread. Playtest these patterns before tuning their balance.

Continue expanding the roster toward the ten-chapter progression and add more exploration rewards. Keep bonus choices differentiated by effect; every reward is limited to its current run.

### Milestone 7 — Boss

Create handcrafted solo bosses for rooms 10 through 100, each with a distinct look and skillset. Make room 100 the final boss and the end of the 100-room run.

At this point, **Too Late is a game**.

### Milestone 8 — Content & Identity

Expand the prototype toward the complete 100-room, ten-chapter run and the target content numbers. Give each ten-room chapter its own visual style and introduce narrative discoveries across the journey.

Replace placeholder assets where worthwhile.

### Milestone 9 — Polish

Balance progression, improve effects/audio/UI, optimize performance, fix bugs and give the game to friends to play.

Then call it finished.

---

## 27. Explicitly Out of Scope

Too Late does **not** include:

- multiplayer;
- online features;
- multiple characters;
- multiple weapons/combat styles;
- character creation;
- large physical hub;
- NPC relationship systems;
- an open-ended or infinite run beyond the 100-room ending;
- open world;
- climbing;
- meaningful vertical exploration;
- procedural bosses;
- extensive quests;
- large dialogue system;
- voice acting;
- literal infinite generation;
- mod support;
- Steam release requirements;
- Nintendo release requirements;
- mobile support.

---

## 28. Definition of Done

Too Late is finished when:

- combat feels responsive and satisfying;
- a complete 100-room run can be played from start to the final boss in room 100;
- procedural runs provide meaningful variation;
- temporary builds noticeably change combat;
- each new run starts from the same base stats with no carried currency or bonuses;
- exploration creates meaningful risk/reward decisions;
- all ten solo bosses can be defeated, including the final boss in room 100;
- the vengeance story has a simple beginning, revelation and ending;
- the Bevy build runs reliably on its supported platforms;
- the game is stable enough to give to friends;
- the game has coherent visuals and audio, even if some third-party assets remain.

Anything beyond this is optional.

---

## 29. Design Rule

When considering a new feature, ask:

> **Does Too Late need this to deliver fast combat, meaningful progression, procedural exploration, or "one more run"?**

If the answer is no:

**don't build it.**
