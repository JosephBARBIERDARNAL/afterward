use super::super::*;
use crate::data::SpawnPosition;
use std::{path::Path, time::Duration};

use bevy::{prelude::MinimalPlugins, state::app::StatesPlugin, time::TimeUpdateStrategy};

#[test]
fn a_platform_stops_a_jump_from_below_and_never_lets_the_player_drop_through() {
    let asset_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/assets");
    let mut data = GameData::load(&asset_root).expect("Chapter 1 data should load");
    // The platforms below are spawned by hand around this spawn point, so the
    // test doesn't depend on room 1's layout.
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
                // A cellar floor, the floor the player stands on, and a plank
                // 1.75 H above their head at the spawn (2.5, 1).
                for (left, bottom, right, top) in [
                    (0.0, -6.0, 32.0, -5.0),
                    (0.0, 0.0, 32.0, 1.0),
                    (1.0, 2.75, 6.0, 3.0),
                ] {
                    commands.spawn(PlatformSurface {
                        left: left * H_WORLD_UNITS,
                        right: right * H_WORLD_UNITS,
                        bottom: bottom * H_WORLD_UNITS,
                        top: top * H_WORLD_UNITS,
                        footstep: FootstepSound::Wood,
                        tilt: None,
                    });
                }
            },
        )
        .add_systems(Update, player::update_player);
    app.update();

    // A jump that would clear 2.9 H bumps its head on the plank instead.
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::Space);
    let mut highest_head_h = f32::MIN;
    for _ in 0..90 {
        app.update();
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .clear();
        let mut players = app.world_mut().query::<(&Transform, &PlayerBody)>();
        let (transform, _) = players.single(app.world()).expect("one player");
        highest_head_h = highest_head_h.max(transform.translation.y / H_WORLD_UNITS + 0.5);
    }
    assert!(
        highest_head_h <= 2.76,
        "the plank should stop the jump, but the player's head reached {highest_head_h} H"
    );
    let mut players = app.world_mut().query::<(&Transform, &PlayerBody)>();
    let (transform, player) = players.single(app.world()).expect("one player");
    assert!(
        player.grounded && (transform.translation.y / H_WORLD_UNITS - 1.5).abs() < 0.01,
        "the player should fall back onto the floor they jumped from"
    );

    // Down + Jump is just a jump now: platforms are solid from every side, so
    // the cellar floor below stays out of reach.
    {
        let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        keys.press(KeyCode::KeyS);
        keys.press(KeyCode::Space);
    }
    for _ in 0..60 {
        app.update();
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .clear();
    }
    let mut players = app.world_mut().query::<(&Transform, &PlayerBody)>();
    let (transform, player) = players.single(app.world()).expect("one player");
    assert!(
        player.grounded && (transform.translation.y / H_WORLD_UNITS - 1.5).abs() < 0.01,
        "Down + Jump should leave the player standing on the floor"
    );
}

#[test]
fn a_platform_in_the_way_stops_a_body_walking_into_its_side() {
    let asset_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/assets");
    let mut data = GameData::load(&asset_root).expect("Chapter 1 data should load");
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
                // A floor, and a tall slab standing on it right of the spawn.
                for (left, bottom, right, top) in [(0.0, 0.0, 32.0, 1.0), (4.0, 1.0, 5.0, 13.0)] {
                    commands.spawn(PlatformSurface {
                        left: left * H_WORLD_UNITS,
                        right: right * H_WORLD_UNITS,
                        bottom: bottom * H_WORLD_UNITS,
                        top: top * H_WORLD_UNITS,
                        footstep: FootstepSound::Stone,
                        tilt: None,
                    });
                }
            },
        )
        .add_systems(Update, player::update_player);
    app.update();

    // Walking into the slab ends against its left side.
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::KeyD);
    for _ in 0..120 {
        app.update();
        let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        keys.clear();
        keys.press(KeyCode::KeyD);
    }
    let mut players = app.world_mut().query::<(&Transform, &PlayerBody)>();
    let (transform, player) = players.single(app.world()).expect("one player");
    let x_h = transform.translation.x / H_WORLD_UNITS;
    assert!(
        player.grounded && (3.0..=4.0).contains(&x_h),
        "the slab should stop the player against its side, who reached x = {x_h} H"
    );
}
