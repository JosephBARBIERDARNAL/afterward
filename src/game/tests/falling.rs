//! Falling out of the room: its lower edge is the end of the world.

use super::super::*;
use crate::data::{RoomConfig, SpawnPosition};
use std::{path::Path, time::Duration};

use bevy::{prelude::MinimalPlugins, state::app::StatesPlugin, time::TimeUpdateStrategy};

#[test]
fn a_player_who_falls_below_the_room_dies_at_once() {
    let asset_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/assets");
    let mut data = GameData::load(&asset_root).expect("Chapter 1 data should load");
    // The ledge below is spawned by hand around this spawn point, so the test
    // doesn't depend on room 1's layout.
    data.rooms
        .get_mut(&1)
        .expect("room 1 should be authored")
        .player_spawn = SpawnPosition { x_h: 2.5, y_h: 1.0 };
    let sheets = data
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
        .collect();
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, StatesPlugin))
        .insert_state(GameScreen::Playing)
        .insert_resource(data)
        .insert_resource(SpriteSheets(sheets))
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_micros(
            16_667,
        )))
        .insert_resource(RunSession {
            room_id: 1,
            ..default()
        })
        .init_resource::<SoundQueue>()
        .init_resource::<ControlBindings>()
        .insert_resource(PreferencesDir(None))
        .init_resource::<ButtonInput<KeyCode>>()
        .add_systems(Startup, controls::initialize_control_bindings)
        .add_systems(
            Startup,
            |mut commands: Commands, data: Res<GameData>, sheets: Res<SpriteSheets>| {
                world::spawn_player(&mut commands, &data.rooms[&1], &data, &sheets);
                // A ledge ending at 8 H, with nothing beyond it.
                commands.spawn(PlatformSurface {
                    left: 0.0,
                    right: 8.0 * H_WORLD_UNITS,
                    bottom: 0.0,
                    top: H_WORLD_UNITS,
                    footstep: FootstepSound::Stone,
                    tilt: None,
                });
            },
        )
        .add_systems(
            Update,
            (player::update_player, combat::update_falls).chain(),
        );
    app.update();

    // Walking off the end of the ledge drops the player out of the room.
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::KeyD);
    for _ in 0..180 {
        app.update();
        let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        keys.clear();
        keys.press(KeyCode::KeyD);
    }
    assert_eq!(app.world().resource::<RunSession>().health, 0.0);
    assert_eq!(
        *app.world().resource::<State<GameScreen>>().get(),
        GameScreen::Dead,
        "the run ends as soon as the player is out of sight below the room"
    );
}

#[test]
fn a_chasing_enemy_stops_at_a_bottomless_edge_instead_of_walking_in() {
    let asset_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/assets");
    let mut data = GameData::load(&asset_root).expect("Chapter 1 data should load");
    // A fixed layout, independent of the authored rooms: two ledges with a
    // gap too wide to leap between them, the player across it from one blade unit.
    let room = data.rooms.get_mut(&3).expect("room 3 should exist");
    let layout: RoomConfig = yaml_serde::from_str(
        "
room_id: 3
chapter_id: 1
size: { width_h: 48, height_h: 27 }
background_profiles: [city_teal_evening]
player_spawn: { x_h: 17, y_h: 1 }
platforms:
  - { material: city_pavement, rect_h: { x: 0, y: 0, width: 10, height: 1 } }
  - { material: city_pavement, rect_h: { x: 16, y: 0, width: 32, height: 1 } }
enemy_spawns:
  - { type_id: blade_unit, position_h: { x: 9, y: 1 } }
bonus: { x_h: 43, y_h: 1 }
exit: { x_h: 45, y_h: 1 }
next_room_id: 4
",
    )
    .expect("the test layout should parse");
    *room = layout;
    let placeholder_sheet = SpriteSheet {
        image: Handle::default(),
        layout: Handle::default(),
        frame_size: Vec2::new(128.0, 128.0),
        frames: 1,
        fps: 1.0,
    };
    let mut sheets = BTreeMap::from([(String::from("player_idle"), placeholder_sheet.clone())]);
    for type_id in data.enemies.types.keys() {
        sheets.insert(type_id.idle_sheet().to_string(), placeholder_sheet.clone());
    }

    let mut app = App::new();
    app.add_plugins((MinimalPlugins, StatesPlugin))
        .insert_state(GameScreen::Playing)
        .insert_resource(data)
        .insert_resource(SpriteSheets(sheets))
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
            50,
        )))
        .insert_resource(RunSession {
            room_id: 3,
            ..default()
        })
        .init_resource::<SoundQueue>()
        .add_systems(
            Startup,
            |mut commands: Commands, data: Res<GameData>, sheets: Res<SpriteSheets>| {
                let room = &data.rooms[&3];
                world::spawn_room_platforms(&mut commands, room, &data);
                world::spawn_player(&mut commands, room, &data, &sheets);
                world::spawn_enemies(&mut commands, room, &data, &sheets, None);
            },
        )
        .add_systems(
            Update,
            (
                combat::update_enemy_ai,
                combat::update_enemy_platform_physics,
                combat::update_falls,
            )
                .chain(),
        );
    for _ in 0..80 {
        app.update();
    }

    let world = app.world_mut();
    let (enemy, transform) = world
        .query::<(&EnemyBody, &Transform)>()
        .single(world)
        .expect("the enemy should still be in the room");
    let feet_h = (enemy.home_y - enemy.feet_offset) / H_WORLD_UNITS;
    let x_h = transform.translation.x / H_WORLD_UNITS;
    assert!(
        (feet_h - 1.0).abs() < 0.01 && (9.0..=10.5).contains(&x_h),
        "the enemy should hold the brink of the gap, but stands at ({x_h}, {feet_h}) H"
    );
}

#[test]
fn an_enemy_that_falls_out_of_the_room_counts_as_down() {
    let asset_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/assets");
    let mut data = GameData::load(&asset_root).expect("Chapter 1 data should load");
    // A fixed layout, independent of the authored rooms: a ledge, and one
    // blade unit left hanging in the air over the gap beyond it.
    let room = data.rooms.get_mut(&3).expect("room 3 should exist");
    let layout: RoomConfig = yaml_serde::from_str(
        "
room_id: 3
chapter_id: 1
size: { width_h: 48, height_h: 27 }
background_profiles: [city_teal_evening]
player_spawn: { x_h: 2.5, y_h: 1 }
platforms:
  - { material: city_pavement, rect_h: { x: 0, y: 0, width: 10, height: 1 } }
enemy_spawns:
  - { type_id: blade_unit, position_h: { x: 20, y: 6 } }
bonus: { x_h: 43, y_h: 1 }
exit: { x_h: 45, y_h: 1 }
next_room_id: 4
",
    )
    .expect("the test layout should parse");
    *room = layout;
    let placeholder_sheet = SpriteSheet {
        image: Handle::default(),
        layout: Handle::default(),
        frame_size: Vec2::new(128.0, 128.0),
        frames: 1,
        fps: 1.0,
    };
    let mut sheets = BTreeMap::from([(String::from("player_idle"), placeholder_sheet.clone())]);
    for type_id in data.enemies.types.keys() {
        sheets.insert(type_id.idle_sheet().to_string(), placeholder_sheet.clone());
    }

    let mut app = App::new();
    app.add_plugins((MinimalPlugins, StatesPlugin))
        .insert_state(GameScreen::Playing)
        .insert_resource(data)
        .insert_resource(SpriteSheets(sheets))
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
            50,
        )))
        .insert_resource(RunSession {
            room_id: 3,
            ..default()
        })
        .insert_resource(PhaseTimer(Timer::from_seconds(0.0, TimerMode::Once)))
        .init_resource::<SoundQueue>()
        .add_systems(
            Startup,
            |mut commands: Commands, data: Res<GameData>, sheets: Res<SpriteSheets>| {
                let room = &data.rooms[&3];
                world::spawn_room_platforms(&mut commands, room, &data);
                world::spawn_enemies(&mut commands, room, &data, &sheets, None);
            },
        )
        .add_systems(
            Update,
            (
                combat::update_enemy_platform_physics,
                combat::update_falls,
                flow::room_clear_check,
            )
                .chain(),
        );
    for _ in 0..40 {
        app.update();
    }

    let world = app.world_mut();
    assert_eq!(
        world.query::<&EnemyBody>().iter(world).count(),
        0,
        "the fall should take the enemy out of the room for good"
    );
    assert!(
        app.world().resource::<RunSession>().room_cleared,
        "a room whose last enemy pits itself still clears"
    );
}
