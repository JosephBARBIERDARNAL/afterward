use super::super::*;
use crate::game::combat::PlayerBullet;
use std::{path::Path, time::Duration};

use bevy::{prelude::MinimalPlugins, state::app::StatesPlugin, time::TimeUpdateStrategy};

#[test]
fn the_gun_fires_alternating_shots_aims_and_reloads_its_magazine() {
    let asset_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/assets");
    let mut data = GameData::load(&asset_root).expect("Chapter 1 data should load");
    data.rooms
        .get_mut(&1)
        .expect("room 1 should be authored")
        .enemy_spawns
        .truncate(1);
    let gun_damage = data.gameplay.player.gun.damage;
    let magazine = data.gameplay.player.gun.magazine_size;
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
        .init_resource::<RunSession>()
        .insert_resource(RandomState(0x636f_6d62_6f73_7764))
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
                assets::animate_sprites,
            )
                .chain(),
        );
    app.update();

    let player = app
        .world_mut()
        .query_filtered::<Entity, With<PlayerBody>>()
        .single(app.world())
        .expect("room 1 should spawn one player");
    let enemy = app
        .world_mut()
        .query_filtered::<Entity, With<EnemyBody>>()
        .single(app.world())
        .expect("the focused room should spawn one enemy");
    let player_position = app
        .world()
        .get::<Transform>(player)
        .expect("the player should remain spawned")
        .translation;
    // A sturdy target a few H in front of the player, with no AI running.
    let enemy_feet_offset = app
        .world()
        .get::<EnemyBody>(enemy)
        .expect("the enemy should remain spawned")
        .feet_offset;
    app.world_mut()
        .get_mut::<Transform>(enemy)
        .expect("the enemy should remain spawned")
        .translation = Vec3::new(
        player_position.x + H_WORLD_UNITS * 4.0,
        player_position.y - H_WORLD_UNITS / 2.0 + enemy_feet_offset,
        player_position.z,
    );
    app.world_mut()
        .get_mut::<EnemyBody>(enemy)
        .expect("the enemy should remain spawned")
        .health = 10_000.0;
    let enemy_health = |app: &App| {
        app.world()
            .get::<EnemyBody>(enemy)
            .expect("the enemy should remain spawned")
            .health
    };
    let body = |app: &App| {
        let body = app
            .world()
            .get::<PlayerBody>(player)
            .expect("the player should remain spawned");
        (body.ammo, body.reload.is_some())
    };

    // Holding the trigger empties the magazine; every bullet hits for the
    // gun's damage, and the shots alternate between the two straight clips.
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::KeyJ);
    let mut sheets_seen = BTreeSet::new();
    for _ in 0..80 {
        app.update();
        sheets_seen.insert(
            app.world()
                .get::<AnimatedSprite>(player)
                .expect("the player animation should remain available")
                .sheet
                .clone(),
        );
        if body(&app) == (0, true) {
            break;
        }
    }
    assert_eq!(
        body(&app),
        (0, true),
        "an empty magazine reloads on its own"
    );
    {
        let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        keys.release(KeyCode::KeyJ);
        keys.clear();
    }
    for _ in 0..30 {
        app.update();
    }
    assert_eq!(
        body(&app),
        (magazine, false),
        "the reload refills the magazine"
    );
    let dealt = 10_000.0 - enemy_health(&app);
    assert!(
        (dealt - gun_damage * magazine as f32).abs() < 0.01,
        "every round should hit for the gun's damage, dealt {dealt}"
    );
    assert!(
        sheets_seen.contains("player_shoot_straight_1")
            && sheets_seen.contains("player_shoot_straight_2"),
        "consecutive shots alternate their clips: {sheets_seen:?}"
    );

    // Holding Up fires diagonally upward with the matching clip.
    {
        let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        keys.press(KeyCode::KeyW);
        keys.press(KeyCode::KeyJ);
    }
    app.update();
    let bullet_velocity = app
        .world_mut()
        .query::<&PlayerBullet>()
        .iter(app.world())
        .map(|bullet| bullet.velocity)
        .next()
        .expect("the shot should spawn a bullet");
    assert!(bullet_velocity.x > 0.0 && bullet_velocity.y > 0.0);
    {
        let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        keys.release_all();
        keys.clear();
    }
    app.update();
    assert!(
        app.world()
            .get::<AnimatedSprite>(player)
            .expect("the player animation should remain available")
            .sheet
            .starts_with("player_shoot_up_")
    );

    // Reload refills a partial magazine early.
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::KeyR);
    app.update();
    assert_eq!(body(&app), (magazine - 1, true));
}
