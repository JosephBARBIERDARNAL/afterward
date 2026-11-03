use super::super::*;
use std::{path::Path, time::Duration};

use bevy::{prelude::MinimalPlugins, time::TimeUpdateStrategy};

#[test]
fn chained_sword_swings_deal_more_damage_until_the_combo_resets() {
    let asset_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/assets");
    let mut data = GameData::load(&asset_root).expect("Chapter 1 data should load");
    data.rooms
        .get_mut(&1)
        .expect("room 1 should be authored")
        .enemy_spawns
        .truncate(1);
    let sword_damage = data.gameplay.player.sword.damage;
    let multipliers = data.gameplay.player.sword.combo.damage_multipliers.clone();
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
    app.add_plugins(MinimalPlugins)
        .insert_resource(data)
        .insert_resource(SpriteSheets(sheets))
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
            50,
        )))
        .init_resource::<RunSession>()
        .insert_resource(RandomState(0x636f_6d62_6f73_7764))
        .init_resource::<SoundQueue>()
        .init_resource::<ControlBindings>()
        .init_resource::<ButtonInput<KeyCode>>()
        .add_systems(Startup, controls::initialize_default_control_bindings)
        .add_systems(
            Startup,
            |mut commands: Commands, data: Res<GameData>, sheets: Res<SpriteSheets>| {
                let room = &data.rooms[&1];
                world::spawn_room_platforms(&mut commands, room);
                world::spawn_player(&mut commands, room, &data, &sheets);
                world::spawn_enemies(&mut commands, room, &data, &sheets);
            },
        )
        .add_systems(
            Update,
            (
                player::update_player,
                combat::player_attacks,
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
    // A sturdy target placed just in front of the player, with no AI running.
    app.world_mut()
        .get_mut::<Transform>(enemy)
        .expect("the enemy should remain spawned")
        .translation = player_position + Vec3::new(H_WORLD_UNITS * 1.5, 0.0, 0.0);
    app.world_mut()
        .get_mut::<EnemyBody>(enemy)
        .expect("the enemy should remain spawned")
        .health = 10_000.0;

    // Waits `idle_frames`, presses J once, then advances until that swing
    // lands; returns its damage and the player animation it plays.
    let swing_after = |app: &mut App, idle_frames: usize| {
        for _ in 0..idle_frames {
            app.update();
        }
        let health_before = app
            .world()
            .get::<EnemyBody>(enemy)
            .expect("the enemy should remain spawned")
            .health;
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::KeyJ);
        for _ in 0..20 {
            app.update();
            {
                let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
                keys.release(KeyCode::KeyJ);
                keys.clear();
            }
            let health = app
                .world()
                .get::<EnemyBody>(enemy)
                .expect("the enemy should remain spawned")
                .health;
            if health < health_before {
                let sheet = app
                    .world()
                    .get::<AnimatedSprite>(player)
                    .expect("the player animation should remain available")
                    .sheet
                    .clone();
                return (health_before - health, sheet);
            }
        }
        panic!("the buffered sword press never landed a swing");
    };

    // Pressing again right away is buffered and continues the chain; each hit
    // plays its own combo animation and hits harder than the last.
    let hits = (0..multipliers.len())
        .map(|_| swing_after(&mut app, 0))
        .collect::<Vec<_>>();
    for (step, ((damage, sheet), multiplier)) in hits.iter().zip(&multipliers).enumerate() {
        assert!((damage - sword_damage * multiplier).abs() < 0.01);
        assert_eq!(sheet, &format!("player_sword_combo_{}", step + 1));
    }
    assert!(
        hits.windows(2).all(|pair| pair[1].0 > pair[0].0),
        "each further combo hit should hit harder: {hits:?}"
    );

    // The finisher ends the chain: the next swing starts over.
    let (restart_damage, restart_sheet) = swing_after(&mut app, 12);
    assert!((restart_damage - sword_damage * multipliers[0]).abs() < 0.01);
    assert_eq!(restart_sheet, "player_sword_combo_1");

    // Stopping the attacks (no press queued once the sword is ready) also drops
    // the combo back to its first hit.
    let (second_hit, _) = swing_after(&mut app, 0);
    assert!((second_hit - sword_damage * multipliers[1]).abs() < 0.01);
    swing_after(&mut app, 30);
    let (restarted_damage, restarted_sheet) = swing_after(&mut app, 30);
    assert!((restarted_damage - sword_damage * multipliers[0]).abs() < 0.01);
    assert_eq!(restarted_sheet, "player_sword_combo_1");
}
