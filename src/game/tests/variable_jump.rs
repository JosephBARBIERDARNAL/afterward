use super::super::*;
use std::{path::Path, time::Duration};

use bevy::{prelude::MinimalPlugins, time::TimeUpdateStrategy};

/// Jumps from the ground, holding Space for `hold_frames` frames, and returns
/// the highest point the player's feet reach above the take-off, in H.
fn jump_peak_h(hold_frames: usize) -> f32 {
    let asset_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/assets");
    let data = GameData::load(&asset_root).expect("Chapter 1 data should load");
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(data)
        .insert_resource(SpriteSheets(Default::default()))
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_micros(
            1_000_000 / 240,
        )))
        .init_resource::<RunSession>()
        .init_resource::<SoundQueue>()
        .init_resource::<ControlBindings>()
        .init_resource::<ButtonInput<KeyCode>>()
        .add_systems(Startup, controls::initialize_default_control_bindings)
        .add_systems(Startup, |mut commands: Commands, data: Res<GameData>| {
            let room = &data.rooms[&1];
            let width = room.width_h(&data.gameplay) * H_WORLD_UNITS;
            let ground_top = room.player_spawn.y_h * H_WORLD_UNITS;
            commands.spawn((
                Transform::from_xyz(width / 2.0, ground_top + H_WORLD_UNITS / 2.0, 0.0),
                Sprite::default(),
                AnimatedSprite {
                    sheet: "player_idle".to_string(),
                    frame: 0,
                    frames: 1,
                    timer: Timer::from_seconds(1.0, TimerMode::Repeating),
                    looping: true,
                },
                PlayerBody {
                    velocity: Vec2::ZERO,
                    facing: 1.0,
                    grounded: true,
                    jumps_used: 0,
                    jump_held: false,
                    dash_timer: 0.0,
                    dash_cooldown: 0.0,
                    dash_direction: 1.0,
                    sword_cooldown: 0.0,
                    combo_step: 0,
                    sword_buffered: false,
                    invulnerability_timer: 0.0,
                    dash_invulnerability_timer: 0.0,
                    dash_strike_timer: 0.0,
                    rhythm_timer: 0.0,
                    rhythm_stacks: 0,
                    chain_reaction_timer: 0.0,
                    sated_fury_timer: 0.0,
                    action_animation_timer: 0.0,
                    footstep_timer: 0.0,
                },
            ));
            commands.spawn(PlatformSurface {
                left: 0.0,
                right: width,
                bottom: ground_top - H_WORLD_UNITS,
                top: ground_top,
                material: PlatformMaterial::Soil,
            });
        })
        .add_systems(Update, player::update_player);

    app.update();
    let start_y = {
        let mut players = app.world_mut().query::<(&Transform, &PlayerBody)>();
        players
            .single(app.world())
            .expect("one player should exist")
            .0
            .translation
            .y
    };
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::Space);
    let mut peak = start_y;
    for frame in 0..480 {
        app.update();
        let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        keys.clear();
        if frame + 1 == hold_frames {
            keys.release(KeyCode::Space);
        }
        let mut players = app.world_mut().query::<(&Transform, &PlayerBody)>();
        let (transform, body) = players
            .single(app.world())
            .expect("one player should exist");
        peak = peak.max(transform.translation.y);
        if body.grounded && frame > 0 {
            break;
        }
    }
    (peak - start_y) / H_WORLD_UNITS
}

#[test]
fn jump_height_grows_with_how_long_jump_is_held() {
    let full_height_h = 2.25;
    let tap = jump_peak_h(1);
    let medium = jump_peak_h(40);
    let held = jump_peak_h(480);

    assert!(
        (held - full_height_h).abs() < 0.1,
        "holding jump reaches the full jump height, got {held} H"
    );
    assert!(tap < medium && medium < held, "{tap} < {medium} < {held}");
    assert!(
        tap < full_height_h * 0.4,
        "a tap gives a short hop, got {tap} H"
    );
}
