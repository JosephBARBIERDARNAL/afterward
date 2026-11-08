use super::super::*;
use std::{path::Path, time::Duration};

use bevy::{prelude::MinimalPlugins, state::app::StatesPlugin, time::TimeUpdateStrategy};

#[test]
fn swordsman_bobs_while_walking_and_returns_to_idle_pose() {
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
        .insert_resource(RunSession {
            room_id: 1,
            ..default()
        })
        .init_resource::<SoundQueue>()
        .add_systems(
            Startup,
            |mut commands: Commands,
             data: Res<GameData>,
             mut session: ResMut<RunSession>,
             sheets: Res<SpriteSheets>| {
                let room = &data.rooms[&1];
                world::spawn_player(
                    &mut commands,
                    room,
                    &data,
                    &sheets,
                    data.gameplay.player.gun.magazine_size,
                );
                world::spawn_enemies(&mut commands, room, &data, &sheets);
                session.world_loaded = true;
            },
        )
        .add_systems(Update, combat::update_enemy_ai);

    app.update();
    let (swordsman, home_y) = {
        let mut enemies = app.world_mut().query::<(Entity, &EnemyBody)>();
        let (entity, enemy) = enemies
            .iter(app.world())
            .find(|(_, enemy)| enemy.type_id == "swordsman")
            .expect("room 1 should spawn a Swordsman");
        (entity, enemy.home_y)
    };
    let mut highest_position = home_y;
    for _ in 0..6 {
        app.update();
        let position_y = app
            .world()
            .get::<Transform>(swordsman)
            .expect("Swordsman should remain spawned")
            .translation
            .y;
        highest_position = highest_position.max(position_y);
    }
    assert!(
        highest_position > home_y,
        "walking should produce a step bob"
    );

    let player = {
        let mut players = app.world_mut().query_filtered::<Entity, With<PlayerBody>>();
        players
            .single(app.world())
            .expect("room should spawn one player")
    };
    let swordsman_x = app
        .world()
        .get::<Transform>(swordsman)
        .expect("Swordsman should remain spawned")
        .translation
        .x;
    // Within the Swordsman's reach, whatever its authored hitbox is, so it
    // stops walking and settles back into its idle pose.
    let stand_off = {
        let enemy = app
            .world()
            .get::<EnemyBody>(swordsman)
            .expect("Swordsman should remain spawned");
        let half_width = app
            .world()
            .resource::<GameData>()
            .gameplay
            .player
            .hurtbox_h
            .width
            / 2.0;
        (enemy.attacks[0].reach_h + half_width) * 0.5 * H_WORLD_UNITS
    };
    let player_y = app
        .world()
        .get::<Transform>(player)
        .expect("player should remain spawned")
        .translation
        .y;
    app.world_mut()
        .get_mut::<Transform>(player)
        .expect("player should remain spawned")
        .translation = Vec3::new(swordsman_x - stand_off, player_y, 0.0);
    app.world_mut()
        .get_mut::<EnemyBody>(swordsman)
        .expect("Swordsman should remain spawned")
        .attack_cooldown = 10.0;

    app.update();
    let swordsman_transform = app
        .world()
        .get::<Transform>(swordsman)
        .expect("Swordsman should remain spawned");
    assert_eq!(swordsman_transform.translation.y, home_y);
    assert_eq!(swordsman_transform.rotation, Quat::IDENTITY);
}
