use super::super::*;
use crate::game::combat::{CombatProjectile, Decoy};
use std::{path::Path, time::Duration};

use bevy::{prelude::MinimalPlugins, state::app::StatesPlugin, time::TimeUpdateStrategy};

/// The Decoy bonus: a roll leaves a standing double of the hero behind, an
/// engaged enemy goes after whichever of the two stands nearer to it, and a
/// blow that lands on the double costs the run nothing.
#[test]
fn a_roll_leaves_a_double_behind_that_is_gone_for_rather_than_the_player() {
    let asset_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/assets");
    let mut data = GameData::load(&asset_root).expect("Chapter 1 data should load");
    // A floor of its own, with the player standing in the middle of it and one
    // body well off to the left, so the test owes nothing to room 1's layout.
    *data.rooms.get_mut(&1).expect("room 1 should exist") = yaml_serde::from_str(
        "
room_id: 1
chapter_id: 1
size: { width_h: 24, height_h: 13.5 }
background_profile: clouds_bright_blue
player_spawn: { x_h: 12, y_h: 1 }
platforms:
  - { material: city_pavement, rect_h: { x: 0, y: 0, width: 24, height: 1 } }
enemy_spawns:
  - { type_id: stalker_unit, position_h: { x: 10, y: 1 } }
bonus: { x_h: 20, y_h: 1 }
exit: { x_h: 22, y_h: 1 }
next_room_id: 2
",
    )
    .expect("the test layout should parse");
    let decoy_health = data.bonus_effect("decoy", "decoy_health");
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
        .insert_resource(RandomState(0x6465_636f_795f_3031))
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
                world::spawn_enemies(&mut commands, room, &data, &sheets);
            },
        )
        .add_systems(
            Update,
            (
                player::update_player,
                combat::plant_decoy,
                combat::update_enemy_ai,
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
    let doubles = |app: &mut App| {
        let world = app.world_mut();
        world
            .query::<(&Transform, &Decoy)>()
            .iter(world)
            .map(|(transform, decoy)| (transform.translation.x, decoy.health))
            .collect::<Vec<_>>()
    };

    // A run that does not own Decoy rolls and leaves nothing behind.
    step(&mut app, &[KeyCode::KeyS, KeyCode::Space]);
    step(&mut app, &[KeyCode::KeyS]);
    assert!(
        app.world()
            .get::<PlayerBody>(player)
            .expect("the player should remain spawned")
            .roll_s
            .is_some(),
        "Down with Jump rolls"
    );
    assert!(
        doubles(&mut app).is_empty(),
        "a roll leaves no double behind until the run owns Decoy"
    );

    // With the bonus taken, the next roll plants one where it set off from,
    // rather than where its first frame has already carried the player.
    for _ in 0..20 {
        step(&mut app, &[]);
    }
    app.world_mut()
        .resource_mut::<RunSession>()
        .owned_bonuses
        .push("decoy".to_string());
    let rolled_from = app
        .world()
        .get::<Transform>(player)
        .expect("the player should remain spawned")
        .translation
        .x;
    step(&mut app, &[KeyCode::KeyS, KeyCode::Space]);
    step(&mut app, &[KeyCode::KeyS]);
    let standing = doubles(&mut app);
    assert_eq!(standing.len(), 1, "the roll plants one double");
    assert!(
        (standing[0].0 - rolled_from).abs() < 0.01,
        "it stands where the roll began, at {} rather than {rolled_from}",
        standing[0].0
    );
    assert_eq!(
        standing[0].1, decoy_health,
        "with the health the bonus gives it"
    );
    assert!(
        app.world()
            .get::<Transform>(player)
            .expect("the player should remain spawned")
            .translation
            .x
            > standing[0].0,
        "and the player has rolled out of it"
    );

    // Only one stands at a time: rolling again replaces the double rather than
    // leaving a line of them across the room.
    for _ in 0..20 {
        step(&mut app, &[]);
    }
    step(&mut app, &[KeyCode::KeyS, KeyCode::Space]);
    step(&mut app, &[KeyCode::KeyS]);
    assert_eq!(
        doubles(&mut app).len(),
        1,
        "a second roll replaces the first one's double"
    );

    // Standing nearer the enemy than the player does, the double is what an
    // engaged enemy turns to go after.
    for _ in 0..20 {
        step(&mut app, &[]);
    }
    let double_x = doubles(&mut app)[0].0;
    // The player is stood well off to the left and the body brought up beside
    // the double, so the double is both the nearer of the two and on the
    // enemy's other hand: the way it turns says which one it is going after.
    app.world_mut()
        .get_mut::<Transform>(player)
        .expect("the player should remain spawned")
        .translation
        .x = double_x - 16.0 * H_WORLD_UNITS;
    {
        let world = app.world_mut();
        let mut enemies = world.query::<(&mut Transform, &mut EnemyBody)>();
        let (mut transform, mut enemy) = enemies
            .single_mut(world)
            .expect("the body should remain spawned");
        transform.translation.x = double_x - 4.0 * H_WORLD_UNITS;
        enemy.engaged = true;
    }
    step(&mut app, &[]);
    {
        let world = app.world_mut();
        let enemy = world
            .query::<&EnemyBody>()
            .single(world)
            .expect("the body should remain spawned");
        assert!(
            enemy.facing > 0.0,
            "the enemy turns toward the double it stands nearest, not the \
             player on its other side"
        );
    }

    // A shot that lands on the double costs the run nothing and the double its
    // health. It is aimed at the double, well away from where the player now
    // stands, so only the double can be what it reached.
    let health_before = app.world().resource::<RunSession>().health;
    let double_health_before = doubles(&mut app)[0].1;
    let double_position = {
        let world = app.world_mut();
        world
            .query_filtered::<&Transform, With<Decoy>>()
            .single(world)
            .expect("the double should still stand")
            .translation
    };
    app.world_mut().spawn((
        Transform::from_translation(double_position + Vec3::X * 4.0),
        CombatProjectile {
            velocity_x: -H_WORLD_UNITS,
            damage: 10.0,
            life_s: 1.0,
            remaining_range: H_WORLD_UNITS,
            half_size: Vec2::splat(4.0),
        },
    ));
    step(&mut app, &[]);
    assert_eq!(
        app.world().resource::<RunSession>().health,
        health_before,
        "a shot the double caught costs the run nothing"
    );
    assert!(
        doubles(&mut app)
            .first()
            .is_none_or(|(_, health)| *health < double_health_before),
        "and comes out of the double's own health"
    );
}
