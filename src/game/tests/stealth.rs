//! Hiding from the enemies (`combat/stealth.rs`): what sets it off, and what
//! an enemy hunting a hidden player does.

use super::super::*;
use std::{path::Path, time::Duration};

use bevy::{prelude::MinimalPlugins, state::app::StatesPlugin, time::TimeUpdateStrategy};

#[test]
fn an_enemy_hunting_a_hidden_player_walks_to_where_it_saw_them_and_stands_down() {
    let asset_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/assets");
    let mut data = GameData::load(&asset_root).expect("Chapter 1 data should load");
    // A flat floor, a blade unit and the player 6 H apart.
    *data.rooms.get_mut(&3).expect("room 3 should exist") = yaml_serde::from_str(
        "
room_id: 3
chapter_id: 1
size: { width_h: 48, height_h: 27 }
background_profiles: [city_teal_evening]
player_spawn: { x_h: 14, y_h: 1 }
platforms:
  - { material: city_pavement, rect_h: { x: 0, y: 0, width: 48, height: 1 } }
enemy_spawns:
  - { type_id: blade_unit, position_h: { x: 20, y: 1 } }
bonus: { x_h: 43, y_h: 1 }
exit: { x_h: 45, y_h: 1 }
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
        .insert_state(GameScreen::Playing)
        .insert_resource(data)
        .insert_resource(SpriteSheets(sheets))
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
            50,
        )))
        .insert_resource(RunSession {
            room_id: 3,
            ..default()
        })
        .init_resource::<SoundQueue>()
        .add_message::<BonusTell>()
        .add_systems(
            Startup,
            |mut commands: Commands, data: Res<GameData>, sheets: Res<SpriteSheets>| {
                let room = &data.rooms[&3];
                world::spawn_room_platforms(&mut commands, room, &data);
                world::spawn_player(&mut commands, room, &data, &sheets);
                world::spawn_enemies(&mut commands, room, &data, &sheets, None);
            },
        )
        .add_systems(
            Update,
            (
                combat::update_enemy_ai,
                combat::update_enemy_platform_physics,
            )
                .chain(),
        );
    // It sees the player and comes for them.
    for _ in 0..4 {
        app.update();
    }
    let world = app.world_mut();
    let enemy = world
        .query::<&EnemyBody>()
        .single(world)
        .expect("one enemy");
    assert!(enemy.engaged, "the enemy has seen the player");

    // The player vanishes and slips 20 H away behind it.
    let world = app.world_mut();
    let (mut transform, mut player) = world
        .query::<(&mut Transform, &mut PlayerBody)>()
        .single_mut(world)
        .expect("one player");
    player.invisible_s = 30.0;
    let seen_at_h = transform.translation.x / H_WORLD_UNITS;
    transform.translation.x = 40.0 * H_WORLD_UNITS;
    for _ in 0..80 {
        app.update();
    }
    let world = app.world_mut();
    let (enemy, transform) = world
        .query::<(&EnemyBody, &Transform)>()
        .single(world)
        .expect("one enemy");
    let stands_h = transform.translation.x / H_WORLD_UNITS;
    assert!(!enemy.engaged, "it stands down, having found nobody");
    assert!(
        (stands_h - seen_at_h).abs() < 2.0,
        "it went to where it last saw the player ({seen_at_h} H), and stands at {stands_h} H"
    );
    assert_eq!(
        app.world().resource::<RunSession>().health,
        100.0,
        "it struck at nothing"
    );
}

#[test]
fn smoke_step_hides_a_dash_then_waits_out_its_cooldown() {
    let asset_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/assets");
    let data = GameData::load(&asset_root).expect("Chapter 1 data should load");
    let smoke_step = data.bonuses.bonus("smoke_step").expect("in the pool");
    let vanish_s = smoke_step
        .effect(BonusRarity::Rare, "vanish_s")
        .expect("a rare Smoke Step");
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, StatesPlugin))
        .insert_resource(data)
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
            50,
        )))
        .insert_resource(RunSession {
            owned_bonuses: vec![BonusPick::new("smoke_step", BonusRarity::Rare)],
            ..default()
        })
        .add_message::<BonusTell>()
        .add_systems(Update, combat::vanish);
    app.world_mut()
        .spawn((PlayerBody::default(), Transform::default()));
    app.update();
    let dash = |app: &mut App, dashing: bool| {
        let world = app.world_mut();
        world
            .query::<&mut PlayerBody>()
            .single_mut(world)
            .expect("one player")
            .dash_s = dashing.then_some(0.0);
        app.update();
        let world = app.world_mut();
        world
            .query::<&PlayerBody>()
            .single(world)
            .expect("one player")
            .invisible_s
    };
    let hidden_s = dash(&mut app, true);
    assert!(
        hidden_s > vanish_s - 0.1 && hidden_s <= vanish_s,
        "a dash hides the player for {vanish_s} s, got {hidden_s} s"
    );
    dash(&mut app, false);
    for _ in 0..40 {
        app.update();
    }
    assert_eq!(dash(&mut app, false), 0.0, "the hiding runs out");
    assert_eq!(
        dash(&mut app, true),
        0.0,
        "a second dash within the cooldown hides nothing"
    );
}
