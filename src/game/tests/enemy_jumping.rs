use super::super::*;
use std::{path::Path, time::Duration};

use bevy::{prelude::MinimalPlugins, state::app::StatesPlugin, time::TimeUpdateStrategy};

#[test]
fn ground_enemy_climbs_the_staircase_to_a_player_on_a_higher_platform() {
    let asset_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/assets");
    let data = GameData::load(&asset_root).expect("Chapter 1 data should load");
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
            room_id: 3,
            ..default()
        })
        .init_resource::<SoundQueue>()
        .add_systems(
            Startup,
            |mut commands: Commands, data: Res<GameData>, sheets: Res<SpriteSheets>| {
                let room = &data.rooms[&3];
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
                combat::update_enemy_ai,
                combat::update_enemy_platform_physics,
            )
                .chain(),
        );
    app.update();

    // Room 3 has a high walkway reached through plank steps. Keep only the
    // ground Gorgon at x = 8 and stand the player on the walkway, at x = 18.
    let mut enemies = app.world_mut().query::<(Entity, &EnemyBody, &Transform)>();
    let (climber, others) = enemies.iter(app.world()).fold(
        (None, Vec::new()),
        |(climber, mut others), (entity, enemy, transform)| {
            if enemy.type_id == "gorgon_1" && transform.translation.x == 8.0 * H_WORLD_UNITS {
                (Some(entity), others)
            } else {
                others.push(entity);
                (climber, others)
            }
        },
    );
    let climber = climber.expect("room 3 should spawn a ground Gorgon at x = 8 H");
    for entity in others {
        app.world_mut().despawn(entity);
    }
    let player_feet_y = 7.0 * H_WORLD_UNITS;
    let mut players = app
        .world_mut()
        .query_filtered::<&mut Transform, With<PlayerBody>>();
    players
        .single_mut(app.world_mut())
        .expect("room should spawn one player")
        .translation = Vec3::new(
        18.0 * H_WORLD_UNITS,
        player_feet_y + 0.5 * H_WORLD_UNITS,
        4.0,
    );

    let mut jumped = false;
    let mut reached_player_level = false;
    for _ in 0..200 {
        app.update();
        let enemy = app
            .world()
            .get::<EnemyBody>(climber)
            .expect("the Gorgon should remain spawned");
        jumped |= enemy.jump.is_some();
        let feet_y = enemy.home_y - enemy.feet_offset;
        if enemy.grounded && (feet_y - player_feet_y).abs() <= 1.0 {
            reached_player_level = true;
            break;
        }
    }
    assert!(jumped, "the Gorgon should jump instead of waiting below");
    assert!(
        reached_player_level,
        "the Gorgon should climb onto the player's platform"
    );
}

#[test]
fn evil_wizard_jump_slam_damages_the_player_on_landing() {
    let asset_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/assets");
    let data = GameData::load(&asset_root).expect("Chapter 1 data should load");
    let slam_index = data.enemies.types["evil_wizard"]
        .attacks
        .iter()
        .position(|attack| attack.kind == "jump_slam")
        .expect("the Evil Wizard should have a jump slam");
    let placeholder_sheet = SpriteSheet {
        image: Handle::default(),
        layout: Handle::default(),
        frame_size: Vec2::new(250.0, 250.0),
        frames: 1,
        fps: 1.0,
    };
    let sheets = SpriteSheets(BTreeMap::from([
        (String::from("player_idle"), placeholder_sheet.clone()),
        (String::from("boss_idle"), placeholder_sheet),
    ]));

    let mut app = App::new();
    app.add_plugins((MinimalPlugins, StatesPlugin))
        .insert_resource(data)
        .insert_resource(sheets)
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
            20,
        )))
        .init_state::<GameScreen>()
        .insert_resource(RunSession {
            room_id: 10,
            ..default()
        })
        .init_resource::<SoundQueue>()
        .add_systems(
            Startup,
            |mut commands: Commands, data: Res<GameData>, sheets: Res<SpriteSheets>| {
                let room = &data.rooms[&10];
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
                combat::update_enemy_ai,
                combat::update_enemy_platform_physics,
            )
                .chain(),
        );
    app.update();

    let mut bosses = app.world_mut().query::<&mut EnemyBody>();
    let mut boss = bosses
        .single_mut(app.world_mut())
        .expect("room 10 should spawn only the Evil Wizard");
    boss.next_attack = slam_index;
    boss.engaged = true;

    let mut slammed = false;
    for _ in 0..500 {
        app.update();
        let mut bosses = app.world_mut().query::<&EnemyBody>();
        let boss = bosses
            .single(app.world())
            .expect("the Evil Wizard should remain spawned");
        slammed |= boss.jump.is_some_and(|jump| jump.slam);
        if app.world().resource::<RunSession>().health < 100.0 {
            break;
        }
    }
    assert!(slammed, "the jump slam should launch the boss into the air");
    assert_eq!(
        app.world().resource::<RunSession>().health,
        100.0 - 18.0,
        "the first hit should be the 18-damage slam landing"
    );
}
