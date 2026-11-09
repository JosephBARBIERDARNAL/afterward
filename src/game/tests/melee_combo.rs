use super::super::*;
use std::{path::Path, time::Duration};

use bevy::{prelude::MinimalPlugins, state::app::StatesPlugin, time::TimeUpdateStrategy};

/// The chain: Attack throws its blows in order, one blow hurts everything it
/// reaches, letting the window lapse drops it back to the first step, and
/// owning Fire Kick lengthens it to four.
#[test]
fn the_chain_steps_through_its_blows_catches_two_bodies_and_drops_back_when_the_window_lapses() {
    let asset_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/assets");
    let mut data = GameData::load(&asset_root).expect("Chapter 1 data should load");
    // A floor of its own under the spawn, and two bodies of polymer (no
    // shield to soak the first blow) standing together inside arm's reach, so
    // the test owes nothing to room 1's layout.
    *data.rooms.get_mut(&1).expect("room 1 should exist") = yaml_serde::from_str(
        "
room_id: 1
chapter_id: 1
size: { width_h: 48, height_h: 13.5 }
background_profile: clouds_bright_blue
player_spawn: { x_h: 2.5, y_h: 1 }
platforms:
  - { material: city_pavement, rect_h: { x: 0, y: 0, width: 48, height: 1 } }
enemy_spawns:
  - { type_id: stalker_unit, position_h: { x: 3.05, y: 1 } }
  - { type_id: stalker_unit, position_h: { x: 3.5, y: 1 } }
bonus: { x_h: 43, y_h: 1 }
exit: { x_h: 45, y_h: 1 }
next_room_id: 2
",
    )
    .expect("the test layout should parse");
    let combo_window_s = data.gameplay.player.melee.combo_window_s;
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
        .insert_resource(RandomState(0x636f_6d62_6f21_2121))
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
            (player::update_player, combat::player_melee).chain(),
        );
    app.update();
    let player = app
        .world_mut()
        .query_filtered::<Entity, With<PlayerBody>>()
        .single(app.world())
        .expect("one player should spawn");

    // One frame, with Attack pressed or not. Nothing clears the input between
    // frames here, so every press is released again before the next one.
    let frame = |app: &mut App, attack: bool| {
        {
            let mut input = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
            input.release_all();
            input.clear();
            if attack {
                input.press(KeyCode::KeyJ);
            }
        }
        app.update();
    };
    // Attack runs after movement, so the blow it starts shows from the next
    // frame; the clip then plays itself out before the chain takes another.
    let blow = |app: &mut App, idle_frames: usize| {
        frame(app, true);
        frame(app, false);
        let sheet = app
            .world()
            .get::<AnimatedSprite>(player)
            .expect("the player keeps its animation")
            .sheet
            .clone();
        for _ in 0..idle_frames {
            frame(app, false);
        }
        sheet
    };
    let health = |app: &mut App| {
        let world = app.world_mut();
        let mut healths = world
            .query::<&EnemyBody>()
            .iter(world)
            .map(|enemy| enemy.health)
            .collect::<Vec<_>>();
        healths.sort_by(f32::total_cmp);
        healths
    };

    let full_health = health(&mut app);
    assert_eq!(
        full_health.len(),
        2,
        "both bodies should be standing at the start"
    );

    // The chain's first blow, which both bodies are inside.
    assert_eq!(blow(&mut app, 6), "player_strike_1");
    let after_one_blow = health(&mut app);
    assert!(
        after_one_blow
            .iter()
            .zip(&full_health)
            .all(|(after, before)| after < before),
        "one blow hurts every body its hitbox reaches, got {after_one_blow:?} \
         from {full_health:?}"
    );

    // Asked for soon enough, the chain goes on to its second and third blows
    // and then wraps back to the first.
    assert_eq!(blow(&mut app, 7), "player_strike_2");
    assert_eq!(blow(&mut app, 6), "player_kick");
    assert_eq!(blow(&mut app, 6), "player_strike_1");

    // Let the window lapse: the chain forgets where it was.
    for _ in 0..(combo_window_s / 0.05).ceil() as usize + 2 {
        frame(&mut app, false);
    }
    assert_eq!(
        blow(&mut app, 6),
        "player_strike_1",
        "a chain left to lapse starts over at its first blow"
    );

    // Fire Kick adds a fourth blow, after the kick that used to end it.
    for _ in 0..(combo_window_s / 0.05).ceil() as usize + 2 {
        frame(&mut app, false);
    }
    app.world_mut()
        .resource_mut::<RunSession>()
        .owned_bonuses
        .push(String::from("fire_kick"));
    assert_eq!(blow(&mut app, 6), "player_strike_1");
    assert_eq!(blow(&mut app, 7), "player_strike_2");
    assert_eq!(blow(&mut app, 6), "player_kick");
    assert_eq!(
        blow(&mut app, 9),
        "player_fire_kick",
        "with Fire Kick owned the chain runs to a fourth blow"
    );

    // Cover plants the player: the chain cannot be thrown from behind a guard.
    {
        let mut input = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        input.release_all();
        input.clear();
        input.press(KeyCode::KeyL);
        input.press(KeyCode::KeyJ);
    }
    app.update();
    app.update();
    assert_eq!(
        app.world()
            .get::<AnimatedSprite>(player)
            .expect("the player keeps its animation")
            .sheet,
        "player_cover",
        "Attack is ignored while Cover is held"
    );
    assert!(
        app.world()
            .get::<PlayerBody>(player)
            .expect("the player should remain spawned")
            .strike
            .is_none(),
        "no blow is thrown from behind a guard"
    );
}
