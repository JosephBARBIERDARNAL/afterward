//! How an engaged enemy carries itself: it keeps to its floor, fires from
//! wherever it stands, and turns on a player it only heard.

use super::super::*;
use std::{path::Path, time::Duration};

use bevy::{prelude::MinimalPlugins, state::app::StatesPlugin, time::TimeUpdateStrategy};

#[test]
fn an_enemy_holds_its_ledge_until_the_player_stands_below_it() {
    let asset_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/assets");
    let mut data = GameData::load(&asset_root).expect("Chapter 1 data should load");
    // A fixed layout, independent of the authored rooms: a floor under two
    // ledges 4 H up, with a gap too wide to leap between them. The player
    // stands on one and a blade unit on the other.
    *data.rooms.get_mut(&3).expect("room 3 should exist") = yaml_serde::from_str(
        "
room_id: 3
chapter_id: 1
size: { width_h: 48, height_h: 27 }
background_profiles: [city_teal_evening]
player_spawn: { x_h: 11.5, y_h: 5 }
platforms:
  - { material: city_pavement, rect_h: { x: 0, y: 0, width: 48, height: 1 } }
  - { material: city_pavement, rect_h: { x: 5, y: 4, width: 7, height: 1 } }
  - { material: city_pavement, rect_h: { x: 20, y: 4, width: 10, height: 1 } }
enemy_spawns:
  - { type_id: blade_unit, position_h: { x: 21, y: 5 } }
bonus: { x_h: 43, y_h: 1 }
exit: { x_h: 45, y_h: 1 }
next_room_id: 4
",
    )
    .expect("the test layout should parse");
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
            )
                .chain(),
        );
    let stands_at_h = |app: &mut App| {
        let world = app.world_mut();
        let (enemy, transform) = world
            .query::<(&EnemyBody, &Transform)>()
            .single(world)
            .expect("the enemy should still be in the room");
        assert!(enemy.engaged, "the enemy should have seen the player");
        Vec2::new(transform.translation.x, enemy.home_y - enemy.feet_offset) / H_WORLD_UNITS
    };

    // The player is across the gap, at the enemy's own height: it walks to
    // the brink and no further.
    for _ in 0..80 {
        app.update();
    }
    let brink = stands_at_h(&mut app);
    assert!(
        (brink.y - 5.0).abs() < 0.01 && (19.5..=20.5).contains(&brink.x),
        "the enemy should hold the brink of its ledge, but stands at {brink} H"
    );

    // The player drops to the floor under it: now the way to them is down.
    let world = app.world_mut();
    world
        .query_filtered::<&mut Transform, With<PlayerBody>>()
        .single_mut(world)
        .expect("room should spawn one player")
        .translation
        .y = 1.5 * H_WORLD_UNITS;
    for _ in 0..80 {
        app.update();
    }
    let below = stands_at_h(&mut app);
    assert!(
        (below.y - 1.0).abs() < 0.01,
        "the enemy should have stepped down after the player, but stands at {below} H"
    );
}

#[test]
fn a_ranged_enemy_fires_at_a_player_standing_against_it_and_holds_its_ground() {
    let asset_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/assets");
    let mut data = GameData::load(&asset_root).expect("Chapter 1 data should load");
    // A fixed layout, independent of the authored rooms: a flat floor, and a
    // rifle unit with the player 1 H in front of it, far inside the distance
    // it closes to.
    *data.rooms.get_mut(&3).expect("room 3 should exist") = yaml_serde::from_str(
        "
room_id: 3
chapter_id: 1
size: { width_h: 48, height_h: 27 }
background_profiles: [city_teal_evening]
player_spawn: { x_h: 19, y_h: 1 }
platforms:
  - { material: city_pavement, rect_h: { x: 0, y: 0, width: 48, height: 1 } }
enemy_spawns:
  - { type_id: rifle_unit, position_h: { x: 20, y: 1 } }
bonus: { x_h: 43, y_h: 1 }
exit: { x_h: 45, y_h: 1 }
next_room_id: 4
",
    )
    .expect("the test layout should parse");
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
        .add_systems(Update, combat::update_enemy_ai);
    let enemy_x_h = |app: &mut App| {
        let world = app.world_mut();
        world
            .query_filtered::<&Transform, With<EnemyBody>>()
            .single(world)
            .expect("the enemy should still be in the room")
            .translation
            .x
            / H_WORLD_UNITS
    };
    let shots = |app: &mut App| {
        let world = app.world_mut();
        world
            .query::<&combat::CombatProjectile>()
            .iter(world)
            .count()
    };

    let mut fired_from_h = None;
    for _ in 0..20 {
        app.update();
        if shots(&mut app) > 0 {
            fired_from_h = Some(enemy_x_h(&mut app));
            break;
        }
    }
    assert_eq!(
        fired_from_h,
        Some(20.0),
        "the enemy should fire from where it stands, without backing away first"
    );

    // With the shot cooling down it holds its ground rather than backing away.
    for _ in 0..10 {
        app.update();
    }
    assert_eq!(
        enemy_x_h(&mut app),
        20.0,
        "the enemy should not back away while its next shot cools down"
    );
}

#[test]
fn a_blow_is_heard_by_the_idle_enemies_standing_near_the_one_it_lands_on() {
    let asset_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/assets");
    let mut data = GameData::load(&asset_root).expect("Chapter 1 data should load");
    // A fixed layout, independent of the authored rooms: a flat floor with
    // one body inside arm's reach, and two more with their backs to the
    // player, one within earshot of the first and one far beyond it.
    *data.rooms.get_mut(&1).expect("room 1 should exist") = yaml_serde::from_str(
        "
room_id: 1
chapter_id: 1
size: { width_h: 48, height_h: 13.5 }
background_profiles: [city_teal_evening]
player_spawn: { x_h: 2.5, y_h: 1 }
platforms:
  - { material: city_pavement, rect_h: { x: 0, y: 0, width: 48, height: 1 } }
enemy_spawns:
  - { type_id: blade_unit, position_h: { x: 3.05, y: 1 } }
  - { type_id: blade_unit, position_h: { x: 9, y: 1 }, facing: right }
  - { type_id: blade_unit, position_h: { x: 30, y: 1 }, facing: right }
bonus: { x_h: 43, y_h: 1 }
exit: { x_h: 45, y_h: 1 }
next_room_id: 2
",
    )
    .expect("the test layout should parse");
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
    for type_id in data.enemies.types.keys() {
        sheets.insert(
            type_id.idle_sheet().to_string(),
            SpriteSheet {
                image: Handle::default(),
                layout: Handle::default(),
                frame_size: Vec2::new(128.0, 128.0),
                frames: 1,
                fps: 1.0,
            },
        );
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
            room_id: 1,
            ..default()
        })
        .insert_resource(RandomState(0x7075_6e63_6821_2121))
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
                world::spawn_enemies(&mut commands, room, &data, &sheets, None);
            },
        )
        .add_systems(
            Update,
            (player::update_player, combat::player_melee).chain(),
        );
    app.update();

    let engaged = |app: &mut App| {
        let world = app.world_mut();
        let mut enemies = world
            .query::<(&EnemyBody, &Transform)>()
            .iter(world)
            .map(|(enemy, transform)| (transform.translation.x, enemy.engaged))
            .collect::<Vec<_>>();
        enemies.sort_by(|(left, _), (right, _)| left.total_cmp(right));
        enemies
            .into_iter()
            .map(|(_, engaged)| engaged)
            .collect::<Vec<_>>()
    };
    assert_eq!(engaged(&mut app), [false, false, false]);

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::KeyJ);
    for _ in 0..8 {
        app.update();
    }
    assert_eq!(
        engaged(&mut app),
        [true, true, false],
        "the blow wakes the body it lands on and the one near enough to hear it"
    );
}
