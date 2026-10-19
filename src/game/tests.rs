use super::*;
use std::{path::Path, time::Duration};

use bevy::{prelude::MinimalPlugins, state::app::StatesPlugin, time::TimeUpdateStrategy};

#[test]
fn chapter_one_room_flow_reaches_completion_with_unique_bonus_choices() {
    let asset_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/assets");
    let data = GameData::load(&asset_root).expect("Chapter 1 data should load");
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, StatesPlugin))
        .insert_resource(data)
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs(1)))
        .init_state::<GameScreen>()
        .insert_state(GameScreen::Playing)
        .init_resource::<RunSession>()
        .init_resource::<RandomState>()
        .insert_resource(PhaseTimer(Timer::from_seconds(1.0, TimerMode::Once)))
        .init_resource::<SoundQueue>()
        .init_resource::<ControlBindings>()
        .init_resource::<ButtonInput<KeyCode>>()
        .add_systems(
            Startup,
            controls::initialize_default_control_bindings,
        )
        .add_systems(OnEnter(GameScreen::Playing), spawn_test_player)
        .add_systems(
            OnEnter(GameScreen::RoomLoading),
            (spawn_test_player, flow::start_room_transition_fade_out).chain(),
        )
        .add_systems(
            OnEnter(GameScreen::BossIntro),
            (spawn_test_player, flow::start_boss_intro).chain(),
        )
        .add_systems(
            Update,
            (
                simulate_bonus_confirmation,
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
            flow::handle_bonus_input.run_if(in_state(GameScreen::Bonus)),
        )
        .add_systems(
            Update,
            flow::update_room_transition.run_if(in_state(GameScreen::Transitioning)),
        )
        .add_systems(
            Update,
            flow::update_room_transition_fade_out
                .run_if(in_state(GameScreen::TransitioningOut)),
        )
        .add_systems(
            Update,
            flow::update_boss_intro.run_if(in_state(GameScreen::BossIntro)),
        );

    for _ in 0..100 {
        app.update();
        if *app.world().resource::<State<GameScreen>>().get() == GameScreen::Complete {
            break;
        }
    }

    let session = app.world().resource::<RunSession>();
    let owned_bonuses = session
        .owned_bonuses
        .iter()
        .collect::<BTreeSet<_>>();
    assert_eq!(
        *app.world().resource::<State<GameScreen>>().get(),
        GameScreen::Complete,
        "the simulated run should reach the Chapter 1 completion screen"
    );
    assert_eq!(session.room_id, 10);
    assert_eq!(session.owned_bonuses.len(), 10);
    assert_eq!(owned_bonuses.len(), session.owned_bonuses.len());
}

fn spawn_test_player(
    mut commands: Commands,
    data: Res<GameData>,
    session: Res<RunSession>,
    players: Query<(), With<PlayerBody>>,
) {
    if players.iter().next().is_some() {
        return;
    }
    let room = &data.rooms[&session.room_id];
    commands.spawn((
        Transform::from_xyz(
            room.exit.x_h * H_WORLD_UNITS,
            room.exit.y_h * H_WORLD_UNITS + H_WORLD_UNITS / 2.0,
            3.0,
        ),
        PlayerBody {
            velocity: Vec2::ZERO,
            facing: 1.0,
            grounded: true,
            jumps_used: 0,
            dash_timer: 0.0,
            dash_cooldown: 0.0,
            dash_direction: 1.0,
            sword_cooldown: 0.0,
            gun_cooldown: 0.0,
            invulnerability_timer: 0.0,
            dash_invulnerability_timer: 0.0,
            dash_strike_timer: 0.0,
            stationary_timer: 0.0,
            rhythm_timer: 0.0,
            rhythm_stacks: 0,
            chain_reaction_timer: 0.0,
            sated_fury_timer: 0.0,
            action_animation_timer: 0.0,
            footstep_timer: 0.0,
        },
        RoomEntity,
    ));
    session.world_loaded = true;
}

fn simulate_bonus_confirmation(
    screen: Res<State<GameScreen>>,
    mut keys: ResMut<ButtonInput<KeyCode>>,
) {
    keys.clear();
    if *screen.get() == GameScreen::Bonus {
        keys.press(KeyCode::KeyE);
    }
}
