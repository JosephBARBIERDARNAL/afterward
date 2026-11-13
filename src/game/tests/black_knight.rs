use super::super::*;
use crate::game::combat::Stance;
use std::{path::Path, time::Duration};

use bevy::{prelude::MinimalPlugins, state::app::StatesPlugin, time::TimeUpdateStrategy};

#[test]
fn a_raised_guard_turns_the_chain_aside_and_answers_it() {
    let asset_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/assets");
    let mut data = GameData::load(&asset_root).expect("Chapter 1 data should load");
    // A flat hall, independent of the authored rooms, with the knight within
    // arm's reach of the player.
    *data.rooms.get_mut(&1).expect("room 1 should exist") = yaml_serde::from_str(
        "
room_id: 1
chapter_id: 1
size: { width_h: 48, height_h: 13.5 }
background_profiles: [clouds_bright_blue]
player_spawn: { x_h: 10, y_h: 1 }
platforms:
  - { material: city_pavement, rect_h: { x: 0, y: 0, width: 48, height: 1 } }
enemy_spawns:
  - { type_id: black_knight, position_h: { x: 11, y: 1 } }
bonus: { x_h: 43, y_h: 1 }
exit: { x_h: 45, y_h: 1 }
next_room_id: 2
",
    )
    .expect("the test layout should parse");
    let knight = &data.enemies.types["black_knight"];
    let parry_index = knight
        .attacks
        .iter()
        .position(|attack| attack.kind == "parry")
        .expect("the Black Knight should have a parry");
    let parry = &knight.attacks[parry_index];
    let counter_damage = parry.damage.expect("the parry's answer should hurt");
    let hold_s = parry.guard.as_ref().expect("a parry has a guard").hold_s;
    let full_health = knight.health;
    let mut sheets = data
        .player_animations
        .keyed()
        .into_iter()
        .map(|(key, clip)| {
            (
                key,
                SpriteSheet {
                    image: Handle::default(),
                    layout: Handle::default(),
                    frame_size: Vec2::new(128.0, 128.0),
                    frames: clip.frames,
                    fps: clip.fps,
                },
            )
        })
        .collect::<BTreeMap<_, _>>();
    sheets.insert(
        String::from("black_knight_unarmed_idle"),
        SpriteSheet {
            image: Handle::default(),
            layout: Handle::default(),
            frame_size: Vec2::new(150.0, 62.0),
            frames: 1,
            fps: 1.0,
        },
    );

    let mut app = App::new();
    app.add_plugins((MinimalPlugins, StatesPlugin))
        .insert_state(GameScreen::Playing)
        .insert_resource(data)
        .insert_resource(SpriteSheets(sheets))
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
            20,
        )))
        .insert_resource(RunSession {
            room_id: 1,
            ..default()
        })
        .insert_resource(RandomState(0x6b6e_6967_6874_2121))
        .init_resource::<SoundQueue>()
        .init_resource::<ControlBindings>()
        .insert_resource(PreferencesDir(None))
        .init_resource::<ButtonInput<KeyCode>>()
        .add_systems(Startup, controls::initialize_control_bindings)
        .add_systems(
            Startup,
            |mut commands: Commands, data: Res<GameData>, sheets: Res<SpriteSheets>| {
                let room = &data.rooms[&1];
                world::spawn_room_platforms(&mut commands, room, &data);
                world::spawn_player(&mut commands, room, &data, &sheets);
                world::spawn_enemies(&mut commands, room, &data, &sheets);
            },
        )
        .add_systems(
            Update,
            (
                player::update_player,
                combat::player_melee,
                combat::update_enemy_ai,
                combat::update_enemy_platform_physics,
            )
                .chain(),
        );
    app.update();

    // Stand the knight in its guard, then throw the chain at it.
    let mut knights = app.world_mut().query::<&mut EnemyBody>();
    let mut knight = knights
        .single_mut(app.world_mut())
        .expect("the layout should spawn only the knight");
    knight.engaged = true;
    knight.active_attack = parry_index;
    knight.attack_cooldown = 30.0;
    knight.stance = Some((Stance::Guard, hold_s));
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::KeyJ);

    let mut answered = false;
    for _ in 0..100 {
        app.update();
        let knight = knights
            .single(app.world())
            .expect("the knight should remain spawned");
        answered |= knight.countering;
        assert_eq!(
            (knight.health, knight.hurt_timer),
            (full_health, 0.0),
            "a blow on the guard, or on the answer to it, neither wounds nor interrupts"
        );
        if app.world().resource::<RunSession>().health < 100.0 {
            break;
        }
    }
    assert!(answered, "a blow on the guard should set off the counter");
    assert_eq!(
        app.world().resource::<RunSession>().health,
        100.0 - counter_damage,
        "the counter should land on the player who struck the guard"
    );
}

#[test]
fn a_beaten_knight_rolls_through_the_player_untouched() {
    let asset_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/assets");
    let mut data = GameData::load(&asset_root).expect("Chapter 1 data should load");
    // The same flat hall: the knight stands to the player's right.
    *data.rooms.get_mut(&1).expect("room 1 should exist") = yaml_serde::from_str(
        "
room_id: 1
chapter_id: 1
size: { width_h: 48, height_h: 13.5 }
background_profiles: [clouds_bright_blue]
player_spawn: { x_h: 10, y_h: 1 }
platforms:
  - { material: city_pavement, rect_h: { x: 0, y: 0, width: 48, height: 1 } }
enemy_spawns:
  - { type_id: black_knight, position_h: { x: 11, y: 1 } }
bonus: { x_h: 43, y_h: 1 }
exit: { x_h: 45, y_h: 1 }
next_room_id: 2
",
    )
    .expect("the test layout should parse");
    let knight = &data.enemies.types["black_knight"];
    let roll = knight.roll.as_ref().expect("the Black Knight should roll");
    let after_wounds = roll.after_wounds;
    let invulnerable_s = roll.invulnerable_from_s..=roll.invulnerable_to_s;
    let roll_s = roll.duration_s;
    let mut sheets = data
        .player_animations
        .keyed()
        .into_iter()
        .map(|(key, clip)| {
            (
                key,
                SpriteSheet {
                    image: Handle::default(),
                    layout: Handle::default(),
                    frame_size: Vec2::new(128.0, 128.0),
                    frames: clip.frames,
                    fps: clip.fps,
                },
            )
        })
        .collect::<BTreeMap<_, _>>();
    sheets.insert(
        String::from("black_knight_unarmed_idle"),
        SpriteSheet {
            image: Handle::default(),
            layout: Handle::default(),
            frame_size: Vec2::new(150.0, 62.0),
            frames: 1,
            fps: 1.0,
        },
    );

    let mut app = App::new();
    app.add_plugins((MinimalPlugins, StatesPlugin))
        .insert_state(GameScreen::Playing)
        .insert_resource(data)
        .insert_resource(SpriteSheets(sheets))
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
            20,
        )))
        .insert_resource(RunSession {
            room_id: 1,
            ..default()
        })
        .insert_resource(RandomState(0x6b6e_6967_6874_2121))
        .init_resource::<SoundQueue>()
        .init_resource::<ControlBindings>()
        .insert_resource(PreferencesDir(None))
        .init_resource::<ButtonInput<KeyCode>>()
        .add_systems(Startup, controls::initialize_control_bindings)
        .add_systems(
            Startup,
            |mut commands: Commands, data: Res<GameData>, sheets: Res<SpriteSheets>| {
                let room = &data.rooms[&1];
                world::spawn_room_platforms(&mut commands, room, &data);
                world::spawn_player(&mut commands, room, &data, &sheets);
                world::spawn_enemies(&mut commands, room, &data, &sheets);
            },
        )
        .add_systems(
            Update,
            (
                player::update_player,
                combat::player_melee,
                combat::update_enemy_ai,
                combat::update_enemy_platform_physics,
            )
                .chain(),
        );
    app.update();

    // Throw the chain at a knight with nothing in hand: every blow wounds
    // it, until it has taken as many as it stands for.
    let mut knights = app.world_mut().query::<(&mut EnemyBody, &Transform)>();
    let (mut knight, _) = knights
        .single_mut(app.world_mut())
        .expect("the layout should spawn only the knight");
    knight.engaged = true;
    knight.attack_cooldown = 30.0;
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::KeyJ);
    let mut players = app
        .world_mut()
        .query_filtered::<&Transform, With<PlayerBody>>();

    let mut health_when_it_rolled = None;
    let mut rolled_out = false;
    for _ in 0..400 {
        app.update();
        let (knight, transform) = knights
            .single(app.world())
            .expect("the knight should remain spawned");
        match (knight.stance, health_when_it_rolled) {
            (Some((Stance::Roll { .. }, left_s)), None) => {
                assert_eq!(
                    knight.wounds, 0,
                    "the roll spends the wounds that set it off"
                );
                assert!(invulnerable_s.contains(&(roll_s - left_s)) || left_s == roll_s);
                health_when_it_rolled = Some(knight.health);
            }
            (Some((Stance::Roll { .. }, left_s)), Some(health))
                if invulnerable_s.contains(&(roll_s - left_s)) =>
            {
                assert_eq!(
                    knight.health, health,
                    "no blow reaches the knight through the middle of its roll"
                );
            }
            (None, Some(_)) => {
                rolled_out = true;
                let player_x = players
                    .single(app.world())
                    .expect("the layout should spawn one player")
                    .translation
                    .x;
                assert!(
                    transform.translation.x < player_x,
                    "the roll should carry the knight to the player's back"
                );
                break;
            }
            _ => assert!(
                knight.wounds < after_wounds || knight.hurt_timer > 0.0,
                "the knight should roll as soon as its flinch lets it"
            ),
        }
    }
    assert!(
        rolled_out,
        "the knight should roll once it has been wounded {after_wounds} times"
    );
}
