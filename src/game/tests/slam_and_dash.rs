use super::super::*;
use crate::game::combat::CombatProjectile;
use std::{path::Path, time::Duration};

use bevy::{prelude::MinimalPlugins, state::app::StatesPlugin, time::TimeUpdateStrategy};

/// Attack with Down held in mid-air drives the player at the floor and breaks
/// it to both sides, and Dash carries them through anything that would hit,
/// on the ground and once in mid-air, ending early against a wall.
#[test]
fn the_slam_breaks_the_floor_and_the_dash_carries_the_player_through_a_shot() {
    let asset_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/assets");
    let mut data = GameData::load(&asset_root).expect("Chapter 1 data should load");
    // A floor of its own, and one body of polymer standing just within the
    // slam's reach, so the test owes nothing to room 1's layout.
    *data.rooms.get_mut(&1).expect("room 1 should exist") = yaml_serde::from_str(
        "
room_id: 1
chapter_id: 1
size: { width_h: 24, height_h: 13.5 }
background_profiles: [city_teal_evening]
player_spawn: { x_h: 2.5, y_h: 1 }
platforms:
  - { material: city_pavement, rect_h: { x: 0, y: 0, width: 24, height: 1 } }
enemy_spawns:
  - { type_id: blade_unit, position_h: { x: 4, y: 1 } }
bonus: { x_h: 20, y_h: 1 }
exit: { x_h: 22, y_h: 1 }
",
    )
    .expect("the test layout should parse");
    let dash = (
        data.gameplay.player.dash.speed_h_s,
        data.gameplay.player.dash.invulnerable_from_s,
    );
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
        .insert_resource(RandomState(0x736c_616d_726f_6c6c))
        .init_resource::<SoundQueue>()
        .add_message::<BonusTell>()
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
            (
                player::update_player,
                combat::player_melee,
                combat::update_projectiles,
            )
                .chain(),
        );
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
    let sheet = |app: &App| {
        app.world()
            .get::<AnimatedSprite>(player)
            .expect("the player keeps its animation")
            .sheet
            .clone()
    };
    let enemy_health = |app: &mut App| {
        let world = app.world_mut();
        world
            .query::<&EnemyBody>()
            .iter(world)
            .map(|enemy| enemy.health)
            .next()
            .expect("the body should remain spawned")
    };

    let full_health = enemy_health(&mut app);

    // Jump, then Attack with Down held: the player stops dead and is driven
    // at the floor.
    step(&mut app, &[KeyCode::Space]);
    for _ in 0..6 {
        step(&mut app, &[KeyCode::Space]);
    }
    assert!(
        !app.world()
            .get::<PlayerBody>(player)
            .expect("the player should remain spawned")
            .grounded,
        "the jump should have left the floor"
    );
    step(&mut app, &[KeyCode::KeyS, KeyCode::KeyJ]);
    {
        let body = app
            .world()
            .get::<PlayerBody>(player)
            .expect("the player should remain spawned");
        assert!(body.slamming, "Attack with Down held in mid-air slams");
        assert!(
            body.velocity.y < 0.0,
            "the slam drives the player downward, got {} H/s",
            body.velocity.y / H_WORLD_UNITS
        );
    }
    step(&mut app, &[KeyCode::KeyS]);
    assert_eq!(sheet(&app), "player_ground_slam");

    // It reaches the floor and breaks it, which costs the body beside the
    // player health even though the player never faced it down.
    let mut landed = false;
    for _ in 0..40 {
        step(&mut app, &[]);
        if app
            .world()
            .get::<PlayerBody>(player)
            .expect("the player should remain spawned")
            .grounded
        {
            landed = true;
            break;
        }
    }
    assert!(landed, "the slam should reach the floor");
    step(&mut app, &[]);
    assert!(
        enemy_health(&mut app) < full_health,
        "the broken floor hurts what is standing on it"
    );

    // Dash on the ground keeps its own speed and direction.
    for _ in 0..20 {
        step(&mut app, &[]);
    }
    step(&mut app, &[KeyCode::KeyL]);
    {
        let body = app
            .world()
            .get::<PlayerBody>(player)
            .expect("the player should remain spawned");
        assert!(body.dash_s.is_some(), "Dash dashes");
        assert!(
            body.grounded,
            "the dash runs along the floor rather than leaving it"
        );
        assert!(
            (body.velocity.x / H_WORLD_UNITS - dash.0).abs() < 0.01,
            "the dash carries the player at its own speed, got {} H/s",
            body.velocity.x / H_WORLD_UNITS
        );
    }
    step(&mut app, &[]);
    assert_eq!(sheet(&app), "player_dash");

    // Nothing can reach the player through the middle of the dash.
    for _ in 0..(dash.1 / 0.05).ceil() as usize {
        step(&mut app, &[]);
    }
    let health_before = app.world().resource::<RunSession>().health;
    let position = app
        .world()
        .get::<Transform>(player)
        .expect("the player should remain spawned")
        .translation;
    app.world_mut()
        .get_mut::<PlayerBody>(player)
        .expect("the player should remain spawned")
        .invulnerability_timer = 0.0;
    app.world_mut().spawn((
        Transform::from_translation(position + Vec3::X * 10.0),
        CombatProjectile {
            velocity_x: -H_WORLD_UNITS,
            velocity_y: 0.0,
            damage: 20.0,
            life_s: 1.0,
            remaining_range: H_WORLD_UNITS,
            half_size: Vec2::splat(4.0),
        },
    ));
    step(&mut app, &[]);
    assert_eq!(
        app.world().resource::<RunSession>().health,
        health_before,
        "a shot that lands in the middle of a dash costs nothing"
    );

    // In mid-air the dash holds the player level, and only one is allowed
    // before they stand again.
    for _ in 0..20 {
        step(&mut app, &[]);
    }
    step(&mut app, &[KeyCode::Space]);
    for _ in 0..4 {
        step(&mut app, &[KeyCode::Space]);
    }
    step(&mut app, &[KeyCode::KeyL]);
    {
        let body = app
            .world()
            .get::<PlayerBody>(player)
            .expect("the player should remain spawned");
        assert!(
            body.dash_s.is_some() && !body.grounded,
            "Dash dashes in mid-air"
        );
        assert!(
            body.velocity.y.abs() < f32::EPSILON,
            "an air dash holds the player level"
        );
        assert_eq!(
            body.air_dashes_used, 1,
            "it spends the one air dash the player has before landing"
        );
    }

    // A dash run into a wall ends there, and the player is free at once.
    for _ in 0..30 {
        step(&mut app, &[]);
    }
    {
        let mut transform = app
            .world_mut()
            .get_mut::<Transform>(player)
            .expect("the player should remain spawned");
        transform.translation.x = HALF_BODY_WIDTH + 2.0;
    }
    app.world_mut()
        .get_mut::<PlayerBody>(player)
        .expect("the player should remain spawned")
        .facing = -1.0;
    step(&mut app, &[KeyCode::KeyL]);
    step(&mut app, &[]);
    let body = app
        .world()
        .get::<PlayerBody>(player)
        .expect("the player should remain spawned");
    assert!(
        body.dash_s.is_none() && body.action.is_none(),
        "a dash into the room's edge ends as it meets it"
    );
}
