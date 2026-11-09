use super::super::*;
use std::{path::Path, time::Duration};

use bevy::{prelude::MinimalPlugins, state::app::StatesPlugin, time::TimeUpdateStrategy};

#[test]
fn an_enemy_only_chases_a_player_it_can_see() {
    let asset_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/assets");
    let mut data = GameData::load(&asset_root).expect("Chapter 1 data should load");
    // A fixed layout, independent of the authored rooms: a flat street with a
    // pillar to hide behind, one BladeUnit watching the way the player walks
    // in from, and a second one at the far end with its back to it.
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
  - { type_id: blade_unit, position_h: { x: 30, y: 1 } }
  - { type_id: blade_unit, position_h: { x: 40, y: 1 }, facing: right }
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
                world::spawn_player(&mut commands, room, &data, &sheets);
                world::spawn_enemies(&mut commands, room, &data, &sheets);
            },
        )
        .add_systems(Update, combat::update_enemy_ai);

    app.update();
    let spawned = {
        let mut enemies = app
            .world_mut()
            .query_filtered::<(Entity, &Transform), With<EnemyBody>>();
        let mut spawned = enemies
            .iter(app.world())
            .map(|(enemy, transform)| (enemy, transform.translation.x))
            .collect::<Vec<_>>();
        spawned.sort_by(|(_, left), (_, right)| left.total_cmp(right));
        spawned
    };
    let [(blade_unit, _), (lookout, _)] = spawned[..] else {
        panic!("the layout should spawn two Swordsmen");
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
    };
    let chasing = |app: &App, enemy: Entity| {
        app.world()
            .get::<EnemyBody>(enemy)
            .expect("the BladeUnit should remain spawned")
            .engaged
    };

    // Detection range is 10 H: the player walks in far beyond it.
    stands_at(&mut app, 2.5, 1.0);
    assert!(
        !chasing(&app, blade_unit),
        "a player across the room should go unnoticed"
    );
    // Close enough, but high above the BladeUnit's eyes.
    stands_at(&mut app, 26.0, 6.0);
    assert!(
        !chasing(&app, blade_unit),
        "a player overhead should go unnoticed"
    );
    // At its level and in front of it, but with the pillar in between.
    stands_at(&mut app, 22.0, 1.0);
    assert!(
        !chasing(&app, blade_unit),
        "a player behind the pillar should go unnoticed"
    );
    // Close and in the open, but behind the BladeUnit's back, and in front of
    // the lookout but out of its range.
    stands_at(&mut app, 34.0, 1.0);
    assert!(
        !chasing(&app, blade_unit),
        "a player behind the BladeUnit should go unnoticed"
    );
    assert!(
        !chasing(&app, lookout),
        "a player behind the lookout, which the room turns the other way, should go unnoticed"
    );
    // In the open, in front of it, within range: seen at last.
    stands_at(&mut app, 26.0, 1.0);
    assert!(
        chasing(&app, blade_unit),
        "a player in plain sight should be chased"
    );
    // Once seen, the chase holds even when the player breaks sight again.
    stands_at(&mut app, 2.5, 1.0);
    assert!(
        chasing(&app, blade_unit),
        "a seen player stays the target while retain_target is set"
    );
    // The lookout watches the way the room points it, toward the exit.
    stands_at(&mut app, 44.0, 1.0);
    assert!(
        chasing(&app, lookout),
        "a player stepping in front of an enemy facing right should be chased"
    );
}
