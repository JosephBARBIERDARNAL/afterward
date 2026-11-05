use super::super::*;
use std::{path::Path, time::Duration};

use bevy::{prelude::MinimalPlugins, state::app::StatesPlugin, time::TimeUpdateStrategy};

#[test]
fn behind_the_bonus_screen_the_player_lands_and_idles_without_taking_input() {
    let asset_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/assets");
    let data = GameData::load(&asset_root).expect("Chapter 1 data should load");
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
        .insert_state(GameScreen::Bonus)
        .insert_resource(data)
        .insert_resource(SpriteSheets(sheets))
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
            20,
        )))
        .init_resource::<RunSession>()
        .init_resource::<SoundQueue>()
        .init_resource::<ControlBindings>()
        .insert_resource(PreferencesDir(None))
        .init_resource::<ButtonInput<KeyCode>>()
        .add_systems(Startup, controls::initialize_control_bindings)
        .add_systems(
            Startup,
            |mut commands: Commands, data: Res<GameData>, sheets: Res<SpriteSheets>| {
                let room = &data.rooms[&1];
                world::spawn_player(&mut commands, room, &data, &sheets, 10);
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
            (player::update_player, assets::animate_sprites).chain(),
        );
    app.update();
    let player = app
        .world_mut()
        .query_filtered::<Entity, With<PlayerBody>>()
        .single(app.world())
        .expect("one player should spawn");
    // The room was cleared mid-jump: the player hangs 2 H above the floor.
    let start_x = {
        let mut transform = app
            .world_mut()
            .get_mut::<Transform>(player)
            .expect("the player has a transform");
        transform.translation.y += 2.0 * H_WORLD_UNITS;
        transform.translation.x
    };
    if let Some(mut body) = app.world_mut().get_mut::<PlayerBody>(player) {
        body.grounded = false;
    }

    let mut idle_frames = BTreeSet::new();
    for _ in 0..150 {
        {
            let mut input = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
            input.release_all();
            input.clear();
            for key in [KeyCode::KeyD, KeyCode::Space, KeyCode::KeyJ] {
                input.press(key);
            }
        }
        app.update();
        let animation = app
            .world()
            .get::<AnimatedSprite>(player)
            .expect("the player keeps its animation");
        if animation.sheet == "player_idle" {
            idle_frames.insert(animation.frame);
        }
    }

    let body = app
        .world()
        .get::<PlayerBody>(player)
        .expect("the player keeps its body");
    let transform = app
        .world()
        .get::<Transform>(player)
        .expect("the player keeps its transform");
    assert!(body.grounded, "the player lands behind the bonus screen");
    assert!(
        (transform.translation.x - start_x).abs() < 0.01,
        "movement input is ignored while the bonus screen is open"
    );
    assert!(
        idle_frames.len() > 1,
        "the calm idle keeps animating, got frames {idle_frames:?}"
    );
}
