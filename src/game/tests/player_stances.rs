use super::super::*;
use std::{path::Path, time::Duration};

use bevy::{prelude::MinimalPlugins, state::app::StatesPlugin, time::TimeUpdateStrategy};

#[test]
fn crouching_slows_cover_blocks_frontal_hits_and_death_plays_out_before_the_run_ends() {
    let asset_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/assets");
    let data = GameData::load(&asset_root).expect("Chapter 1 data should load");
    let crouch_speed_h_s = data.gameplay.player.crouch_speed_h_s;
    let cover_multiplier = data.gameplay.player.cover_damage_taken_multiplier;
    let death_s =
        data.player_animations.clips.death.duration_s() + data.gameplay.player.death_hold_s;
    let sheets = data
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
        .collect();

    let mut app = App::new();
    app.add_plugins((MinimalPlugins, StatesPlugin))
        .insert_resource(data)
        .insert_resource(SpriteSheets(sheets))
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
            50,
        )))
        .init_state::<GameScreen>()
        .init_resource::<RunSession>()
        .insert_resource(RandomState(0x7374_616e_6365_7321))
        .init_resource::<SoundQueue>()
        .init_resource::<ControlBindings>()
        .init_resource::<ButtonInput<KeyCode>>()
        .add_systems(Startup, controls::initialize_default_control_bindings)
        .add_systems(
            Startup,
            |mut commands: Commands, data: Res<GameData>, sheets: Res<SpriteSheets>| {
                let room = &data.rooms[&1];
                world::spawn_player(
                    &mut commands,
                    room,
                    &data,
                    &sheets,
                    data.gameplay.player.gun.magazine_size,
                );
                let width = room.width_h(&data.gameplay) * H_WORLD_UNITS;
                commands.spawn(PlatformSurface {
                    left: 0.0,
                    right: width,
                    bottom: -H_WORLD_UNITS,
                    top: room.player_spawn.y_h * H_WORLD_UNITS,
                    material: PlatformMaterial::Soil,
                    one_way: false,
                });
            },
        )
        .add_systems(
            Update,
            (
                player::update_player,
                combat::player_gun,
                combat::update_projectiles,
                combat::update_player_death,
                assets::animate_sprites,
            )
                .chain(),
        );
    app.update();
    let player = app
        .world_mut()
        .query_filtered::<Entity, With<PlayerBody>>()
        .single(app.world())
        .expect("one player should spawn");
    let hold = |app: &mut App, keys: &[KeyCode], frames: usize| {
        for _ in 0..frames {
            {
                let mut input = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
                input.release_all();
                input.clear();
                for key in keys {
                    input.press(*key);
                }
            }
            app.update();
        }
    };
    let sheet = |app: &App| {
        app.world()
            .get::<AnimatedSprite>(player)
            .expect("the player should keep its animation")
            .sheet
            .clone()
    };
    // An enemy shot flying at the player from the right, where it faces.
    let shoot_at_player = |app: &mut App, damage: f32| {
        let position = app
            .world()
            .get::<Transform>(player)
            .expect("the player should remain spawned")
            .translation;
        app.world_mut()
            .get_mut::<PlayerBody>(player)
            .expect("the player should remain spawned")
            .invulnerability_timer = 0.0;
        app.world_mut().spawn((
            Transform::from_translation(position + Vec3::X * 10.0),
            CombatProjectile {
                velocity_x: -H_WORLD_UNITS,
                damage,
                life_s: 1.0,
                remaining_range: H_WORLD_UNITS,
                half_size: Vec2::splat(4.0),
            },
        ));
        app.update();
    };

    // Holding Down crouches; moving while crouched is the slower crouch walk.
    hold(&mut app, &[KeyCode::KeyS], 2);
    assert_eq!(sheet(&app), "player_crouch");
    hold(&mut app, &[KeyCode::KeyS, KeyCode::KeyD], 10);
    let body = app
        .world()
        .get::<PlayerBody>(player)
        .expect("the player should remain spawned");
    assert!(body.crouching);
    assert!((body.velocity.x - crouch_speed_h_s * H_WORLD_UNITS).abs() < 0.5);
    assert_eq!(sheet(&app), "player_crouch_walk");

    // Holding Cover stops the player in place and halves hits from the front.
    hold(&mut app, &[KeyCode::KeyL], 10);
    let covered_x = app
        .world()
        .get::<Transform>(player)
        .expect("the player should remain spawned")
        .translation
        .x;
    hold(&mut app, &[KeyCode::KeyL, KeyCode::KeyD], 4);
    assert_eq!(
        app.world()
            .get::<Transform>(player)
            .expect("the player should remain spawned")
            .translation
            .x,
        covered_x,
        "the player cannot move while in cover"
    );
    assert_eq!(sheet(&app), "player_cover");
    let health_before = app.world().resource::<RunSession>().health;
    shoot_at_player(&mut app, 10.0);
    let taken = health_before - app.world().resource::<RunSession>().health;
    assert!((taken - 10.0 * cover_multiplier).abs() < 0.01);

    // A lethal hit plays the death animation; the run ends once it has held.
    hold(&mut app, &[], 1);
    app.world_mut().resource_mut::<RunSession>().health = 5.0;
    shoot_at_player(&mut app, 10.0);
    app.update();
    assert_eq!(sheet(&app), "player_death");
    assert_eq!(
        *app.world().resource::<State<GameScreen>>().get(),
        GameScreen::MainMenu,
        "the run only ends after the death animation"
    );
    for _ in 0..(death_s / 0.05).ceil() as usize + 2 {
        app.update();
    }
    assert_eq!(
        *app.world().resource::<State<GameScreen>>().get(),
        GameScreen::Dead
    );
}
