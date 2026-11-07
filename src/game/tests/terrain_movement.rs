use super::super::*;
use crate::data::SpawnPosition;
use std::{path::Path, time::Duration};

use bevy::{prelude::MinimalPlugins, state::app::StatesPlugin, time::TimeUpdateStrategy};

#[test]
fn solid_terrain_stops_the_player_and_low_ledges_are_mantled() {
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
                world::spawn_player(
                    &mut commands,
                    &data.rooms[&1],
                    &data,
                    &sheets,
                    data.gameplay.player.gun.magazine_size,
                );
                // Player spawn is (2.5, 1): a floor, a ceiling over the spawn,
                // and a 2 H tall wall to the right.
                for (left, bottom, right, top) in [
                    (0.0, 0.0, 32.0, 1.0),
                    (1.0, 3.5, 5.0, 4.0),
                    (6.0, 1.0, 7.0, 3.0),
                ] {
                    commands.spawn(PlatformSurface {
                        left: left * H_WORLD_UNITS,
                        right: right * H_WORLD_UNITS,
                        bottom: bottom * H_WORLD_UNITS,
                        top: top * H_WORLD_UNITS,
                        footstep: FootstepSound::Stone,
                        one_way: false,
                    });
                }
            },
        )
        .add_systems(Update, player::update_player);
    app.update();

    // Jump straight up into the ceiling: the head stops at its underside.
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::Space);
    let mut peak_head_h: f32 = 0.0;
    for _ in 0..90 {
        app.update();
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .clear();
        let mut players = app.world_mut().query::<&Transform>();
        let transform = players.single(app.world()).expect("one player");
        peak_head_h = peak_head_h.max(transform.translation.y / H_WORLD_UNITS + 0.5);
    }
    assert!(
        (peak_head_h - 3.5).abs() < 0.05,
        "the ceiling caps the jump, head peaked at {peak_head_h} H"
    );

    // Walk into the wall: it stops the player at its side.
    {
        let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        keys.release(KeyCode::Space);
        keys.press(KeyCode::KeyD);
    }
    for _ in 0..90 {
        app.update();
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .clear();
    }
    let mut players = app.world_mut().query::<(&Transform, &PlayerBody)>();
    let (transform, player) = players.single(app.world()).expect("one player");
    assert!(player.grounded);
    assert!(
        (transform.translation.x / H_WORLD_UNITS - (6.0 - PLAYER_COLLISION_WIDTH_H / 2.0)).abs()
            < 0.01,
        "the wall stops the player, who reached x = {} H",
        transform.translation.x / H_WORLD_UNITS
    );

    // Jump while pressing toward the wall: the player mantles onto its top.
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::Space);
    let mut mantled = false;
    for _ in 0..90 {
        app.update();
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .clear();
        let mut players = app.world_mut().query::<&PlayerBody>();
        let climbing = players
            .single(app.world())
            .expect("one player")
            .mantle
            .is_some();
        if mantled && !climbing {
            break;
        }
        mantled |= climbing;
    }
    let mut players = app.world_mut().query::<(&Transform, &PlayerBody)>();
    let (transform, player) = players.single(app.world()).expect("one player");
    assert!(mantled, "reaching the ledge should start a mantle");
    assert!(player.grounded);
    assert!(
        transform.translation.x / H_WORLD_UNITS > 6.0
            && (transform.translation.y / H_WORLD_UNITS - 3.5).abs() < 0.01,
        "the player should stand on the wall top, got {:?}",
        transform.translation / H_WORLD_UNITS
    );
}

#[test]
fn one_way_platforms_are_jumped_through_and_dropped_through() {
    let asset_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/assets");
    let mut data = GameData::load(&asset_root).expect("Chapter 1 data should load");
    // The terrain below is spawned by hand around this spawn point, so the
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
                world::spawn_player(
                    &mut commands,
                    &data.rooms[&1],
                    &data,
                    &sheets,
                    data.gameplay.player.gun.magazine_size,
                );
                // A floor, and a plank 2 H above the player spawn at (2.5, 1).
                for (left, bottom, right, top, one_way) in
                    [(0.0, 0.0, 32.0, 1.0, false), (1.0, 2.75, 6.0, 3.0, true)]
                {
                    commands.spawn(PlatformSurface {
                        left: left * H_WORLD_UNITS,
                        right: right * H_WORLD_UNITS,
                        bottom: bottom * H_WORLD_UNITS,
                        top: top * H_WORLD_UNITS,
                        footstep: FootstepSound::Wood,
                        one_way,
                    });
                }
            },
        )
        .add_systems(Update, player::update_player);
    app.update();

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::Space);
    for _ in 0..90 {
        app.update();
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .clear();
    }
    let mut players = app.world_mut().query::<(&Transform, &PlayerBody)>();
    let (transform, player) = players.single(app.world()).expect("one player");
    assert!(
        player.grounded
            && player.on_one_way
            && (transform.translation.y / H_WORLD_UNITS - 3.5).abs() < 0.01,
        "jumping from below should pass through the plank and land on it"
    );

    // Down + Jump drops through the plank back to the floor.
    {
        let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        keys.release(KeyCode::Space);
        keys.press(KeyCode::KeyS);
    }
    app.update();
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::Space);
    for _ in 0..90 {
        app.update();
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .clear();
    }
    let mut players = app.world_mut().query::<(&Transform, &PlayerBody)>();
    let (transform, player) = players.single(app.world()).expect("one player");
    assert!(
        player.grounded
            && !player.on_one_way
            && (transform.translation.y / H_WORLD_UNITS - 1.5).abs() < 0.01,
        "Down + Jump should drop through the plank onto the floor"
    );
}

#[test]
fn jumping_off_a_wall_kicks_the_player_away_from_it() {
    let asset_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/assets");
    let mut data = GameData::load(&asset_root).expect("Chapter 1 data should load");
    // The terrain below is spawned by hand around this spawn point, so the
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
                world::spawn_player(
                    &mut commands,
                    &data.rooms[&1],
                    &data,
                    &sheets,
                    data.gameplay.player.gun.magazine_size,
                );
                // A floor and a wall too tall to mantle, right of the spawn.
                for (left, bottom, right, top) in [(0.0, 0.0, 32.0, 1.0), (4.0, 1.0, 5.0, 13.0)] {
                    commands.spawn(PlatformSurface {
                        left: left * H_WORLD_UNITS,
                        right: right * H_WORLD_UNITS,
                        bottom: bottom * H_WORLD_UNITS,
                        top: top * H_WORLD_UNITS,
                        footstep: FootstepSound::Stone,
                        one_way: false,
                    });
                }
            },
        )
        .add_systems(Update, player::update_player);
    app.update();

    // Jump against the wall and slide down it, holding toward it.
    {
        let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        keys.press(KeyCode::KeyD);
        keys.press(KeyCode::Space);
    }
    let mut sliding = false;
    for _ in 0..120 {
        app.update();
        let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        keys.clear();
        keys.release(KeyCode::Space);
        let mut players = app.world_mut().query::<&PlayerBody>();
        let player = players.single(app.world()).expect("one player");
        if player.wall_side == Some(1.0) && player.velocity.y < 0.0 {
            sliding = true;
            break;
        }
    }
    assert!(sliding, "the player should slide down the wall");
    // The wall art has the player's back to the wall: on it, it faces away.
    for _ in 0..15 {
        app.update();
    }
    let mut players = app.world_mut().query::<(&PlayerBody, &Sprite)>();
    let (player, sprite) = players.single(app.world()).expect("one player");
    assert!(
        player.wall_side == Some(1.0) && player.facing < 0.0 && sprite.flip_x,
        "a player on a wall to its right faces left"
    );

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::Space);
    app.update();
    let mut players = app.world_mut().query::<&PlayerBody>();
    let player = players.single(app.world()).expect("one player");
    assert!(
        player.velocity.x < 0.0 && player.velocity.y > 0.0 && player.facing < 0.0,
        "a wall jump kicks up and away from the wall, got {:?}",
        player.velocity
    );
    // Still holding toward the wall, the player keeps facing away from it
    // through the whole wall-jump animation.
    let mut wall_jump_frames = 0;
    for _ in 0..30 {
        {
            let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
            keys.clear();
            keys.release(KeyCode::Space);
        }
        app.update();
        let mut players = app.world_mut().query::<&PlayerBody>();
        let player = players.single(app.world()).expect("one player");
        if player.action != Some(PlayerAction::WallJump) {
            break;
        }
        wall_jump_frames += 1;
        assert!(player.facing < 0.0, "the wall jump turned back to the wall");
    }
    assert!(wall_jump_frames > 0, "the wall jump plays its animation");
}
