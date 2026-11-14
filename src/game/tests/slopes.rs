use super::super::*;
use crate::data::{PlatformConfig, RoomConfig, SpawnPosition};
use std::{path::Path, time::Duration};

use bevy::{prelude::MinimalPlugins, state::app::StatesPlugin, time::TimeUpdateStrategy};

#[test]
fn a_gentle_slope_is_walked_and_a_steep_one_is_a_wall() {
    let asset_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/assets");
    let mut data = GameData::load(&asset_root).expect("Chapter 1 data should load");
    // The platforms below are spawned by hand around this spawn point, so the
    // test doesn't depend on room 1's layout.
    data.rooms
        .get_mut(&1)
        .expect("room 1 should be authored")
        .player_spawn = SpawnPosition { x_h: 2.5, y_h: 1.0 };
    // A floor whose top is at 1 H, a plank leaning 20 degrees out of it, and
    // further on a slab leaning 60 degrees.
    let blocks: Vec<PlatformConfig> = yaml_serde::from_str(
        "
- { material: city_pavement, rect_h: { x: 0, y: 0, width: 40, height: 1 } }
- { material: city_pavement, rect_h: { x: 6, y: 1, width: 6, height: 0.5 }, rotation_deg: 20 }
- { material: city_pavement, rect_h: { x: 20, y: 1, width: 6, height: 0.5 }, rotation_deg: 60 }
",
    )
    .expect("the test platforms should parse");
    // The plank's upper face, from its low end to its high one.
    let [_, _, high, low] = blocks[1].corners_h();
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
            move |mut commands: Commands, data: Res<GameData>, sheets: Res<SpriteSheets>| {
                world::spawn_player(&mut commands, &data.rooms[&1], &data, &sheets);
                let mut room = data.rooms[&1].clone();
                room.platforms.clone_from(&blocks);
                room.props.clear();
                for surface in PlatformSurface::of_room(&room, &data) {
                    commands.spawn(surface);
                }
            },
        )
        .add_systems(Update, player::update_player);
    app.update();

    // Part of the way up and back down again: the plank falls away under a
    // player walking down it, who is held on it rather than left in the air.
    for key in [KeyCode::KeyD, KeyCode::KeyA] {
        let mut highest_feet_h = f32::MIN;
        for _ in 0..75 {
            app.update();
            let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
            keys.clear();
            keys.press(key);
            let mut players = app.world_mut().query::<(&Transform, &PlayerBody)>();
            let (transform, player) = players.single(app.world()).expect("one player");
            highest_feet_h = highest_feet_h.max(transform.translation.y / H_WORLD_UNITS - 0.5);
            assert!(player.grounded, "the player should never leave the ground");
        }
        assert!(
            highest_feet_h > 1.5,
            "the player should be up the plank, not at {highest_feet_h} H"
        );
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .release(key);
    }

    // Walking right climbs the plank without ever leaving it: the foot that
    // leads rides its face all the way up.
    let half_body_h = PLAYER_COLLISION_WIDTH_H / 2.0;
    let mut highest_feet_h = f32::MIN;
    for _ in 0..360 {
        app.update();
        let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        keys.clear();
        keys.press(KeyCode::KeyD);
        let mut players = app.world_mut().query::<(&Transform, &PlayerBody)>();
        let (transform, player) = players.single(app.world()).expect("one player");
        let x_h = transform.translation.x / H_WORLD_UNITS;
        let feet_h = transform.translation.y / H_WORLD_UNITS - 0.5;
        highest_feet_h = highest_feet_h.max(feet_h);
        let lead_h = x_h + half_body_h;
        if (8.0..high.x).contains(&lead_h) {
            let face_h = low.y + (high.y - low.y) * (lead_h - low.x) / (high.x - low.x);
            assert!(
                player.grounded && (feet_h - face_h).abs() < 0.02,
                "at x = {x_h} H the player should stand on the plank at {face_h} H, \
                 not at {feet_h} H (grounded: {})",
                player.grounded
            );
        }
    }
    assert!(
        (highest_feet_h - high.y).abs() < 0.02,
        "the climb should top out on the plank's high end at {} H, not at {highest_feet_h} H",
        high.y
    );
    // Past its high end the player dropped back to the floor, walked on, and
    // met the slab: too steep to walk, it stopped them where it leaves the
    // floor, at about x = 22.6 H.
    let mut players = app.world_mut().query::<(&Transform, &PlayerBody)>();
    let (transform, player) = players.single(app.world()).expect("one player");
    let x_h = transform.translation.x / H_WORLD_UNITS;
    let feet_h = transform.translation.y / H_WORLD_UNITS - 0.5;
    assert!(
        player.grounded && (feet_h - 1.0).abs() < 0.01 && (22.0..22.6).contains(&x_h),
        "the slab should stop the player on the floor, who reached ({x_h}, {feet_h}) H"
    );

    // Walking back, the plank's high end hangs over the floor: its underside
    // is as solid as its face, and stops the player's head under it.
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .release(KeyCode::KeyD);
    for _ in 0..360 {
        app.update();
        let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        keys.clear();
        keys.press(KeyCode::KeyA);
    }
    let mut players = app.world_mut().query::<(&Transform, &PlayerBody)>();
    let (transform, player) = players.single(app.world()).expect("one player");
    let x_h = transform.translation.x / H_WORLD_UNITS;
    let feet_h = transform.translation.y / H_WORLD_UNITS - 0.5;
    assert!(
        player.grounded && (feet_h - 1.0).abs() < 0.01 && (11.5..12.5).contains(&x_h),
        "the plank's underside should stop the player, who reached ({x_h}, {feet_h}) H"
    );
}

#[test]
fn a_tilted_platform_hides_and_stops_shots_only_where_it_stands() {
    let asset_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/assets");
    let data = GameData::load(&asset_root).expect("Chapter 1 data should load");
    // A thin plank 10 H long, turned 45 degrees about (10, 10): it runs from
    // about (6.5, 6.5) to (13.5, 13.5), and its bounds cover that whole square.
    let block: PlatformConfig = yaml_serde::from_str(
        "{ material: city_pavement, rect_h: { x: 5, y: 9.75, width: 10, height: 0.5 }, rotation_deg: 45 }",
    )
    .expect("the plank should parse");
    let mut room = data.rooms[&1].clone();
    room.platforms = vec![block];
    room.props.clear();
    let [plank] = PlatformSurface::of_room(&room, &data)[..] else {
        panic!("the plank is the room's one collider");
    };
    let at = |x_h: f32, y_h: f32| Vec2::new(x_h, y_h) * H_WORLD_UNITS;

    // Across the plank, corner to corner of its bounds.
    assert!(plank.blocks_sight(at(7.0, 13.0), at(13.0, 7.0)));
    // Inside its bounds but under it all the way.
    assert!(!plank.blocks_sight(at(9.0, 7.0), at(13.0, 11.0)));

    let shot = Vec2::splat(0.1 * H_WORLD_UNITS);
    assert!(plank.intersects_projectile(at(8.0, 12.0), at(10.0, 10.0), shot));
    assert!(!plank.intersects_projectile(at(11.0, 7.0), at(13.0, 8.0), shot));
}

#[test]
fn a_ground_enemy_walks_up_a_slope_to_a_player_standing_above_it() {
    let asset_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/assets");
    let mut data = GameData::load(&asset_root).expect("Chapter 1 data should load");
    // A fixed layout, independent of the authored rooms: a floor, a walkway
    // whose top is 5 H up, too high to leap at, and a plank leaning 20
    // degrees from the one to the other.
    let room = data.rooms.get_mut(&3).expect("room 3 should exist");
    let layout: RoomConfig = yaml_serde::from_str(
        "
room_id: 3
chapter_id: 1
size: { width_h: 48, height_h: 27 }
background_profiles: [city_teal_evening]
player_spawn: { x_h: 2.5, y_h: 1 }
platforms:
  - { material: city_pavement, rect_h: { x: 0, y: 0, width: 48, height: 1 } }
  - { material: city_pavement, rect_h: { x: 17.5, y: 4.5, width: 14, height: 0.5 } }
  - { material: city_plating, rect_h: { x: 6, y: 2.45, width: 12, height: 0.5 }, rotation_deg: 20 }
enemy_spawns:
  - { type_id: blade_unit, position_h: { x: 5, y: 1 } }
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
        .insert_resource(data)
        .insert_resource(SpriteSheets(sheets))
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_micros(
            16_667,
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
    // The blade unit sees the player beside it, who is then stood on the walkway.
    app.update();
    let mut enemies = app.world_mut().query_filtered::<Entity, With<EnemyBody>>();
    let climber = enemies
        .single(app.world())
        .expect("the layout should spawn one blade unit");
    let player_feet_y = 5.0 * H_WORLD_UNITS;
    let mut players = app
        .world_mut()
        .query_filtered::<&mut Transform, With<PlayerBody>>();
    players
        .single_mut(app.world_mut())
        .expect("room should spawn one player")
        .translation = Vec3::new(
        24.0 * H_WORLD_UNITS,
        player_feet_y + 0.5 * H_WORLD_UNITS,
        4.0,
    );

    let mut highest_feet_h = f32::MIN;
    for _ in 0..900 {
        app.update();
        let enemy = app
            .world()
            .get::<EnemyBody>(climber)
            .expect("the blade unit should remain spawned");
        let feet_y = enemy.home_y - enemy.feet_offset;
        highest_feet_h = highest_feet_h.max(feet_y / H_WORLD_UNITS);
        if enemy.grounded && (feet_y - player_feet_y).abs() <= 1.0 {
            return;
        }
    }
    panic!(
        "the blade unit should walk up the plank to the walkway, but got no higher than {highest_feet_h} H"
    );
}
