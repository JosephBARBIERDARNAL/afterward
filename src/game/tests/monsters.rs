//! The monsters of `assets/monster_sheets.rs`: each finds a standing player
//! and wounds them with the attack its type is given.

use super::super::*;
use crate::game::combat::CombatProjectile;
use std::{path::Path, time::Duration};

use bevy::{prelude::MinimalPlugins, state::app::StatesPlugin, time::TimeUpdateStrategy};

#[test]
fn every_monster_reaches_a_standing_player_and_the_eye_beast_does_it_with_its_void_ball() {
    for monster in [
        "abomination",
        "mage",
        "catto",
        "mad_ghost",
        "eye_beast",
        "abomination_rust",
        "mage_gold",
        "catto_purple",
    ] {
        let asset_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/assets");
        let mut data = GameData::load(&asset_root).expect("Chapter 1 data should load");
        // A flat hall, independent of the authored rooms, with the monster
        // in sight of the player.
        *data.rooms.get_mut(&1).expect("room 1 should exist") = yaml_serde::from_str(&format!(
            "
room_id: 1
chapter_id: 1
size: {{ width_h: 48, height_h: 13.5 }}
background_profiles: [clouds_bright_blue]
player_spawn: {{ x_h: 10, y_h: 1 }}
platforms:
  - {{ material: city_pavement, rect_h: {{ x: 0, y: 0, width: 48, height: 1 }} }}
enemy_spawns:
  - {{ type_id: {monster}, position_h: {{ x: 17, y: 1 }} }}
bonus: {{ x_h: 43, y_h: 1 }}
exit: {{ x_h: 45, y_h: 1 }}
next_room_id: 2
"
        ))
        .expect("the test layout should parse");
        let scale = data.enemies.types[monster]
            .sprite
            .expect("a monster says how it is drawn")
            .scale;
        let sheet = |frames| SpriteSheet {
            image: Handle::default(),
            layout: Handle::default(),
            frame_size: Vec2::new(64.0, 64.0),
            frames,
            fps: 12.0,
        };
        let sheets = BTreeMap::from([
            (String::from("player_idle"), sheet(1)),
            (format!("{monster}_idle"), sheet(4)),
            (format!("{monster}_attack"), sheet(13)),
            (String::from("eye_beast_shot"), sheet(3)),
        ]);

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
                health: 100.0,
                max_health: 100.0,
                ..default()
            })
            .init_resource::<SoundQueue>()
            .add_systems(
                Startup,
                |mut commands: Commands, data: Res<GameData>, sheets: Res<SpriteSheets>| {
                    let room = &data.rooms[&1];
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
                    combat::update_projectiles,
                    combat::update_hazards,
                )
                    .chain(),
            );

        let mut shots_seen = 0;
        for _ in 0..600 {
            app.update();
            let world = app.world_mut();
            for (shot, transform) in world
                .query_filtered::<(&AnimatedSprite, &Transform), With<CombatProjectile>>()
                .iter(world)
            {
                shots_seen += 1;
                assert_eq!(shot.sheet, "eye_beast_shot");
                assert_eq!(transform.scale, Vec3::splat(scale));
            }
            if world.resource::<RunSession>().health < 100.0 {
                break;
            }
        }
        assert!(
            app.world().resource::<RunSession>().health < 100.0,
            "the {monster} should have wounded the player"
        );
        assert_eq!(
            shots_seen > 0,
            monster == "eye_beast",
            "only the eye beast fires, and its shot wears its own sheet"
        );
    }
}
