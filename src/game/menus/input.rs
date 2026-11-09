//! Walking a menu and pressing its buttons, with the mouse or the keyboard.
//!
//! This decides *which* button the player is on and *that* one was pressed; it
//! does not carry it out. A press leaves a [`MenuPressed`], which
//! `screens::handle_screen_input` performs alongside the letter shortcuts, so
//! a click and a shortcut reach exactly the same code.
//!
//! Every move of the focus drops a droplet on the button reached: where the
//! mouse touched it, or in its middle when an arrow walked there.

use super::*;
use crate::game::ripple::{Splash, Spot, UiPointer};

/// A menu button the player pressed.
#[derive(Message, Clone, Copy)]
pub(in crate::game) struct MenuPressed {
    pub(in crate::game) action: MenuAction,
    /// How far along a volume bar a click landed, 0 to 1. `None` whenever the
    /// press did not aim inside one, which is every press but that.
    pub(in crate::game) along_bar: Option<f32>,
}

#[derive(SystemParam)]
pub(in crate::game) struct MenuInput<'w, 's> {
    screen: Res<'w, State<GameScreen>>,
    session: Res<'w, RunSession>,
    keys: Res<'w, ButtonInput<KeyCode>>,
    controls: Res<'w, ControlBindings>,
    focus: ResMut<'w, MenuFocus>,
    settings: ResMut<'w, AudioSettings>,
    preferences: Res<'w, PreferencesDir>,
    sounds: ResMut<'w, SoundQueue>,
    buttons: Query<'w, 's, (Entity, &'static MenuButton)>,
    bars: Query<
        'w,
        's,
        (
            &'static VolumeBar,
            &'static ComputedNode,
            &'static UiGlobalTransform,
        ),
    >,
    pointer: UiPointer<'w, 's>,
}

pub(in crate::game) fn navigate_menus(
    input: MenuInput,
    mut pointer_overs: MessageReader<PointerOver>,
    mut pointer_clicks: MessageReader<PointerClick>,
    mut splashes: MessageWriter<Splash>,
    mut presses: MessageWriter<MenuPressed>,
) {
    let MenuInput {
        screen,
        session,
        keys,
        controls,
        mut focus,
        mut settings,
        preferences,
        mut sounds,
        buttons,
        bars,
        pointer,
    } = input;
    let screen = *screen.get();
    let actions = MenuAction::of(screen);
    // A menu opens on its first button, except the difficulty, which opens on
    // the one already chosen.
    let opening = actions
        .iter()
        .position(|action| *action == MenuAction::ChooseDifficulty(session.difficulty))
        .unwrap_or(0);
    let Some(standing_on) = focus.settle(screen, actions.len(), opening) else {
        return;
    };
    // While the controls screen listens for a key to bind, every key belongs
    // to that, and the mouse must not walk the focus off the row being bound.
    if controls.rebinding.is_some() {
        return;
    }

    let aimed_at = |entity, at| buttons.get(entity).ok().map(|(_, button)| (*button, at));
    let pointed_at = pointer_overs
        .read()
        .filter_map(|over| aimed_at(over.entity, over.pointer.position))
        .last();
    let clicked = pointer_clicks
        .read()
        .filter(|click| click.button == PointerButton::Primary)
        .filter_map(|click| aimed_at(click.entity, click.pointer.position))
        .last();
    let middle_of = |index: usize| {
        buttons
            .iter()
            .find(|(_, button)| button.index == index)
            .map(|(entity, _)| Splash(Spot::Middle(entity)))
    };

    // Pointing at a button walks the focus straight to it, and a droplet falls
    // exactly where the mouse touched. Only a button the pointer newly reached
    // splashes, so resting on one does not splash over and over — a click
    // always does, even on the button already under the pointer.
    if let Some((button, at)) = pointed_at.filter(|(button, _)| button.index != standing_on) {
        focus.point_at(button.index);
        sounds.push(SoundCue::UiMove);
        if clicked.is_none() {
            splashes.write(Splash(Spot::Pointer(at)));
        }
    }
    if let Some((button, at)) = clicked {
        focus.point_at(button.index);
        splashes.write(Splash(Spot::Pointer(at)));
    }

    // The arrows walk the list, and the droplet falls in the middle of the
    // button they reach.
    let step = isize::from(controls.just_pressed(GameAction::NavigateDown, &keys))
        - isize::from(controls.just_pressed(GameAction::NavigateUp, &keys));
    if step != 0 {
        focus.walk(step, actions.len());
        sounds.push(SoundCue::UiMove);
        splashes.write_batch(middle_of(focus.index));
    }
    let standing_on = focus.index;

    // Left and right move the volume the focus is standing on.
    if let Some(MenuAction::Volume(channel)) = actions.get(standing_on) {
        let right = controls.just_pressed(GameAction::MoveRight, &keys);
        if right || controls.just_pressed(GameAction::MoveLeft, &keys) {
            settings.step_volume(*channel, right);
            settings::save_settings(&settings, &preferences);
            sounds.push(SoundCue::UiMove);
        }
    }

    // A click presses the button it landed on, and a click inside a volume bar
    // also says where along it to set that volume. The confirm key presses the
    // button the focus is standing on, which is what its badge promises.
    let pressed = if let Some((button, at)) = clicked {
        Some(MenuPressed {
            action: button.action,
            along_bar: match button.action {
                MenuAction::Volume(channel) => bars
                    .iter()
                    .find(|(bar, _, _)| bar.0 == channel)
                    .and_then(|(_, node, transform)| {
                        let width = node.size.x * node.inverse_scale_factor;
                        let middle = transform.translation.x * node.inverse_scale_factor;
                        let along = (pointer.at(at).x - middle) / width.max(f32::EPSILON) + 0.5;
                        (0.0..=1.0).contains(&along).then_some(along)
                    }),
                _ => None,
            },
        })
    } else if controls.just_pressed(GameAction::Interact, &keys)
        || keys.just_pressed(KeyCode::Enter)
    {
        splashes.write_batch(middle_of(standing_on));
        actions.get(standing_on).map(|action| MenuPressed {
            action: *action,
            along_bar: None,
        })
    } else {
        None
    };
    presses.write_batch(pressed);
}
