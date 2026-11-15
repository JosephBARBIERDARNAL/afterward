//! The devices that hurt on a cycle (lasers, fire traps, presses), on touch
//! (spikes, blades) or from afar (turrets).

use super::*;
use crate::data::TurretConfig;

/// How long a turret's shot flies before it fades, at most.
const SHOT_LIFE_S: f32 = 4.0;
/// The frame of the shot sheet a turret's shot shows.
const SHOT_FRAME: f32 = 1.0;

/// Steps every device on a cycle and shows where in it it is: its frame,
/// and whether it hurts. Spikes and blades always hurt.
pub(super) fn run_cycles(
    time: Res<Time>,
    data: Res<GameData>,
    mut devices: Query<(&mut Device, &mut combat::Hurts)>,
) {
    let dt = time.delta_secs().min(0.05);
    for (mut device, mut hurts) in &mut devices {
        device.elapsed_s += dt;
        let (frame, live) = Device::state(&device.config, &data.devices, device.elapsed_s);
        // A blade's frame is its speed's, which `move_platforms` knows.
        if device.frame != frame && !matches!(device.config.kind, DeviceKind::Blade(_)) {
            device.frame = frame;
        }
        if hurts.live != live {
            hurts.live = live;
        }
    }
}

/// How many shots a turret has fired.
#[derive(Component, Default)]
pub(super) struct TurretState {
    fired: i64,
}

/// Charges each turret up over its interval and fires a shot the way it
/// faces at the end of it. The shot is an enemy's: it stops at platforms
/// and hurts the player it reaches.
pub(super) fn run_turrets(
    mut commands: Commands,
    time: Res<Time>,
    data: Res<GameData>,
    asset_server: Res<AssetServer>,
    mut turrets: Query<(&mut Device, &mut TurretState, &Transform)>,
) {
    let dt = time.delta_secs().min(0.05);
    for (mut device, mut state, transform) in &mut turrets {
        let DeviceKind::Turret(turret) = device.config.kind.clone() else {
            continue;
        };
        device.elapsed_s += dt;
        // It fires `fire_s` before the end of each interval, then plays the
        // shot's frames out.
        let fire_s = (4.0 / device.fps).min(turret.interval_s);
        let charge_s = turret.interval_s - fire_s;
        let into = device.elapsed_s.rem_euclid(turret.interval_s);
        device.frame = if into < charge_s {
            (into / charge_s.max(f32::EPSILON) * 5.0) as usize % 5
        } else {
            5 + ((into - charge_s) * device.fps) as usize % 4
        };
        let fired = ((device.elapsed_s - charge_s) / turret.interval_s).floor() as i64 + 1;
        if fired > state.fired {
            state.fired = fired;
            fire_shot(
                &mut commands,
                &data,
                &asset_server,
                &device.config,
                &turret,
                transform,
            );
        }
    }
}

/// Lays one of a turret's shots at its muzzle.
fn fire_shot(
    commands: &mut Commands,
    data: &GameData,
    asset_server: &AssetServer,
    config: &DeviceConfig,
    turret: &TurretConfig,
    transform: &Transform,
) {
    let center = transform.translation.truncate();
    let Some(muzzle) = device_box(config, &data.devices, "muzzle", center) else {
        return;
    };
    // The way it faces, mirrored and turned as the turret is.
    let placement = &config.placement;
    let muzzle = placement.place(muzzle.center(), center);
    let facing = f32::from(turret.facing) * if placement.mirrored { -1.0 } else { 1.0 };
    let heading = Vec2::from_angle(placement.rotation_deg.to_radians()) * facing;
    let px = data.devices.px_per_h;
    let size = Vec2::splat(16.0 / px) * H_WORLD_UNITS;
    let speed = turret.shot_speed_h_s * H_WORLD_UNITS;
    commands.spawn((
        Sprite {
            image: room_art::load_texture(asset_server, "effects/sci_fi/shot_red.png"),
            rect: Some(Rect::new(
                SHOT_FRAME * 16.0,
                0.0,
                SHOT_FRAME * 16.0 + 16.0,
                16.0,
            )),
            custom_size: Some(size),
            ..default()
        },
        // The shot is drawn heading right, and turned the way it flies.
        Transform::from_translation(muzzle.extend(DEVICE_Z + 0.1))
            .with_rotation(Quat::from_rotation_z(heading.to_angle())),
        combat::CombatProjectile {
            velocity_x: heading.x * speed,
            velocity_y: heading.y * speed,
            damage: turret.damage,
            life_s: SHOT_LIFE_S,
            remaining_range: speed * SHOT_LIFE_S,
            half_size: size / 4.0,
        },
        RoomEntity,
    ));
}
