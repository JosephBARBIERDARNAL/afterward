use super::super::*;
use std::{path::Path, time::Duration};

use bevy::{prelude::MinimalPlugins, state::app::StatesPlugin, time::TimeUpdateStrategy};

/// Wall Runner runs a player pressed into a wall in mid-air up it, as high as
/// the bonus allows, in a clip of its own; then they cling and slide as
/// anyone would.
#[test]
fn wall_runner_runs_up_a_wall_as_high_as_it_allows() {
    // The highest the player's middle reaches jumping into a tall slab with
    // the slab held into, and whether they were seen running up it.
    let climb = |bonuses: Vec<BonusPick>| {
        let asset_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/assets");
        let mut data = GameData::load(&asset_root).expect("Chapter 1 data should load");
        *data.rooms.get_mut(&1).expect("room 1 should exist") = yaml_serde::from_str(
            "
room_id: 1
chapter_id: 1
size: { width_h: 24, height_h: 20 }
background_profiles: [city_teal_evening]
player_spawn: { x_h: 2.5, y_h: 1 }
platforms:
  - { material: city_pavement, rect_h: { x: 0, y: 0, width: 24, height: 1 } }
  - { material: city_plating, rect_h: { x: 4, y: 1, width: 2, height: 16 } }
enemy_spawns: []
bonus: { x_h: 20, y_h: 1 }
exit: { x_h: 22, y_h: 1 }
",
        )
        .expect("the test layout should parse");
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
            .insert_state(GameScreen::Playing)
            .insert_resource(data)
            .insert_resource(SpriteSheets(sheets))
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
                20,
            )))
            .insert_resource(RunSession {
                room_id: 1,
                owned_bonuses: bonuses,
                ..default()
            })
            .insert_resource(RandomState(7))
            .init_resource::<SoundQueue>()
            .add_message::<BonusTell>()
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
                },
            )
            .add_systems(Update, player::update_player);
        app.update();
        let player = app
            .world_mut()
            .query_filtered::<Entity, With<PlayerBody>>()
            .single(app.world())
            .expect("one player should spawn");
        let mut input = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        input.press(KeyCode::KeyD);
        for _ in 0..40 {
            app.update();
        }
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Space);
        let mut peak_h = 0.0_f32;
        let mut ran = false;
        for _ in 0..200 {
            app.update();
            app.world_mut()
                .resource_mut::<ButtonInput<KeyCode>>()
                .clear();
            let height_h = app
                .world()
                .get::<Transform>(player)
                .expect("the player stays")
                .translation
                .y
                / H_WORLD_UNITS;
            peak_h = peak_h.max(height_h);
            ran |= app
                .world()
                .get::<AnimatedSprite>(player)
                .expect("the player keeps its animation")
                .sheet
                == "player_wall_run";
        }
        (peak_h, ran)
    };
    let (unaided_h, ran_unaided) = climb(Vec::new());
    assert!(!ran_unaided, "without the bonus a wall is only slid down");
    let (run_h, ran) = climb(vec![BonusPick::new("wall_runner", BonusRarity::Legendary)]);
    assert!(ran, "running up the wall plays its own clip");
    assert!(
        (run_h - unaided_h - 5.0).abs() < 0.6,
        "a legendary Wall Runner adds its 5 H to the jump: {unaided_h} H alone, {run_h} H with it"
    );
}
