# Afterward

## Run

```sh
cargo run
```

## Headless Chapter 1 playthrough

Run the automated combat and room-flow check locally with:

```sh
cargo test --locked chapter_one_combat_run_reaches_completion_with_unique_bonus_choices
```

The test loads the room and enemy YAML, spawns each configured roster through the game's room spawner, defeats enemies with the real gun, projectile, and damage systems, chooses a bonus after each room, and completes the Warrior encounter. Its test pilot teleports and remains invulnerable, so it does not replace a manual visual playthrough.

For a local visual walkthrough, run:

```sh
cargo run -- --visual-walkthrough
```

The walkthrough opens a 1280×720 window, captures the main menu and each room's combat, clear, bonus, door, and transition screens, then exits after the Chapter One completion screen. Screenshots are saved under `target/visual-walkthrough/`. It clears enemies after capturing each room so the real room and UI flow can advance without a manual playthrough.

Focused headless tests also drive the default movement, jump, and dash controls, verify fullscreen can be remapped, verify repeated gunfire restarts the shooting animation, check the bold red received-damage styling, and confirm that the Swordsman's data-configured procedural gait animates while walking and returns to its idle pose when it stops.
