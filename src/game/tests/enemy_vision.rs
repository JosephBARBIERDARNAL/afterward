use super::super::*;
use std::{path::Path, time::Duration};

use bevy::{prelude::MinimalPlugins, state::app::StatesPlugin, time::TimeUpdateStrategy};

#[test]
fn an_enemy_only_chases_a_player_it_can_see() {
    let asset_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/assets");
    let mut data = GameData::load(&asset_root).expect("Chapter 1 data should load");
    // A fixed layout, independent of the authored rooms: a flat street with a
    // pillar to hide behind and one Swordsman facing the way the player walks
    // in from.
    *data.rooms.get_mut(&1).expect("room 1 should exist") = yaml_serde::from_str(
        "
room_id: 1
chapter_id: 1
size: { width_h: 48, height_h: 13.5 }
background_profile: clouds_bright_blue
player_spawn: { x_h: 2.5, y_h: 1 }
platforms:
  - { material: city_pavement, rect_h: { x: 0, y: 0, width: 48, height: 1 } }
  - { material: city_plating, rect_h: { x: 24, y: 1, width: 1, height: 4 } }
enemy_spawns:
  - { type_id: swordsman, position_h: { x: 30, y: 1 } }
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
        .insert_resource(RunSession {
            room_id: 1,
            ..default()
        })
        .init_resource::<SoundQueue>()
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
        .add_systems(Update, combat::update_enemy_ai);

    app.update();
    let swordsman = {
        let mut enemies = app.world_mut().query_filtered::<Entity, With<EnemyBody>>();
        enemies
            .single(app.world())
            .expect("the layout should spawn one Swordsman")
    };
    let player = {
        let mut players = app.world_mut().query_filtered::<Entity, With<PlayerBody>>();
        players
            .single(app.world())
            .expect("room should spawn one player")
    };
    let stands_at = |app: &mut App, x_h: f32, feet_y_h: f32| {
        app.world_mut()
            .get_mut::<Transform>(player)
            .expect("player should remain spawned")
            .translation = Vec3::new(x_h * H_WORLD_UNITS, (feet_y_h + 0.5) * H_WORLD_UNITS, 4.0);
        app.update();
        app.world()
            .get::<EnemyBody>(swordsman)
            .expect("the Swordsman should remain spawned")
            .engaged
    };

    // Detection range is 10 H: the player walks in far beyond it.
    assert!(
        !stands_at(&mut app, 2.5, 1.0),
        "a player across the room should go unnoticed"
    );
    // Close enough, but high above the Swordsman's eyes.
    assert!(
        !stands_at(&mut app, 26.0, 6.0),
        "a player overhead should go unnoticed"
    );
    // At its level and in front of it, but with the pillar in between.
    assert!(
        !stands_at(&mut app, 22.0, 1.0),
        "a player behind the pillar should go unnoticed"
    );
    // Close and in the open, but behind the Swordsman's back.
    assert!(
        !stands_at(&mut app, 34.0, 1.0),
        "a player behind the Swordsman should go unnoticed"
    );
    // In the open, in front of it, within range: seen at last.
    assert!(
        stands_at(&mut app, 26.0, 1.0),
        "a player in plain sight should be chased"
    );
    // Once seen, the chase holds even when the player breaks sight again.
    assert!(
        stands_at(&mut app, 2.5, 1.0),
        "a seen player stays the target while retain_target is set"
    );
}
