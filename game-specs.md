# Too Late — Game Specification

## 1. Game

**Too Late** is a small, single-player action roguelite set in a surreal reinterpretation of Egyptian mythology.

The player enters a procedurally generated world, explores branching paths, fights enemies in discrete arenas, creates a temporary build, collects permanent resources, and attempts to reach and defeat the final boss.

Death ends the run and destroys the temporary build. Collected permanent resources survive and are spent between runs to make the character stronger.

The central player feeling is:

> **One more run.**

The game is a hobby/side project designed to be **finished**, not an attempt to implement the full scope of a commercial roguelite.

---

## 2. Core Loop

1. Start a run.
2. Explore a procedurally generated branching world.
3. Enter a combat room.
4. Defeat all enemies.
5. Receive resources and/or choose a temporary upgrade.
6. Explore and choose the next path.
7. Take optional risks for better rewards.
8. Continue until the player dies or reaches the final boss.
9. On death, lose all temporary upgrades but keep permanent resources.
10. Spend resources on permanent upgrades.
11. Start another run.

A successful run ends by defeating the final boss.

---

## 3. Run Length

Early runs should naturally last approximately **5–10 minutes** because the player is weak and inexperienced.

As the player improves and gains permanent upgrades, runs become longer.

A successful late-game run should take approximately **40–60 minutes**.

The game supports suspending and resuming an active run.

---

## 4. World

The game contains **one world**.

It is a surreal reinterpretation of ancient Egypt and Egyptian mythology rather than a historically accurate representation.

The world becomes mechanically richer as the player progresses instead of introducing separate biomes.

Progression introduces:

- new enemies;
- stronger enemy combinations;
- elite enemies;
- new room layouts;
- hazards;
- optional challenges;
- secrets;
- new temporary upgrades.

---

## 5. World Generation

The world is procedurally generated for every run.

The player physically explores it rather than selecting nodes from an abstract map.

The structure is a branching network of rooms.

Rooms themselves are built from **handcrafted templates and components** combined procedurally.

The generator controls:

- room selection;
- room connections;
- branching;
- enemy encounters;
- hazards;
- rewards;
- optional paths;
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

When combat begins, exits lock. Defeating all enemies reopens them and grants a reward.

Exploration occurs between encounters and inside non-combat areas.

---

## 7. Perspective & Visual Style

The game uses a **top-down/isometric camera**.

There is no third-person camera and no important vertical traversal.

The visual target is **stylized pixel art**, with the readability and visual character of games such as _Dead Cells_, adapted to a Hades-like overhead perspective.

Existing asset packs are used during development.

All visual and audio assets must remain replaceable without changing gameplay code.

---

## 8. Player Character

There is one protagonist.

The protagonist has no defined gender or sex.

Character creation is not included.

Cosmetic customization is not required.

The protagonist travels to the transformed Egyptian world seeking **vengeance**.

During the journey, they gradually discover why the event they are avenging actually happened.

---

## 9. Controls

The game is designed **controller-first**, particularly for handheld play.

Core controls:

- left stick — movement;
- right stick — aim;
- primary attack;
- heavy attack;
- dash;
- interact.

Keyboard and mouse are supported during PC development.

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

A fast melee combo used as the standard offensive action.

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

---

## 11. Health

Health persists between rooms during a run.

Health does not automatically regenerate after combat.

Healing is scarce enough that taking damage matters.

The player should regularly evaluate whether an optional encounter is worth risking their remaining health.

Healing can appear as a room reward or exploration reward.

---

## 12. Temporary Builds

Every run creates a new temporary build.

The current prototype uses a pool of ten temporary room-clear bonuses. They may improve stats, restore health, or change how attacks work. These bonuses last for one run and are cleared when a new run starts.

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

When receiving an upgrade, the player chooses from a **small randomized selection**.

Use **three choices** initially.

The player cannot freely construct a predetermined build. Randomness deliberately forces experimentation.

The upgrade-selection system can only become more complicated if playtesting demonstrates a problem with the three-choice system.

---

## 14. Risk and Reward

Some optional paths are visibly more dangerous than the standard route.

Dangerous paths provide better rewards.

Possible rewards include:

- stronger temporary upgrades;
- permanent resources;
- healing;
- rare permanent bonuses;
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

The player keeps:

- permanent resources;
- discoveries;
- unlocked progression;
- rare permanent bonuses.

Death should hurt, but a failed run should still usually contribute to long-term progress.

---

## 16. Permanent Progression

Permanent progression primarily consists of **explicit numerical upgrades**.

Examples:

- increased maximum health;
- increased base damage;
- increased heavy-attack damage;
- improved healing;
- improved dash properties.

The player spends resources collected during runs to purchase these upgrades.

The character should become substantially stronger over time.

The game does **not** scale old enemies to compensate.

Early enemies that were once dangerous should eventually become easy.

---

## 17. Between Runs

There is **no physical hub**.

After death, the player enters a simple progression menu containing a shop/upgrade interface.

The flow is:

**Death → results → buy upgrades → new run.**

The interface should make restarting extremely quick.

---

## 18. Enemies

The finished game contains:

- **6 standard enemy types**;
- **2 elite variants**;
- **1 final boss**.

Standard enemies have clearly different behaviors rather than primarily different statistics.

The roster should include approximately:

1. aggressive melee pursuer;
2. heavy slow attacker;
3. ranged attacker;
4. mobile attacker;
5. area-denial enemy;
6. support/control enemy.

Their actual designs are derived from the Egyptian setting.

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

There is **one handcrafted final boss**.

The boss is never procedurally generated.

The encounter has deliberate attack patterns, phases, telegraphs, and mechanics.

The boss is directly connected to the protagonist's vengeance and the mystery behind it.

The design principle is:

> **Procedural journey. Handcrafted destination.**

Defeating the boss completes the game's primary objective.

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

- 1 world/theme;
- 1 playable character;
- 1 combat style;
- 6 standard enemies;
- 2 elite variants;
- 1 boss;
- 15–25 reusable room templates;
- 20–30 temporary upgrades;
- 10–15 permanent upgrades;
- several optional challenge/secret room variants.

These numbers are **scope limits**, not minimum quotas.

Do not add content simply to reach them.

---

## 23. Platform

Development targets **browser**, cross-platform.

Focus on a modern web stack for video games in the browser. If relevant, consider using Rust compiling to Wasm if that helps for performances.

---

## 24. Technology Requirements

The engine must support:

- strong 2D or 2.5D workflows;
- visual scene/room editing;
- procedural generation through code;
- animation;
- particles/effects;
- straightforward collision/physics;
- data-driven content;
- save games;
- good iteration speed.

**Finishing takes priority over programming-language preference.**

---

## 25. MVP

The first milestone is **not the roguelite**.

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

For the current prototype, use a fixed sequence of five handcrafted rooms that gets harder from room to room. After clearing each non-final room, offer three different temporary bonuses such as increased max health, increased base damage, healing, or a new combat power. Show chosen bonuses in the run HUD and clear them when a new run starts. Do not use high-risk/low-risk routes or Ash rewards in this prototype.

Add death/reset and basic rewards.

### Milestone 4 — Builds

Expand the randomized three-choice temporary-upgrade system to approximately ten options, mixing direct bonuses with upgrades that change attacks.

Use upgrades such as a wider heavy swing, a burning final combo hit, a final-hit burst, and life-stealing attacks to make builds feel different.

Playtest whether different choices meaningfully change combat and tune from feedback.

### Current combat tuning note

The ranged enemy currently aims at the player's earlier position before firing. Its attack feels too predictable; revisit its aim timing or variation in a later combat-tuning pass.

### Milestone 5 — Permanent Progression

Add persistent resources, saving and the between-run upgrade shop.

### Milestone 6 — Full Enemy Roster

Expand toward 6 enemies and 2 elites.

Add optional dangerous paths and exploration rewards.

### Milestone 7 — Boss

Create the final boss and make the game completable from beginning to end.

At this point, **Too Late is a game**.

### Milestone 8 — Content & Identity

Expand room/upgrades toward target numbers.

Introduce the surreal Egyptian setting and narrative discoveries.

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
- multiple worlds/biomes;
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
- a complete run can be played from start to boss;
- procedural runs provide meaningful variation;
- temporary builds noticeably change combat;
- permanent progression makes later runs easier;
- exploration creates meaningful risk/reward decisions;
- the final boss can be defeated;
- the vengeance story has a simple beginning, revelation and ending;
- progression saves correctly;
- the game is stable enough to give to friends;
- the game has coherent visuals and audio, even if some third-party assets remain.

Anything beyond this is optional.

---

## 29. Design Rule

When considering a new feature, ask:

> **Does Too Late need this to deliver fast combat, meaningful progression, procedural exploration, or "one more run"?**

If the answer is no:

**don't build it.**
