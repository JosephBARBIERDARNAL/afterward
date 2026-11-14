//! Buttons and the doors they open.

use super::*;
use crate::game::interact::InReach;

/// Whether a button has been pressed, and how long ago.
#[derive(Component, Default)]
pub(crate) struct ButtonState {
    pressed_s: Option<f32>,
}

/// Whether a door has been opened, and how long ago.
#[derive(Component, Default)]
pub(crate) struct DoorState {
    open_s: Option<f32>,
}

/// Presses the button in reach when Interact goes down: it sinks in, and
/// every door sharing its link starts opening.
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
        if in_reach.0 != Some(entity) || button.pressed_s.is_some() {
            continue;
        }
        button.pressed_s = Some(0.0);
        sounds.push(SoundCue::DoorOpen);
        for (door_device, mut door) in &mut doors {
            if matches!(&door_device.config.kind, DeviceKind::Door(door_config)
                if door_config.link == config.link)
                && door.open_s.is_none()
            {
                door.open_s = Some(0.0);
            }
        }
    }
}

/// Plays a pressed button's frames, and opens a door: its light turns
/// green, it slides open, and it stops being solid.
pub(super) fn open_doors(
    mut commands: Commands,
    time: Res<Time>,
    mut buttons: Query<(&mut Device, &mut ButtonState), Without<DoorState>>,
    mut doors: Query<(Entity, &mut Device, &mut DoorState, Has<PlatformSurface>)>,
) {
    let dt = time.delta_secs().min(0.05);
    for (mut device, mut button) in &mut buttons {
        if let Some(pressed_s) = button.pressed_s.as_mut() {
            *pressed_s += dt;
            device.frame =
                ((*pressed_s * device.fps) as usize).min(device.frames.saturating_sub(1));
        }
    }
    for (entity, mut device, mut door, solid) in &mut doors {
        let Some(open_s) = door.open_s.as_mut() else {
            continue;
        };
        *open_s += dt;
        // Unlocked, then three frames opening, then held open.
        device.frame = (1 + (*open_s * device.fps) as usize).min(4);
        if device.frame >= 3 && solid {
            commands.entity(entity).remove::<PlatformSurface>();
        }
    }
}
