use super::super::*;
use std::{path::Path, time::Duration};

use bevy::{prelude::MinimalPlugins, state::app::StatesPlugin, time::TimeUpdateStrategy};

/// What the chain is heard doing: each blow that lands plays the punch of its
/// own step, and one that reaches nobody is heard cutting the air instead.
#[test]
fn a_landed_blow_plays_the_punch_of_its_step_and_a_missed_one_cuts_the_air() {
    let asset_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/assets");
    let mut data = GameData::load(&asset_root).expect("Chapter 1 data should load");
    // One body of polymer inside arm's reach, with the health to take the
    // chain's first two blows and fall to its third, so the fourth finds
    // nobody there.
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
  - type_id: stalker_unit
    position_h: { x: 3.05, y: 1 }
    stat_overrides: { health: 60 }
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
        .insert_resource(RandomState(0x7075_6e63_6821_2121))
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
            (player::update_player, combat::player_melee).chain(),
        );
    app.update();

    // One blow thrown and played out, and the punches heard over it. Nothing
    // plays the queue here, so it holds every cue since it was last emptied.
    let blow = |app: &mut App| {
        app.world_mut().resource_mut::<SoundQueue>().0.clear();
        for attack in [true, false, false, false, false, false, false, false] {
            {
                let mut input = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
                input.release_all();
                input.clear();
                if attack {
                    input.press(KeyCode::KeyJ);
                }
            }
            app.update();
        }
        app.world()
            .resource::<SoundQueue>()
            .0
            .iter()
            .copied()
            .filter(|cue| matches!(cue, SoundCue::MeleeHit(_) | SoundCue::MeleeMiss))
            .collect::<Vec<_>>()
    };

    assert_eq!(blow(&mut app), [SoundCue::MeleeHit(0)]);
    assert_eq!(blow(&mut app), [SoundCue::MeleeHit(1)]);
    assert_eq!(blow(&mut app), [SoundCue::MeleeHit(2)]);
    assert_eq!(
        blow(&mut app),
        [SoundCue::MeleeMiss],
        "with the body down the next blow reaches nobody"
    );
}
