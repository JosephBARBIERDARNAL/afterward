//! Buttons, the doors they open, and ladders.

use super::*;
use crate::game::interact::InReach;

/// Whether a button is pressed in, and how long since it last moved.
#[derive(Component)]
pub(crate) struct ButtonState {
    pressed: bool,
    since_s: f32,
}

impl Default for ButtonState {
    /// Up, and long since.
    fn default() -> Self {
        Self {
            pressed: false,
            since_s: f32::INFINITY,
        }
    }
}

/// Whether a door is open, and how long since it last opened or closed.
#[derive(Component)]
pub(crate) struct DoorState {
    open: bool,
    since_s: f32,
}

impl Default for DoorState {
    /// Closed, and long since.
    fn default() -> Self {
        Self {
            open: false,
            since_s: f32::INFINITY,
        }
    }
}

/// Every door, with what it opens and closes with.
type Doors<'world, 'state> = Query<
    'world,
    'state,
    (
        Entity,
        &'static mut Device,
        &'static mut DoorState,
        &'static Transform,
        Has<PlatformSurface>,
        Option<&'static Children>,
    ),
>;

/// One of the stubs an open door leaves at its top and bottom: a child of
/// the door, solid as long as the door stands open.
#[derive(Component)]
pub(super) struct DoorStub;

/// A ladder's column, from its foot to its top, in world units: where the
/// player grabs it and climbs (`player::update_player`).
#[derive(Component)]
pub(crate) struct Ladder {
    pub(crate) area: Rect,
}

/// Presses the button in reach when Interact goes down: it sinks in and
/// every door sharing its link opens, or, pressed a second time, it rises
/// again and they close.
pub(crate) fn press_buttons(
    keys: Res<ButtonInput<KeyCode>>,
    controls: Res<ControlBindings>,
    in_reach: Res<InReach>,
    mut sounds: ResMut<SoundQueue>,
    mut buttons: Query<(Entity, &Device, &mut ButtonState)>,
    mut doors: Query<(&Device, &mut DoorState), Without<ButtonState>>,
) {
    if !controls.just_pressed(GameAction::Interact, &keys) {
        return;
    }
    for (entity, device, mut button) in &mut buttons {
        let DeviceKind::Button(config) = &device.config.kind else {
            continue;
        };
        if in_reach.0 != Some(entity) {
            continue;
        }
        button.pressed = !button.pressed;
        button.since_s = 0.0;
        sounds.push(SoundCue::DoorOpen);
        for (door_device, mut door) in &mut doors {
            if matches!(&door_device.config.kind, DeviceKind::Door(door_config)
                if door_config.link == config.link)
                && door.open != button.pressed
            {
                door.open = button.pressed;
                door.since_s = 0.0;
            }
        }
    }
}

/// Plays a button's frames as it sinks or rises, and a door's as it opens
/// or closes. An opening door stops being solid once it slides apart, all
/// but the stubs left at its top and bottom; a closing one is solid again
/// once it has shut.
pub(super) fn open_doors(
    mut commands: Commands,
    time: Res<Time>,
    data: Res<GameData>,
    mut buttons: Query<(&mut Device, &mut ButtonState), Without<DoorState>>,
    mut doors: Doors,
    stubs: Query<(), With<DoorStub>>,
) {
    let dt = time.delta_secs().min(0.05);
    for (mut device, mut button) in &mut buttons {
        button.since_s += dt;
        let sunk = ((button.since_s * device.fps) as usize).min(device.frames.saturating_sub(1));
        device.frame = if button.pressed {
            sunk
        } else {
            device.frames.saturating_sub(1) - sunk
        };
    }
    for (entity, mut device, mut door, transform, solid, children) in &mut doors {
        door.since_s += dt;
        let step = (door.since_s * device.fps) as usize;
        let center = transform.translation.truncate();
        if door.open {
            // Unlocked, then three frames opening, then held open.
            device.frame = (1 + step).min(4);
            if device.frame >= 3 && solid {
                commands.entity(entity).remove::<PlatformSurface>();
                for name in ["open_top", "open_bottom"] {
                    if let Some(area) = device_box(&device.config, &data.devices, name, center) {
                        commands.spawn((surface(area, &data), DoorStub, ChildOf(entity)));
                    }
                }
            }
        } else {
            // Two frames closing, then shut.
            device.frame = if step < 2 { 5 + step } else { 0 };
            if device.frame == 0
                && !solid
                && let Some(area) = device_box(&device.config, &data.devices, "solid", center)
            {
                commands.entity(entity).insert(surface(area, &data));
                for child in children.into_iter().flatten() {
                    if stubs.contains(*child) {
                        commands.entity(*child).despawn();
                    }
                }
            }
        }
    }
}
