use super::super::*;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
    time::Duration,
};

use bevy::{
    input::keyboard::KeyboardInput, prelude::MinimalPlugins, state::app::StatesPlugin,
    time::TimeUpdateStrategy,
};

const TEST_PILOT_COMBAT_DISTANCE_H: f32 = 4.0;

type TestPilotPlayers<'w, 's> = Query<
    'w,
    's,
    (&'static Transform, &'static PlayerBody),
    (With<PlayerBody>, Without<EnemyBody>),
>;
type TestPilotEnemies<'w, 's> =
    Query<'w, 's, (&'static EnemyBody, &'static Transform), Without<PlayerBody>>;

/// A scripted pilot plays all of Chapter 1, so this depends on every room's
/// layout. It is ignored while the rooms are being designed: run it on demand
/// with `just playthrough`.
#[test]
#[ignore = "depends on the authored room layouts; run with `just playthrough`"]
fn chapter_one_combat_run_reaches_completion_with_unique_bonus_choices() {
    let asset_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/assets");
    let data = GameData::load(&asset_root).expect("Chapter 1 data should load");
    // Room 0 is empty: its bonus is taken, but no roster is ever cleared.
    let expected_rosters = data
        .rooms
        .iter()
        .filter(|(_, room)| !room.enemy_spawns.is_empty())
        .map(|(room_id, room)| (*room_id, room.enemy_spawns.len()))
        .collect::<BTreeMap<_, _>>();
    let placeholder_sheet = SpriteSheet {
        image: Handle::default(),
        layout: Handle::default(),
        frame_size: Vec2::new(128.0, 128.0),
        frames: 1,
        fps: 1.0,
    };
    let mut sheets = BTreeMap::from([
        (String::from("player_idle"), placeholder_sheet.clone()),
        (ExitDoor::SHEET.to_string(), placeholder_sheet.clone()),
    ]);
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
        .init_resource::<RunSession>()
        .init_resource::<RoomPan>()
        .init_resource::<InReach>()
        .init_resource::<StartRoom>()
        .init_resource::<AudioSettings>()
        .insert_resource(RandomState(0x4a75_6572_6e65_7974))
        .insert_resource(PhaseTimer(Timer::from_seconds(1.0, TimerMode::Once)))
        .init_resource::<SoundQueue>()
        .init_resource::<ControlBindings>()
        .insert_resource(PreferencesDir(None))
        .init_resource::<ButtonInput<KeyCode>>()
        .add_message::<KeyboardInput>()
        .add_message::<PointerOver>()
        .add_message::<PointerClick>()
        // The pan between rooms moves the camera; this harness draws nothing,
        // so it stands one in for the pan to carry.
        .add_systems(
            Startup,
            (
                controls::initialize_control_bindings,
                |mut commands: Commands| {
                    commands.spawn((Camera2d, Transform::default()));
                },
            ),
        )
        .add_systems(OnEnter(GameScreen::Playing), spawn_test_room)
        .add_systems(
            Update,
            (
                |mut keys: ResMut<ButtonInput<KeyCode>>| {
                    keys.release_all();
                    keys.clear();
                    keys.press(KeyCode::KeyE);
                },
                screens::handle_screen_input,
            )
                .chain()
                .run_if(in_state(GameScreen::MainMenu).or_else(in_state(GameScreen::Difficulty))),
        )
        .add_systems(
            OnEnter(GameScreen::RoomLoading),
            (
                room_pan::send_off_the_departing_room,
                spawn_test_room,
                room_pan::start_room_pan,
            )
                .chain(),
        )
        .add_systems(
            OnEnter(GameScreen::RoomIntro),
            (spawn_test_room, room_intro::start_room_intro).chain(),
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
                 players: TestPilotPlayers<'_, '_>,
                 enemies: TestPilotEnemies<'_, '_>| {
                    // Keep holding jump like a player would: re-pressing it
                    // every frame would count as fresh presses.
                    let space_was_held = keys.pressed(KeyCode::Space);
                    let press_jump = |keys: &mut ButtonInput<KeyCode>| {
                        keys.press(KeyCode::Space);
                        if space_was_held {
                            keys.clear_just_pressed(KeyCode::Space);
                        }
                    };
                    keys.release_all();
                    keys.clear();
                    if *screen.get() != GameScreen::Playing {
                        return;
                    }
                    let Ok((player_transform, player)) = players.single() else {
                        return;
                    };
                    if let Some((_, enemy_transform)) = enemies
                        .iter()
                        .filter(|(enemy, _)| enemy.health > 0.0)
                        .min_by(|(_, first), (_, second)| {
                            first
                                .translation
                                .truncate()
                                .distance_squared(player_transform.translation.truncate())
                                .total_cmp(
                                    &second
                                        .translation
                                        .truncate()
                                        .distance_squared(player_transform.translation.truncate()),
                                )
                        })
                    {
                        let dx = enemy_transform.translation.x - player_transform.translation.x;
                        let enemy_above_player = enemy_transform.translation.y
                            > player_transform.translation.y + H_WORLD_UNITS * 0.5;
                        // Shoot up diagonally at an enemy above, from about as
                        // far to the side as it is high.
                        let rise_h = (enemy_transform.translation.y
                            - player_transform.translation.y)
                            / H_WORLD_UNITS;
                        let combat_distance_h = if enemy_above_player {
                            rise_h.clamp(1.0, 3.0)
                        } else {
                            TEST_PILOT_COMBAT_DISTANCE_H
                        };
                        if enemy_above_player {
                            keys.press(KeyCode::KeyW);
                        }
                        // Bullets stop at the platform under an enemy well
                        // below: walk off the ledge toward it (on, in the
                        // facing direction, when it is right below).
                        let step_off_ledge = enemy_transform.translation.y
                            < player_transform.translation.y - H_WORLD_UNITS * 1.5
                            && player.grounded;
                        let walking = step_off_ledge
                            || dx.abs() > H_WORLD_UNITS * combat_distance_h
                            || dx.signum() != player.facing;
                        let direction = if step_off_ledge && dx.abs() < H_WORLD_UNITS * 1.5 {
                            player.facing
                        } else {
                            dx.signum()
                        };
                        if walking {
                            keys.press(if direction >= 0.0 {
                                KeyCode::KeyD
                            } else {
                                KeyCode::KeyA
                            });
                        }
                        // An enemy right below a plank: drop through it.
                        let enemy_below = enemy_transform.translation.y
                            < player_transform.translation.y - H_WORLD_UNITS
                            && dx.abs() < H_WORLD_UNITS * 1.5;
                        if enemy_below && player.grounded {
                            keys.press(KeyCode::KeyS);
                            press_jump(&mut keys);
                        }
                        // Jumping at an enemy out of reach would keep the pilot
                        // airborne forever, and enemies only drop down to a
                        // grounded player; wait below for those instead.
                        let enemy_within_jump_reach = enemy_transform.translation.y
                            - player_transform.translation.y
                            <= data.gameplay.player.jump_height_h * H_WORLD_UNITS;
                        // Hold jump through the rise for a full-height jump.
                        if (enemy_above_player && enemy_within_jump_reach && player.grounded)
                            || player.jump_held
                        {
                            press_jump(&mut keys);
                        }
                        keys.press(KeyCode::KeyJ);
                    } else {
                        let room = &data.rooms[&session.room_id];
                        // The bonus item first, then the door it unlocks.
                        let target = if session.door_open {
                            room.exit
                        } else {
                            room.bonus
                        };
                        let feet_h = player_transform.translation.y / H_WORLD_UNITS - 0.5;
                        // Asked every frame: it only does anything in reach.
                        keys.press(KeyCode::KeyE);
                        let exit_below = target.y_h < feet_h - 0.5 && player.grounded;
                        if exit_below {
                            keys.press(KeyCode::KeyS);
                            press_jump(&mut keys);
                        } else if target.x_h * H_WORLD_UNITS - player_transform.translation.x
                            > H_WORLD_UNITS * 0.65
                        {
                            keys.press(KeyCode::KeyD);
                            if player.jump_held {
                                press_jump(&mut keys);
                            }
                        }
                    }
                },
                player::update_player,
                combat::player_melee,
                combat::update_enemy_ai,
                combat::update_enemy_platform_physics,
                combat::update_projectiles,
                combat::update_health_pickups,
                combat::update_player_death,
                bonus_pickup::spawn_bonus_pickup,
                interact::mark_what_is_in_reach,
                bonus_pickup::update_bonus_pickup,
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
                |screen: Res<State<GameScreen>>,
                 session: Res<RunSession>,
                 mut keys: ResMut<ButtonInput<KeyCode>>| {
                    keys.release_all();
                    keys.clear();
                    if *screen.get() == GameScreen::Bonus {
                        let preferred_bonus = [
                            "room_ward",
                            "vitality",
                            "thick_skin",
                            "vampiric_rounds",
                            "last_round",
                            "rhythm_training",
                        ]
                        .into_iter()
                        .find(|preferred| {
                            session
                                .offered_bonuses
                                .iter()
                                .any(|offered| offered == preferred)
                        });
                        if preferred_bonus.is_some_and(|preferred| {
                            session
                                .offered_bonuses
                                .get(1)
                                .is_some_and(|offered| offered == preferred)
                        }) {
                            keys.press(KeyCode::Digit2);
                        } else {
                            keys.press(KeyCode::Digit1);
                        }
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
            room_pan::update_room_pan.run_if(in_state(GameScreen::Panning)),
        )
        .add_systems(
            Update,
            room_intro::update_room_intro.run_if(in_state(GameScreen::RoomIntro)),
        )
        .add_systems(
            Update,
            flow::update_boss_intro.run_if(in_state(GameScreen::BossIntro)),
        );

    assert_eq!(
        *app.world().resource::<State<GameScreen>>().get(),
        GameScreen::MainMenu,
        "a complete run should begin at the main menu"
    );

    let mut bonus_rooms = BTreeSet::new();
    let mut cleared_rosters = BTreeMap::new();
    let mut saw_boss_intro = false;
    let mut intro_rooms = BTreeSet::new();
    let mut enemy_bodies = app.world_mut().query::<&EnemyBody>();
    let mut targets = app.world_mut().query::<(&EnemyBody, &Transform)>();
    let mut player_bodies = app.world_mut().query::<(&mut Transform, &mut PlayerBody)>();
    let mut last_progress = None;
    let mut frames_without_progress = 0;
    for _ in 0..10_000 {
        app.update();
        let screen = *app.world().resource::<State<GameScreen>>().get();
        let room_id = app.world().resource::<RunSession>().room_id;
        // Shortcut: the pilot fights through inputs but cannot find its way
        // through a room's layout. When nothing progresses for a while, it
        // travels next to the nearest living enemy, or to the open exit.
        let door_open = app.world().resource::<RunSession>().door_open;
        let enemy_health = enemy_bodies
            .iter(app.world())
            .map(|enemy| enemy.health.max(0.0))
            .sum::<f32>();
        let progress = (screen, room_id, door_open, enemy_health.to_bits());
        if last_progress == Some(progress) {
            frames_without_progress += 1;
        } else {
            last_progress = Some(progress);
            frames_without_progress = 0;
        }
        if screen == GameScreen::Playing && frames_without_progress >= 120 {
            frames_without_progress = 0;
            let data = app.world().resource::<GameData>();
            let room = &data.rooms[&room_id];
            let player_x = player_bodies
                .iter(app.world())
                .next()
                .map_or(0.0, |(transform, _)| transform.translation.x);
            let room_cleared = app.world().resource::<RunSession>().room_cleared;
            let feet_h = if door_open {
                Some(Vec2::new(room.exit.x_h - 2.0, room.exit.y_h))
            } else if room_cleared {
                Some(Vec2::new(room.bonus.x_h, room.bonus.y_h))
            } else {
                targets
                    .iter(app.world())
                    .filter(|(enemy, _)| enemy.health > 0.0 && enemy.grounded)
                    .min_by(|(_, first), (_, second)| {
                        (first.translation.x - player_x)
                            .abs()
                            .total_cmp(&(second.translation.x - player_x).abs())
                    })
                    .map(|(enemy, transform)| {
                        let x_h = transform.translation.x / H_WORLD_UNITS;
                        let y_h = (transform.translation.y - enemy.feet_offset) / H_WORLD_UNITS;
                        [-2.5, 2.5, -1.5, 1.5]
                            .into_iter()
                            .map(|offset| x_h + offset)
                            .find(|x_h| {
                                // On a platform top, beside the enemy.
                                room.platforms.iter().map(|block| block.rect_h).any(|rect| {
                                    (rect.x..=rect.right()).contains(x_h)
                                        && (rect.top() - y_h).abs() <= 0.01
                                })
                            })
                            .map_or(Vec2::new(x_h, y_h), |x_h| Vec2::new(x_h, y_h))
                    })
            };
            if let Some(feet_h) = feet_h {
                for (mut transform, mut body) in player_bodies.iter_mut(app.world_mut()) {
                    transform.translation.x = feet_h.x * H_WORLD_UNITS;
                    transform.translation.y = (feet_h.y + 0.5) * H_WORLD_UNITS;
                    body.velocity = Vec2::ZERO;
                }
            }
        }
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
        if screen == GameScreen::RoomIntro {
            intro_rooms.insert(room_id);
        }
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
        "the input-driven run should reach the Chapter 1 completion screen; room: {}, health: {}, bonuses: {:?}, living enemies: {}, enemy states: {:?}, door open: {}, player position: {:?}, velocity: {:?}, grounded: {}, facing: {}, left pressed: {}, right pressed: {}, current screen: {:?}, current offers: {:?}, interact binding: {}",
        session.room_id,
        session.health,
        session.owned_bonuses,
        living_enemy_count,
        living_enemy_positions,
        session.door_open,
        player_position,
        player_velocity,
        player_grounded,
        player_facing,
        left_pressed,
        right_pressed,
        app.world().resource::<State<GameScreen>>().get(),
        session.offered_bonuses,
        app.world()
            .resource::<ControlBindings>()
            .binding_label(GameAction::Interact),
    );
    assert_eq!(session.room_id, 10);
    assert!(saw_boss_intro, "the run should show the room 10 boss intro");
    assert_eq!(
        intro_rooms,
        (0..=10).collect(),
        "every room should open on its entrance animation"
    );
    assert_eq!(bonus_rooms, (0..=10).collect());
    assert_eq!(session.owned_bonuses.len(), 11);
    assert_eq!(owned_bonuses.len(), session.owned_bonuses.len());
    assert_eq!(cleared_rosters, expected_rosters);
    let emitter_unit = enemy_bodies
        .iter(app.world())
        .find(|enemy| enemy.type_id == "emitter_unit")
        .expect("room 10 should spawn the Evil Wizard from YAML");
    assert_eq!(emitter_unit.health, 0.0);
    assert!(emitter_unit.phase_two_announced);
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
    world::spawn_room_platforms(&mut commands, room, &data);
    world::spawn_player(&mut commands, room, &data, &sheets);
    world::spawn_enemies(&mut commands, room, &data, &sheets);
    world::spawn_exit_door(&mut commands, room, &data, &sheets);
    session.world_loaded = true;
}
