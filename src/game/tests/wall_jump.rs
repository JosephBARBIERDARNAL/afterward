use super::super::*;
use std::{path::Path, time::Duration};

use bevy::{prelude::MinimalPlugins, state::app::StatesPlugin, time::TimeUpdateStrategy};

/// A platform's side is something to cling to: a player pressed into one in
/// mid-air slides down it instead of falling, and Jump throws them up and away
/// from it, turning them to face the way they are thrown.
#[test]
fn a_player_pressed_into_a_wall_slides_down_it_and_jumps_away_from_it() {
    let asset_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/assets");
    let mut data = GameData::load(&asset_root).expect("Chapter 1 data should load");
    // A floor with a tall slab standing on it right of the spawn, so the
    // player can walk into its side and jump off it.
    *data.rooms.get_mut(&1).expect("room 1 should exist") = yaml_serde::from_str(
        "
room_id: 1
chapter_id: 1
size: { width_h: 24, height_h: 13.5 }
background_profiles: [city_teal_evening]
player_spawn: { x_h: 2.5, y_h: 1 }
platforms:
  - { material: city_pavement, rect_h: { x: 0, y: 0, width: 24, height: 1 } }
  - { material: city_plating, rect_h: { x: 4, y: 1, width: 2, height: 9 } }
enemy_spawns: []
bonus: { x_h: 20, y_h: 1 }
exit: { x_h: 22, y_h: 1 }
next_room_id: 2
",
    )
    .expect("the test layout should parse");
    let slide_speed_h_s = data.gameplay.player.wall.slide_speed_h_s;
    let jump_impulse_h_s = data.gameplay.player.wall.jump_impulse_h_s;
    let jump_push_h_s = data.gameplay.player.wall.jump_push_h_s;
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
        .insert_resource(RunSession {
            room_id: 1,
            ..default()
        })
        .insert_resource(RandomState(0x7761_6c6c_6a75_6d70))
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
            },
        )
        .add_systems(Update, player::update_player);
    app.update();
    let player = app
        .world_mut()
        .query_filtered::<Entity, With<PlayerBody>>()
        .single(app.world())
        .expect("one player should spawn");

    // One frame of input, with exactly `keys` held: anything held last frame
    // and not named now is released, and anything new counts as just pressed,
    // the way a real keyboard reports a hold rather than a run of taps.
    let step = |app: &mut App, keys: &[KeyCode]| {
        {
            let mut input = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
            input.clear();
            for key in input.get_pressed().copied().collect::<Vec<_>>() {
                if !keys.contains(&key) {
                    input.release(key);
                }
            }
            for key in keys {
                if !input.pressed(*key) {
                    input.press(*key);
                }
            }
        }
        app.update();
    };
    let height_h = |app: &App| {
        app.world()
            .get::<Transform>(player)
            .expect("the player should remain spawned")
            .translation
            .y
            / H_WORLD_UNITS
    };

    // Walk into the slab's side: the player stops against it on the ground,
    // where there is nothing to cling to yet.
    for _ in 0..24 {
        step(&mut app, &[KeyCode::KeyD]);
    }
    {
        let body = app
            .world()
            .get::<PlayerBody>(player)
            .expect("the player should remain spawned");
        assert!(body.grounded, "the walk ends on the floor");
        assert!(
            body.wall_cling.is_none(),
            "a wall is only clung to in mid-air"
        );
    }

    // Jump with the slab still held into: once the rise is spent, the player
    // catches the wall and slides down it instead of falling freely.
    step(&mut app, &[KeyCode::KeyD, KeyCode::Space]);
    let mut clung_after_frames = None;
    for frame in 0..40 {
        step(&mut app, &[KeyCode::KeyD, KeyCode::Space]);
        if app
            .world()
            .get::<PlayerBody>(player)
            .expect("the player should remain spawned")
            .wall_cling
            .is_some()
        {
            clung_after_frames = Some(frame);
            break;
        }
    }
    assert!(
        clung_after_frames.is_some(),
        "the rise should end against the slab, with the wall caught"
    );
    {
        let body = app
            .world()
            .get::<PlayerBody>(player)
            .expect("the player should remain spawned");
        assert_eq!(
            body.wall_cling,
            Some(1.0),
            "pressing into the slab in mid-air clings to it"
        );
        assert!(
            body.velocity.y / H_WORLD_UNITS >= -slide_speed_h_s - 0.01,
            "a clung wall caps the fall at the slide speed, got {} H/s",
            body.velocity.y / H_WORLD_UNITS
        );
    }
    assert_eq!(
        app.world()
            .get::<AnimatedSprite>(player)
            .expect("the player keeps its animation")
            .sheet,
        "player_wall_slide"
    );

    // The slide is slow: ten frames of it cover no more ground than the slide
    // speed allows, where a free fall would by now be far faster than that.
    let before_sliding_h = height_h(&app);
    for _ in 0..10 {
        step(&mut app, &[KeyCode::KeyD]);
    }
    let slid_h = before_sliding_h - height_h(&app);
    assert!(
        slid_h > 0.0 && slid_h <= slide_speed_h_s * 0.5 + 0.1,
        "ten frames of sliding should cover about {} H, covered {slid_h} H",
        slide_speed_h_s * 0.5
    );

    // Push off: up, away, turned around, and the wall let go of.
    step(&mut app, &[KeyCode::KeyD, KeyCode::Space]);
    {
        let body = app
            .world()
            .get::<PlayerBody>(player)
            .expect("the player should remain spawned");
        assert!(
            body.velocity.y / H_WORLD_UNITS > jump_impulse_h_s - 2.0,
            "the push throws the player upward, got {} H/s",
            body.velocity.y / H_WORLD_UNITS
        );
        assert!(
            body.velocity.x / H_WORLD_UNITS < -jump_push_h_s + 1.0,
            "and away from the wall, got {} H/s",
            body.velocity.x / H_WORLD_UNITS
        );
        assert_eq!(body.facing, -1.0, "the player faces the way they are sent");
        assert!(
            body.wall_cling.is_none(),
            "the wall is let go of as it is pushed off"
        );
    }

    // The lock keeps the push intact: still holding into the wall must not
    // steer the player straight back onto it.
    for _ in 0..2 {
        step(&mut app, &[KeyCode::KeyD]);
    }
    assert!(
        app.world()
            .get::<PlayerBody>(player)
            .expect("the player should remain spawned")
            .velocity
            .x
            < 0.0,
        "the shove carries the player clear before steering comes back"
    );
}
