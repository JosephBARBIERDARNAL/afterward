use super::super::*;
use std::{path::Path, time::Duration};

use bevy::{prelude::MinimalPlugins, state::app::StatesPlugin, time::TimeUpdateStrategy};

#[test]
fn default_controls_move_jump_and_dash_the_player() {
    let asset_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/assets");
    let data = GameData::load(&asset_root).expect("Chapter 1 data should load");
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
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
            50,
        )))
        .init_resource::<RunSession>()
        .insert_resource(RandomState(0x616e_696d_6174_696f))
        .init_resource::<SoundQueue>()
        .init_resource::<ControlBindings>()
        .insert_resource(PreferencesDir(None))
        .init_resource::<ButtonInput<KeyCode>>()
        .add_systems(Startup, controls::initialize_control_bindings)
        .add_systems(
            Startup,
            |mut commands: Commands, data: Res<GameData>, sheets: Res<SpriteSheets>| {
                let room = &data.rooms[&1];
                world::spawn_player(
                    &mut commands,
                    room,
                    &data,
                    &sheets,
                    data.gameplay.player.gun.magazine_size,
                );
                let width = room.width_h(&data.gameplay) * H_WORLD_UNITS;
                commands.spawn(PlatformSurface {
                    left: 0.0,
                    right: width,
                    bottom: -H_WORLD_UNITS,
                    top: room.player_spawn.y_h * H_WORLD_UNITS,
                    material: PlatformMaterial::Soil,
                    one_way: false,
                });
            },
        )
        .add_systems(
            Update,
            (
                player::update_player,
                combat::player_gun,
                combat::update_player_bullets,
                assets::animate_sprites,
            )
                .chain(),
        );

    app.update();
    let player = {
        let mut players = app.world_mut().query_filtered::<Entity, With<PlayerBody>>();
        players
            .single(app.world())
            .expect("room should spawn one player")
    };
    let start_position = app
        .world()
        .get::<Transform>(player)
        .expect("player should remain spawned")
        .translation;

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::KeyD);
    app.update();
    assert_eq!(
        app.world()
            .get::<AnimatedSprite>(player)
            .expect("player animation should remain available")
            .sheet,
        "player_walk",
        "the player walks while accelerating from a standstill"
    );
    for _ in 0..8 {
        app.update();
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .clear();
    }
    let after_moving = app
        .world()
        .get::<Transform>(player)
        .expect("player should remain spawned")
        .translation;
    assert!(after_moving.x > start_position.x);
    assert_eq!(
        app.world()
            .get::<AnimatedSprite>(player)
            .expect("player animation should remain available")
            .sheet,
        "player_jog"
    );

    {
        let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        keys.clear();
        keys.press(KeyCode::KeyJ);
    }
    // The gun fires after movement, so its clip shows from the next frame.
    app.update();
    app.update();
    assert_eq!(
        app.world()
            .get::<AnimatedSprite>(player)
            .expect("player animation should remain available")
            .sheet,
        "player_run_shooting",
        "firing straight while jogging plays the run-shooting cycle"
    );

    {
        let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        keys.release(KeyCode::KeyD);
        keys.release(KeyCode::KeyJ);
        keys.clear();
    }
    for _ in 0..30 {
        app.update();
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .clear();
    }
    assert_eq!(
        app.world()
            .get::<AnimatedSprite>(player)
            .expect("player animation should remain available")
            .sheet,
        "player_idle",
        "with no enemy around, the player relaxes into the calm idle"
    );

    {
        let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        keys.release(KeyCode::KeyD);
        keys.clear();
        keys.press(KeyCode::Space);
    }
    app.update();
    let after_jumping = app
        .world()
        .get::<Transform>(player)
        .expect("player should remain spawned")
        .translation;
    assert!(after_jumping.y > start_position.y);
    assert_eq!(
        app.world()
            .get::<AnimatedSprite>(player)
            .expect("player animation should remain available")
            .sheet,
        "player_jump"
    );

    let before_dashing_x = after_jumping.x;
    {
        let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        keys.release(KeyCode::Space);
        keys.clear();
        keys.press(KeyCode::ShiftLeft);
    }
    app.update();
    let after_dashing_x = app
        .world()
        .get::<Transform>(player)
        .expect("player should remain spawned")
        .translation
        .x;
    assert!(after_dashing_x > before_dashing_x + 50.0);
    assert!(
        app.world()
            .get::<PlayerBody>(player)
            .expect("player should remain spawned")
            .dash_timer
            > 0.0
    );

    {
        let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        keys.release(KeyCode::ShiftLeft);
        keys.clear();
    }
    for _ in 0..3 {
        app.update();
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .clear();
    }
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::KeyA);
    for _ in 0..5 {
        app.update();
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .clear();
    }
    let airborne_player = app
        .world()
        .get::<PlayerBody>(player)
        .expect("player should remain spawned");
    assert!(!airborne_player.grounded);
    let air_max_speed_h_s = app
        .world()
        .resource::<GameData>()
        .gameplay
        .player
        .air_max_speed_h_s;
    assert!(
        (airborne_player.velocity.x.abs() / H_WORLD_UNITS - air_max_speed_h_s).abs() < 0.01,
        "airborne movement should reach the configured maximum horizontal speed"
    );

    {
        let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        keys.release(KeyCode::ShiftLeft);
        keys.clear();
        keys.press(KeyCode::KeyQ);
    }
    for _ in 0..32 {
        app.update();
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .clear();
    }
    let after_moving_left = app
        .world()
        .get::<Transform>(player)
        .expect("player should remain spawned")
        .translation;
    assert!(
        after_moving_left.x < after_dashing_x,
        "the configured AZERTY left input should move the player"
    );
}
