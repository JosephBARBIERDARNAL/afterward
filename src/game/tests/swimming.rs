use super::super::*;
use crate::game::water::WaterSurface;
use std::{path::Path, time::Duration};

use bevy::{prelude::MinimalPlugins, state::app::StatesPlugin, time::TimeUpdateStrategy};

/// The flood: falling into it from a height breaks the surface with a dive,
/// inside it the stroke replaces gravity — up while Jump is held, a slow sink
/// without — and the bank is reached by hopping out or swimming out the side.
#[test]
fn falling_into_the_flood_dives_then_swims_and_leaves_it_at_the_surface() {
    let asset_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/assets");
    let mut data = GameData::load(&asset_root).expect("Chapter 1 data should load");
    // A floor with a pool standing on it, and the player dropped in from well
    // above its surface.
    *data.rooms.get_mut(&1).expect("room 1 should exist") = yaml_serde::from_str(
        "
room_id: 1
chapter_id: 1
size: { width_h: 24, height_h: 16 }
background_profile: clouds_bright_blue
player_spawn: { x_h: 10, y_h: 11 }
platforms:
  - { material: city_pavement, rect_h: { x: 0, y: 0, width: 24, height: 1 } }
water:
  - { label: the flooded street, rect_h: { x: 6, y: 1, width: 8, height: 5 } }
enemy_spawns: []
bonus: { x_h: 20, y_h: 1 }
exit: { x_h: 22, y_h: 1 }
next_room_id: 2
",
    )
    .expect("the test layout should parse");
    let sink_speed_h_s = data.gameplay.player.water.sink_speed_h_s;
    let dive_s = data.player_animations.clips.dive.duration_s();
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
        .insert_resource(RandomState(0x7377_696d_6d69_6e67))
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
                // A pool's quad is shader work the headless app has no use
                // for; the collider is what swimming reads, so stand one up.
                for pool in &room.water {
                    commands.spawn(WaterSurface {
                        left: pool.rect_h.x * H_WORLD_UNITS,
                        right: pool.rect_h.right() * H_WORLD_UNITS,
                        bottom: pool.rect_h.y * H_WORLD_UNITS,
                        top: pool.rect_h.top() * H_WORLD_UNITS,
                    });
                }
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
    let swimming = |app: &App| {
        app.world()
            .get::<PlayerBody>(player)
            .expect("the player should remain spawned")
            .swimming
    };
    let height_h = |app: &App| {
        app.world()
            .get::<Transform>(player)
            .expect("the player should remain spawned")
            .translation
            .y
            / H_WORLD_UNITS
    };
    let sheet = |app: &App| {
        app.world()
            .get::<AnimatedSprite>(player)
            .expect("the player keeps its animation")
            .sheet
            .clone()
    };

    assert!(!swimming(&app), "the drop begins in open air");

    // Fall until the surface breaks.
    let mut falling_frames = 0;
    while !swimming(&app) && falling_frames < 60 {
        step(&mut app, &[]);
        falling_frames += 1;
    }
    assert!(swimming(&app), "the player should land in the pool");
    assert_eq!(
        sheet(&app),
        "player_dive",
        "coming down fast enough breaks the surface with a dive"
    );

    // The dive plays out, then the stroke takes over and carries the body down
    // slowly rather than letting it fall.
    for _ in 0..(dive_s / 0.05).ceil() as usize + 2 {
        step(&mut app, &[]);
    }
    assert_eq!(sheet(&app), "player_swim");
    let before_sinking_h = height_h(&app);
    for _ in 0..10 {
        step(&mut app, &[]);
    }
    let sank_h = before_sinking_h - height_h(&app);
    assert!(
        sank_h <= sink_speed_h_s * 0.5 + 0.1,
        "the sink should cover about {} H in ten frames, covered {sank_h} H",
        sink_speed_h_s * 0.5
    );
    assert!(
        !app.world()
            .get::<PlayerBody>(player)
            .expect("the player should remain spawned")
            .grounded,
        "a swimmer resting on the pool's floor is still swimming, not standing"
    );

    // Hold Jump: the stroke carries the body up to the surface and no higher.
    for _ in 0..60 {
        step(&mut app, &[KeyCode::Space]);
    }
    let surface_h = height_h(&app);
    assert!(
        (4.5..6.0).contains(&surface_h),
        "the stroke should hold the body at the surface, got {surface_h} H"
    );
    assert!(swimming(&app), "holding at the surface is still swimming");

    // Let go, then ask again at the surface: that is the hop out of the water.
    step(&mut app, &[]);
    step(&mut app, &[KeyCode::Space]);
    assert!(
        app.world()
            .get::<PlayerBody>(player)
            .expect("the player should remain spawned")
            .velocity
            .y
            > 0.0,
        "Jump at the surface throws the player out of the water"
    );

    // The side is a way out too: swim right until the pool is behind you.
    for _ in 0..60 {
        step(&mut app, &[KeyCode::KeyD]);
    }
    assert!(
        !swimming(&app),
        "the bank is reached by swimming out of the pool's side"
    );
}
