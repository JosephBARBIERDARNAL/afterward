use super::*;
use std::{path::Path, time::Duration};

use bevy::{prelude::MinimalPlugins, state::app::StatesPlugin, time::TimeUpdateStrategy};

mod control_bindings;
mod gameplay_controls;
mod player_controls;
mod procedural_movement;

type TestPilotPlayers<'w, 's> = Query<
    'w,
    's,
    (&'static mut Transform, &'static mut PlayerBody),
    (With<PlayerBody>, Without<EnemyBody>),
>;
type TestPilotEnemies<'w, 's> =
    Query<'w, 's, (&'static EnemyBody, &'static Transform), Without<PlayerBody>>;

#[test]
fn chapter_one_combat_run_reaches_completion_with_unique_bonus_choices() {
    let asset_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/assets");
    let data = GameData::load(&asset_root).expect("Chapter 1 data should load");
    let expected_rosters = data
        .rooms
        .iter()
        .map(|(room_id, room)| (*room_id, room.enemy_spawns.len()))
        .collect::<BTreeMap<_, _>>();
    let placeholder_sheet = SpriteSheet {
        image: Handle::default(),
        layout: Handle::default(),
        frames: 1,
        fps: 1.0,
    };
    let mut sheets =
        BTreeMap::from([(String::from("player_idle_sword"), placeholder_sheet.clone())]);
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
        .insert_state(GameScreen::Playing)
        .init_resource::<RunSession>()
        .insert_resource(RandomState(0x4a75_6572_6e65_7974))
        .insert_resource(PhaseTimer(Timer::from_seconds(1.0, TimerMode::Once)))
        .init_resource::<SoundQueue>()
        .init_resource::<ControlBindings>()
        .init_resource::<ButtonInput<KeyCode>>()
        .add_systems(Startup, controls::initialize_default_control_bindings)
        .add_systems(OnEnter(GameScreen::Playing), spawn_test_room)
        .add_systems(
            OnEnter(GameScreen::RoomLoading),
            (spawn_test_room, flow::start_room_transition_fade_out).chain(),
        )
        .add_systems(
            OnEnter(GameScreen::BossIntro),
            (spawn_test_room, flow::start_boss_intro).chain(),
        )
        .add_systems(
            Update,
            (
                |screen: Res<State<GameScreen>>,
                 data: Res<GameData>,
                 session: Res<RunSession>,
                 mut keys: ResMut<ButtonInput<KeyCode>>,
                 mut players: TestPilotPlayers<'_, '_>,
                 enemies: TestPilotEnemies<'_, '_>| {
                    keys.release_all();
                    keys.clear();
                    let Ok((mut transform, mut player)) = players.single_mut() else {
                        return;
                    };
                    player.velocity = Vec2::ZERO;
                    player.grounded = true;
                    player.invulnerability_timer = 10.0;
                    if *screen.get() != GameScreen::Playing {
                        return;
                    }
                    if let Some((_, enemy_transform)) =
                        enemies.iter().find(|(enemy, _)| enemy.health > 0.0)
                    {
                        let direction = if enemy_transform.translation.x >= transform.translation.x
                        {
                            1.0
                        } else {
                            -1.0
                        };
                        transform.translation.x =
                            enemy_transform.translation.x - direction * H_WORLD_UNITS;
                        transform.translation.y = enemy_transform.translation.y;
                        player.facing = direction;
                        keys.press(KeyCode::KeyK);
                    } else {
                        let room = &data.rooms[&session.room_id];
                        transform.translation.x = room.exit.x_h * H_WORLD_UNITS;
                        transform.translation.y =
                            room.exit.y_h * H_WORLD_UNITS + H_WORLD_UNITS / 2.0;
                        keys.press(KeyCode::KeyE);
                    }
                },
                player::update_player,
                combat::player_attacks,
                combat::update_enemy_ai,
                combat::update_projectiles,
                flow::handle_door_touch,
                flow::room_clear_check,
            )
                .chain()
                .run_if(in_state(GameScreen::Playing)),
        )
        .add_systems(
            Update,
            flow::update_room_clear.run_if(in_state(GameScreen::RoomClear)),
        )
        .add_systems(
            Update,
            (
                |screen: Res<State<GameScreen>>, mut keys: ResMut<ButtonInput<KeyCode>>| {
                    keys.release_all();
                    keys.clear();
                    if *screen.get() == GameScreen::Bonus {
                        keys.press(KeyCode::KeyE);
                    }
                },
                flow::handle_bonus_input,
            )
                .chain()
                .run_if(in_state(GameScreen::Bonus)),
        )
        .add_systems(
            Update,
            flow::update_room_transition.run_if(in_state(GameScreen::Transitioning)),
        )
        .add_systems(
            Update,
            flow::update_room_transition_fade_out.run_if(in_state(GameScreen::TransitioningOut)),
        )
        .add_systems(
            Update,
            flow::update_boss_intro.run_if(in_state(GameScreen::BossIntro)),
        );

    let mut bonus_rooms = BTreeSet::new();
    let mut cleared_rosters = BTreeMap::new();
    let mut saw_boss_intro = false;
    let mut enemy_bodies = app.world_mut().query::<&EnemyBody>();
    for _ in 0..10_000 {
        app.update();
        let screen = *app.world().resource::<State<GameScreen>>().get();
        let room_id = app.world().resource::<RunSession>().room_id;
        if screen == GameScreen::RoomClear && !cleared_rosters.contains_key(&room_id) {
            let (enemy_count, living_enemies) = {
                let enemies = enemy_bodies.iter(app.world());
                let mut enemy_count = 0;
                let mut living_enemies = 0;
                for enemy in enemies {
                    enemy_count += 1;
                    living_enemies += usize::from(enemy.health > 0.0);
                }
                (enemy_count, living_enemies)
            };
            assert_eq!(enemy_count, expected_rosters[&room_id]);
            assert_eq!(
                living_enemies, 0,
                "room {room_id} cleared with live enemies"
            );
            cleared_rosters.insert(room_id, enemy_count);
        }
        if screen == GameScreen::Bonus {
            bonus_rooms.insert(room_id);
        }
        saw_boss_intro |= screen == GameScreen::BossIntro;
        if screen == GameScreen::Complete {
            break;
        }
    }

    let living_enemy_count = enemy_bodies
        .iter(app.world())
        .filter(|enemy| enemy.health > 0.0)
        .count();
    let mut player_query = app
        .world_mut()
        .query_filtered::<(&Transform, &PlayerBody), With<PlayerBody>>();
    let (player_transform, player) = player_query
        .single(app.world())
        .expect("the player should remain spawned during the run");
    let player_position = player_transform.translation;
    let player_velocity = player.velocity;
    let player_facing = player.facing;
    let player_grounded = player.grounded;
    let player_dash_cooldown = player.dash_cooldown;
    let input = app.world().resource::<ButtonInput<KeyCode>>();
    let left_pressed = app
        .world()
        .resource::<ControlBindings>()
        .pressed(GameAction::MoveLeft, input);
    let right_pressed = app
        .world()
        .resource::<ControlBindings>()
        .pressed(GameAction::MoveRight, input);
    let mut living_enemies = app.world_mut().query::<(&EnemyBody, &Transform)>();
    let living_enemy_positions = living_enemies
        .iter(app.world())
        .filter(|(enemy, _)| enemy.health > 0.0)
        .map(|(enemy, transform)| {
            (
                enemy.type_id.to_string(),
                transform.translation.x / H_WORLD_UNITS,
                transform.translation.y / H_WORLD_UNITS,
                enemy.health,
                enemy.windup_remaining,
            )
        })
        .collect::<Vec<_>>();
    let session = app.world().resource::<RunSession>();
    let owned_bonuses = session.owned_bonuses.iter().collect::<BTreeSet<_>>();
    assert_eq!(
        *app.world().resource::<State<GameScreen>>().get(),
        GameScreen::Complete,
        "the simulated run should reach the Chapter 1 completion screen; room: {}, health: {}, living enemies: {}, enemy states: {:?}, door open: {}, player position: {:?}, velocity: {:?}, grounded: {}, dash cooldown: {}, facing: {}, left pressed: {}, right pressed: {}, current screen: {:?}, current offers: {:?}, selection: {}, interact binding: {}",
        session.room_id,
        session.health,
        living_enemy_count,
        living_enemy_positions,
        session.door_open,
        player_position,
        player_velocity,
        player_grounded,
        player_dash_cooldown,
        player_facing,
        left_pressed,
        right_pressed,
        app.world().resource::<State<GameScreen>>().get(),
        session.offered_bonuses,
        session.selected_option,
        app.world()
            .resource::<ControlBindings>()
            .binding_label(GameAction::Interact),
    );
    assert_eq!(session.room_id, 10);
    assert!(saw_boss_intro, "the run should show the room 10 boss intro");
    assert_eq!(bonus_rooms, (1..=10).collect());
    assert_eq!(session.owned_bonuses.len(), 10);
    assert_eq!(owned_bonuses.len(), session.owned_bonuses.len());
    assert_eq!(cleared_rosters, expected_rosters);
    let warrior = enemy_bodies
        .iter(app.world())
        .find(|enemy| enemy.type_id == "warrior")
        .expect("room 10 should spawn the Warrior from YAML");
    assert_eq!(warrior.health, 0.0);
    assert!(warrior.phase_two_announced);
}

#[test]
fn projectiles_collide_with_platforms_along_their_travel_path() {
    let platform = PlatformSurface {
        left: 100.0,
        right: 200.0,
        bottom: 0.0,
        top: 20.0,
    };

    assert!(platform.intersects_projectile(
        Vec2::new(0.0, 10.0),
        Vec2::new(110.0, 10.0),
        Vec2::splat(6.0),
    ));
    assert!(!platform.intersects_projectile(
        Vec2::new(0.0, 30.0),
        Vec2::new(110.0, 30.0),
        Vec2::splat(6.0),
    ));
    assert!(!platform.intersects_projectile(
        Vec2::new(0.0, 10.0),
        Vec2::new(50.0, 10.0),
        Vec2::splat(6.0),
    ));
}

fn spawn_test_room(
    mut commands: Commands,
    data: Res<GameData>,
    mut session: ResMut<RunSession>,
    sheets: Res<SpriteSheets>,
) {
    if session.world_loaded {
        return;
    }
    let room = &data.rooms[&session.room_id];
    world::spawn_room_platforms(&mut commands, room);
    world::spawn_player(&mut commands, room, &sheets);
    world::spawn_enemies(&mut commands, room, &data, &sheets);
    session.world_loaded = true;
}
