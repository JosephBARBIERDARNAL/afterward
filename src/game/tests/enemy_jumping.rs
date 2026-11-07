use super::super::*;
use crate::data::RoomConfig;
use std::{path::Path, time::Duration};

use bevy::{prelude::MinimalPlugins, state::app::StatesPlugin, time::TimeUpdateStrategy};

#[test]
fn ground_enemy_climbs_the_staircase_to_a_player_on_a_higher_platform() {
    let asset_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/assets");
    let mut data = GameData::load(&asset_root).expect("Chapter 1 data should load");
    // A fixed layout, independent of the authored rooms: a floor, two plank
    // steps, and a walkway whose top is 7 H up, with one ground Gorgon.
    let room = data.rooms.get_mut(&3).expect("room 3 should exist");
    let layout: RoomConfig = yaml_serde::from_str(
        "
room_id: 3
chapter_id: 1
size: { width_viewports: 2, height_h: 27 }
background_profile: clouds_bright_blue
player_spawn: { x_h: 2.5, y_h: 1 }
platforms:
  - { material: city_pavement, rect_h: { x: 0, y: 0, width: 48, height: 1 } }
  - { material: city_pavement, rect_h: { x: 13, y: 6, width: 14, height: 1 } }
  - { material: city_plating, rect_h: { x: 6, y: 2.75, width: 3, height: 0.25 } }
  - { material: city_plating, rect_h: { x: 10, y: 4.75, width: 3, height: 0.25 } }
enemy_spawns:
  - { type_id: gorgon_1, position_h: { x: 8, y: 1 } }
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
        .insert_resource(data)
        .insert_resource(SpriteSheets(sheets))
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
            50,
        )))
        .init_state::<GameScreen>()
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
                world::spawn_player(
                    &mut commands,
                    room,
                    &data,
                    &sheets,
                    data.gameplay.player.gun.magazine_size,
                );
                world::spawn_enemies(&mut commands, room, &data, &sheets);
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
    app.update();

    // Stand the player on the walkway, at x = 18.
    let mut enemies = app.world_mut().query_filtered::<Entity, With<EnemyBody>>();
    let climber = enemies
        .single(app.world())
        .expect("the layout should spawn one Gorgon");
    let player_feet_y = 7.0 * H_WORLD_UNITS;
    let mut players = app
        .world_mut()
        .query_filtered::<&mut Transform, With<PlayerBody>>();
    players
        .single_mut(app.world_mut())
        .expect("room should spawn one player")
        .translation = Vec3::new(
        18.0 * H_WORLD_UNITS,
        player_feet_y + 0.5 * H_WORLD_UNITS,
        4.0,
    );

    let mut jumped = false;
    let mut reached_player_level = false;
    for _ in 0..200 {
        app.update();
        let enemy = app
            .world()
            .get::<EnemyBody>(climber)
            .expect("the Gorgon should remain spawned");
        jumped |= enemy.jump.is_some();
        let feet_y = enemy.home_y - enemy.feet_offset;
        if enemy.grounded && (feet_y - player_feet_y).abs() <= 1.0 {
            reached_player_level = true;
            break;
        }
    }
    assert!(jumped, "the Gorgon should jump instead of waiting below");
    assert!(
        reached_player_level,
        "the Gorgon should climb onto the player's platform"
    );
}

#[test]
fn evil_wizard_jump_slam_damages_the_player_on_landing() {
    let asset_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/assets");
    let mut data = GameData::load(&asset_root).expect("Chapter 1 data should load");
    // A flat hall, independent of the authored boss room.
    let room = data.rooms.get_mut(&10).expect("room 10 should exist");
    let layout: RoomConfig = yaml_serde::from_str(
        "
room_id: 10
chapter_id: 1
size: { width_viewports: 2, height_h: 22.5 }
background_profile: clouds_thunderstorm
player_spawn: { x_h: 2.5, y_h: 1 }
platforms:
  - { material: city_pavement, rect_h: { x: 0, y: 0, width: 48, height: 1 } }
enemy_spawns:
  - { type_id: evil_wizard, position_h: { x: 18, y: 1 } }
exit: { x_h: 46, y_h: 1 }
next_room_id: 11
",
    )
    .expect("the test layout should parse");
    *room = layout;
    let slam_index = data.enemies.types["evil_wizard"]
        .attacks
        .iter()
        .position(|attack| attack.kind == "jump_slam")
        .expect("the Evil Wizard should have a jump slam");
    let placeholder_sheet = SpriteSheet {
        image: Handle::default(),
        layout: Handle::default(),
        frame_size: Vec2::new(250.0, 250.0),
        frames: 1,
        fps: 1.0,
    };
    let sheets = SpriteSheets(BTreeMap::from([
        (String::from("player_idle"), placeholder_sheet.clone()),
        (String::from("boss_idle"), placeholder_sheet),
    ]));

    let mut app = App::new();
    app.add_plugins((MinimalPlugins, StatesPlugin))
        .insert_resource(data)
        .insert_resource(sheets)
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
            20,
        )))
        .init_state::<GameScreen>()
        .insert_resource(RunSession {
            room_id: 10,
            ..default()
        })
        .init_resource::<SoundQueue>()
        .add_systems(
            Startup,
            |mut commands: Commands, data: Res<GameData>, sheets: Res<SpriteSheets>| {
                let room = &data.rooms[&10];
                world::spawn_room_platforms(&mut commands, room, &data);
                world::spawn_player(
                    &mut commands,
                    room,
                    &data,
                    &sheets,
                    data.gameplay.player.gun.magazine_size,
                );
                world::spawn_enemies(&mut commands, room, &data, &sheets);
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
    app.update();

    let mut bosses = app.world_mut().query::<&mut EnemyBody>();
    let mut boss = bosses
        .single_mut(app.world_mut())
        .expect("room 10 should spawn only the Evil Wizard");
    boss.next_attack = slam_index;
    boss.engaged = true;

    let mut slammed = false;
    for _ in 0..500 {
        app.update();
        let mut bosses = app.world_mut().query::<&EnemyBody>();
        let boss = bosses
            .single(app.world())
            .expect("the Evil Wizard should remain spawned");
        slammed |= boss.jump.is_some_and(|jump| jump.slam);
        if app.world().resource::<RunSession>().health < 100.0 {
            break;
        }
    }
    assert!(slammed, "the jump slam should launch the boss into the air");
    assert_eq!(
        app.world().resource::<RunSession>().health,
        100.0 - 18.0,
        "the first hit should be the 18-damage slam landing"
    );
}
