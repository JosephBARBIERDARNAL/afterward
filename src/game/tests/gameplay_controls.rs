use super::super::*;
use std::{path::Path, time::Duration};

use bevy::{prelude::MinimalPlugins, state::app::StatesPlugin, time::TimeUpdateStrategy};

#[test]
fn default_inputs_move_and_defeat_a_room_enemy() {
    let asset_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/assets");
    let mut data = GameData::load(&asset_root).expect("Chapter 1 data should load");
    // A fixed layout, independent of the authored rooms: a flat floor with
    // the player on the left and one swordsman ahead.
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
  - { type_id: swordsman, position_h: { x: 10, y: 1 } }
bonus: { x_h: 43, y_h: 1 }
exit: { x_h: 45, y_h: 1 }
next_room_id: 2
",
    )
    .expect("the test layout should parse");
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
        .insert_state(GameScreen::Playing)
        .insert_resource(RunSession {
            room_id: 1,
            ..default()
        })
        .insert_resource(RandomState(0x6174_7461_636b_6572))
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
                |mut keys: ResMut<ButtonInput<KeyCode>>,
                 players: Query<&Transform, (With<PlayerBody>, Without<EnemyBody>)>,
                 enemies: Query<(&EnemyBody, &Transform), Without<PlayerBody>>| {
                    keys.clear();
                    keys.release(KeyCode::KeyD);
                    keys.release(KeyCode::ArrowLeft);
                    // Release J every frame so each press fires a fresh shot.
                    keys.release(KeyCode::KeyJ);
                    let (Ok(player), Some((_, target))) = (
                        players.single(),
                        enemies.iter().find(|(enemy, _)| enemy.health > 0.0),
                    ) else {
                        return;
                    };
                    let horizontal_distance = target.translation.x - player.translation.x;
                    if horizontal_distance > H_WORLD_UNITS * 1.5 {
                        keys.press(KeyCode::KeyD);
                    } else if horizontal_distance < -H_WORLD_UNITS * 1.5 {
                        keys.press(KeyCode::ArrowLeft);
                    }
                    keys.press(KeyCode::KeyJ);
                },
                player::update_player,
                combat::player_gun,
                combat::update_player_bullets,
                combat::update_enemy_ai,
                combat::update_projectiles,
            )
                .chain()
                .run_if(in_state(GameScreen::Playing)),
        );

    app.update();
    let mut players = app.world_mut().query_filtered::<Entity, With<PlayerBody>>();
    let player = players
        .single(app.world())
        .expect("room 1 should spawn one player");
    let mut enemies = app.world_mut().query_filtered::<Entity, With<EnemyBody>>();
    let enemy = enemies
        .single(app.world())
        .expect("the focused room should spawn one enemy");
    let starting_x = app
        .world()
        .get::<Transform>(player)
        .expect("the player should remain spawned")
        .translation
        .x;

    for _ in 0..600 {
        if app
            .world()
            .get::<EnemyBody>(enemy)
            .is_some_and(|body| body.health <= 0.0)
        {
            break;
        }
        app.update();
    }

    assert_eq!(
        app.world()
            .get::<EnemyBody>(enemy)
            .expect("the enemy should remain available for verification")
            .health,
        0.0,
        "the player should defeat the swordsman using movement and gun inputs"
    );
    assert!(
        app.world()
            .get::<Transform>(player)
            .expect("the player should remain spawned")
            .translation
            .x
            > starting_x,
        "the configured right movement input should move the player"
    );
    assert!(
        app.world().resource::<RunSession>().health > 0.0,
        "the player should survive the focused combat encounter"
    );
}
