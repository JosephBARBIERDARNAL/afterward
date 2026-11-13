use super::super::*;
use std::{path::Path, time::Duration};

use bevy::{prelude::MinimalPlugins, state::app::StatesPlugin, time::TimeUpdateStrategy};

#[test]
fn each_layer_of_armor_soaks_one_blow_and_keeps_the_enemy_from_being_interrupted() {
    let asset_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/assets");
    let mut data = GameData::load(&asset_root).expect("Chapter 1 data should load");
    // A fixed layout, independent of the authored rooms: a flat floor with the
    // player on the left, a blade unit within arm's reach of it that the room
    // gives two layers of armor, and a second one further off that it gives
    // none.
    *data.rooms.get_mut(&1).expect("room 1 should exist") = yaml_serde::from_str(
        "
room_id: 1
chapter_id: 1
size: { width_h: 48, height_h: 13.5 }
background_profiles: [clouds_bright_blue]
player_spawn: { x_h: 2.5, y_h: 1 }
platforms:
  - { material: city_pavement, rect_h: { x: 0, y: 0, width: 48, height: 1 } }
enemy_spawns:
  - { type_id: blade_unit, position_h: { x: 3.5, y: 1 }, armor_layers: 2 }
  - { type_id: blade_unit, position_h: { x: 20, y: 1 } }
bonus: { x_h: 43, y_h: 1 }
exit: { x_h: 45, y_h: 1 }
next_room_id: 2
",
    )
    .expect("the test layout should parse");
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
    // Six frames at 30 fps, as `art/ui/enemy_shield.typ` renders the shield.
    let shield_sheet = SpriteSheet {
        image: Handle::default(),
        layout: Handle::default(),
        frame_size: Vec2::new(21.0, 16.0),
        frames: 6,
        fps: 30.0,
    };
    sheets.insert(String::from(combat::SHIELD_SHEET), shield_sheet);
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
        .insert_resource(RandomState(0x7368_6965_6c64_2121))
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
                combat::player_melee,
                combat::update_enemy_shields,
            )
                .chain(),
        );
    app.update();

    let enemy_that_is = |app: &mut App, armored: bool| {
        app.world_mut()
            .query::<(Entity, &EnemyBody)>()
            .iter(app.world())
            .find(|(_, enemy)| (enemy.armor > 0) == armored)
            .map(|(entity, _)| entity)
            .expect("the layout should spawn one armored enemy and one bare")
    };
    let metal = enemy_that_is(&mut app, true);
    let bare = enemy_that_is(&mut app, false);
    let body = |app: &App, entity: Entity| {
        let enemy = app
            .world()
            .get::<EnemyBody>(entity)
            .expect("the enemy should remain spawned");
        (enemy.armor, enemy.health)
    };
    let shield_icons = |app: &mut App| {
        app.world_mut()
            .query::<&EnemyShieldIcon>()
            .iter(app.world())
            .count()
    };

    let (armor, full_health) = body(&app, metal);
    assert_eq!(armor, 2, "the room's spawn sets the layers of armor");
    assert_eq!(
        body(&app, bare).0,
        0,
        "an enemy the room gives no armor has none"
    );
    assert_eq!(
        shield_icons(&mut app),
        2,
        "the armored enemy carries one icon per layer, the other none"
    );

    // Catch the enemy in the middle of a wind-up, then throw the chain: each
    // of the first two blows breaks a layer without touching its health or
    // its wind-up, and only the blow after them wounds and interrupts it.
    const WINDUP_S: f32 = 30.0;
    app.world_mut()
        .get_mut::<EnemyBody>(metal)
        .expect("the enemy should remain spawned")
        .windup_remaining = WINDUP_S;
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::KeyJ);
    let mut health_as_layers_broke = Vec::new();
    for _ in 0..80 {
        app.update();
        let (armor, health) = body(&app, metal);
        if usize::from(2 - armor) > health_as_layers_broke.len() {
            health_as_layers_broke.push(health);
            let enemy = app.world().get::<EnemyBody>(metal).expect("still spawned");
            assert_eq!(
                (enemy.windup_remaining, enemy.hurt_timer),
                (WINDUP_S, 0.0),
                "a blow soaked by armor does not interrupt the enemy"
            );
        }
        if health < full_health {
            break;
        }
    }

    assert_eq!(
        health_as_layers_broke,
        vec![full_health; 2],
        "a blow that breaks a layer costs the enemy no health"
    );
    let enemy = app.world().get::<EnemyBody>(metal).expect("still spawned");
    assert!(
        enemy.health < full_health,
        "once the armor is gone the enemy takes damage normally"
    );
    assert!(
        enemy.windup_remaining == 0.0 && enemy.hurt_timer > 0.0,
        "a blow on an enemy without armor interrupts it"
    );

    // The break plays out quickly, then the last icon goes.
    for _ in 0..20 {
        app.update();
        if shield_icons(&mut app) == 0 {
            break;
        }
    }
    assert_eq!(
        shield_icons(&mut app),
        0,
        "the icons despawn once the armor has shattered"
    );
}
