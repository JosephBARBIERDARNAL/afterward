//! The devices bodies stand on: hovering platforms and blades that travel,
//! belts that carry, and platforms that fold away once stood on.

use super::*;

/// How close, in world units, feet must be to a surface's top to stand on it.
const STANDING_TOLERANCE: f32 = 2.0;

/// A device that travels from its spot and back: where its middle started,
/// and where it stood last frame.
#[derive(Component)]
pub(super) struct Mover {
    pub(super) spot: Vec2,
    pub(super) previous: Vec2,
}

/// A folding platform's state: how long it has been stood on, and how long
/// it has been gone with the collider it took away.
#[derive(Component, Default)]
pub(super) struct Folding {
    stood_s: Option<f32>,
    gone: Option<(f32, PlatformSurface)>,
}

/// Every device that travels, with what of it moves along.
type Movers<'world, 'state> = Query<
    'world,
    'state,
    (
        &'static mut Device,
        &'static Mover,
        &'static mut Transform,
        Option<&'static mut PlatformSurface>,
        Option<&'static mut combat::Hurts>,
    ),
>;

/// The enemies a carrier may carry.
type Riders<'world, 'state> = Query<
    'world,
    'state,
    (&'static mut Transform, &'static mut EnemyBody),
    (Without<PlayerBody>, Without<Device>),
>;

/// Whether feet at `feet` stand on `surface`'s top as it was at `top`.
fn stands_on(surface: &PlatformSurface, top: f32, feet: Vec2) -> bool {
    (feet.y - top).abs() <= STANDING_TOLERANCE
        && feet.x >= surface.left - HALF_BODY_WIDTH * 0.5
        && feet.x <= surface.right + HALF_BODY_WIDTH * 0.5
}

/// Moves every hovering platform and blade along its way, its collider and
/// what it hurts with it, and loops its frames.
pub(super) fn move_platforms(time: Res<Time>, mut movers: Movers) {
    let dt = time.delta_secs().min(0.05);
    for (mut device, mover, mut transform, surface, hurts) in &mut movers {
        let (travel, speed) = match &device.config.kind {
            DeviceKind::HoverPlatform(platform) => (
                Vec2::new(platform.travel_x_h, platform.travel_y_h),
                platform.speed_h_s,
            ),
            DeviceKind::Blade(blade) => (
                Vec2::new(blade.travel_x_h, blade.travel_y_h),
                blade.speed_h_s,
            ),
            _ => continue,
        };
        // Blades count their own time in `run_cycles`.
        if matches!(device.config.kind, DeviceKind::HoverPlatform(_)) {
            device.elapsed_s += dt;
            device.frame = (device.elapsed_s * device.fps) as usize % device.frames.max(1);
        }
        let at = mover.spot
            + travelled(
                travel * H_WORLD_UNITS,
                speed * H_WORLD_UNITS,
                device.elapsed_s,
            );
        let delta = at - transform.translation.truncate();
        transform.translation.x = at.x;
        transform.translation.y = at.y;
        if let Some(mut surface) = surface {
            surface.left += delta.x;
            surface.right += delta.x;
            surface.bottom += delta.y;
            surface.top += delta.y;
        }
        if let Some(mut hurts) = hurts {
            hurts.area.min += delta;
            hurts.area.max += delta;
        }
    }
}

/// Carries whoever stands on a hovering platform as it moves, and on a belt
/// along it.
pub(super) fn carry_riders(
    time: Res<Time>,
    mut carriers: Query<(&Device, Option<&mut Mover>, &Transform, &PlatformSurface)>,
    mut players: Query<&mut Transform, (With<PlayerBody>, Without<Device>)>,
    mut enemies: Riders,
) {
    let dt = time.delta_secs().min(0.05);
    for (device, mover, transform, surface) in &mut carriers {
        let (delta, top_before) = match (&device.config.kind, mover) {
            (DeviceKind::HoverPlatform(_), Some(mut mover)) => {
                let at = transform.translation.truncate();
                let delta = at - mover.previous;
                mover.previous = at;
                (delta, surface.top - delta.y)
            }
            (DeviceKind::Conveyor(belt), _) => (
                Vec2::new(belt.speed_h_s * H_WORLD_UNITS * dt, 0.0),
                surface.top,
            ),
            _ => continue,
        };
        if delta == Vec2::ZERO {
            continue;
        }
        for mut player in &mut players {
            let feet = player.translation.truncate() - Vec2::new(0.0, 0.5 * H_WORLD_UNITS);
            if stands_on(surface, top_before, feet) {
                player.translation += delta.extend(0.0);
            }
        }
        for (mut enemy_transform, mut enemy) in &mut enemies {
            let feet = Vec2::new(
                enemy_transform.translation.x,
                enemy.home_y - enemy.feet_offset,
            );
            if stands_on(surface, top_before, feet) {
                enemy_transform.translation += delta.extend(0.0);
                enemy.home_y += delta.y;
            }
        }
    }
}

/// Loops each belt's frames, backwards for one running left.
pub(super) fn run_conveyors(time: Res<Time>, mut belts: Query<&mut Device>) {
    let dt = time.delta_secs().min(0.05);
    for mut device in &mut belts {
        let DeviceKind::Conveyor(belt) = &device.config.kind else {
            continue;
        };
        let backwards = belt.speed_h_s < 0.0;
        device.elapsed_s += dt;
        let step = (device.elapsed_s * device.fps) as usize % device.frames.max(1);
        device.frame = if backwards {
            (device.frames - step) % device.frames.max(1)
        } else {
            step
        };
    }
}

/// Gives a folding platform way once someone has stood on it for its
/// `stand_s`, its frames showing it going, and brings it back `return_s` on.
pub(super) fn fold_platforms(
    mut commands: Commands,
    time: Res<Time>,
    mut platforms: Query<(
        Entity,
        &mut Device,
        &mut Folding,
        &mut Visibility,
        Option<&PlatformSurface>,
    )>,
    players: Query<&Transform, With<PlayerBody>>,
    enemies: Query<(&Transform, &EnemyBody)>,
) {
    let dt = time.delta_secs().min(0.05);
    for (entity, mut device, mut folding, mut visibility, surface) in &mut platforms {
        let DeviceKind::FoldingPlatform(platform) = device.config.kind.clone() else {
            continue;
        };
        if let Some((gone_s, collider)) = folding.gone.as_mut() {
            *gone_s += dt;
            if *gone_s >= platform.return_s {
                commands.entity(entity).insert(*collider);
                folding.gone = None;
                folding.stood_s = None;
                device.frame = 0;
                *visibility = Visibility::Inherited;
            }
            continue;
        }
        let Some(surface) = surface else {
            continue;
        };
        let stood_on = players.iter().any(|player| {
            stands_on(
                surface,
                surface.top,
                player.translation.truncate() - Vec2::new(0.0, 0.5 * H_WORLD_UNITS),
            )
        }) || enemies.iter().any(|(transform, enemy)| {
            stands_on(
                surface,
                surface.top,
                Vec2::new(transform.translation.x, enemy.home_y - enemy.feet_offset),
            )
        });
        // Once stood on it goes, whether its rider stays or not.
        if stood_on && folding.stood_s.is_none() {
            folding.stood_s = Some(0.0);
        }
        let Some(stood_s) = folding.stood_s.as_mut() else {
            continue;
        };
        *stood_s += dt;
        let share = *stood_s / platform.stand_s.max(f32::EPSILON);
        device.frame =
            ((share * device.frames as f32) as usize).min(device.frames.saturating_sub(1));
        if share >= 1.0 {
            folding.gone = Some((0.0, *surface));
            commands.entity(entity).remove::<PlatformSurface>();
            *visibility = Visibility::Hidden;
        }
    }
}
