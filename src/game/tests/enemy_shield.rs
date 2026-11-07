use super::super::*;
use crate::data::EnemyMaterial;
use std::{path::Path, time::Duration};

use bevy::{prelude::MinimalPlugins, state::app::StatesPlugin, time::TimeUpdateStrategy};

#[test]
fn a_metal_enemys_shield_soaks_one_shot_and_shatters_before_its_health_drops() {
    let asset_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/assets");
    let mut data = GameData::load(&asset_root).expect("Chapter 1 data should load");
    // A fixed layout, independent of the authored rooms: a flat floor with the
    // player on the left, a metal swordsman in front of it, and a Gorgon of
    // flesh further off to stand for the unshielded case.
    *data.rooms.get_mut(&1).expect("room 1 should exist") = yaml_serde::from_str(
        "
room_id: 1
chapter_id: 1
size: { width_viewports: 2, height_h: 13.5 }
background_profile: clouds_bright_blue
player_spawn: { x_h: 2.5, y_h: 1 }
terrain:
  - { material: stone_cobble, rect_h: { x: 0, y: 0, width: 48, height: 1 } }
enemy_spawns:
  - { type_id: swordsman, position_h: { x: 10, y: 1 } }
  - { type_id: gorgon_1, position_h: { x: 20, y: 1 } }
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
        .init_resource::<RunSession>()
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
                player::update_player,
                combat::player_gun,
                combat::update_player_bullets,
                combat::update_enemy_shields,
            )
                .chain(),
        );
    app.update();

    let enemy_of = |app: &mut App, material: EnemyMaterial| {
        app.world_mut()
            .query::<(Entity, &EnemyBody)>()
            .iter(app.world())
            .find(|(_, enemy)| enemy.material == material)
            .map(|(entity, _)| entity)
            .expect("the layout should spawn one enemy of each material")
    };
    let metal = enemy_of(&mut app, EnemyMaterial::Metal);
    let flesh = enemy_of(&mut app, EnemyMaterial::Flesh);
    let body = |app: &App, entity: Entity| {
        let enemy = app
            .world()
            .get::<EnemyBody>(entity)
            .expect("the enemy should remain spawned");
        (enemy.shielded, enemy.health)
    };
    let shield_icons = |app: &mut App| {
        app.world_mut()
            .query::<&EnemyShieldIcon>()
            .iter(app.world())
            .count()
    };

    let (shielded, full_health) = body(&app, metal);
    assert!(shielded, "a metal enemy enters the room shielded");
    assert!(!body(&app, flesh).0, "an enemy of flesh has no shield");
    assert_eq!(
        shield_icons(&mut app),
        1,
        "only the metal enemy carries a shield icon"
    );

    // Hold the trigger: the first round that lands breaks the shield without
    // touching the enemy's health, and only later rounds wound it.
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::KeyJ);
    let mut health_when_shield_broke = None;
    for _ in 0..40 {
        app.update();
        let (shielded, health) = body(&app, metal);
        if !shielded && health_when_shield_broke.is_none() {
            health_when_shield_broke = Some(health);
        }
        if health < full_health {
            break;
        }
    }

    assert_eq!(
        health_when_shield_broke,
        Some(full_health),
        "the shot that breaks the shield costs the enemy no health"
    );
    assert!(
        body(&app, metal).1 < full_health,
        "once the shield is gone the enemy takes damage normally"
    );

    // The break plays out quickly, then the icon goes.
    for _ in 0..20 {
        app.update();
        if shield_icons(&mut app) == 0 {
            break;
        }
    }
    assert_eq!(
        shield_icons(&mut app),
        0,
        "the icon despawns once the shield has shattered"
    );
}
