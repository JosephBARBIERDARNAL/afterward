//! What the Interact key acts on. The bonus item, the unlocked door, buttons
//! and terminals carry `Interactable`; `mark_what_is_in_reach` names the
//! nearest one the player stands within reach of, the systems owning those
//! things act on it when Interact is pressed, and `show_the_interact_prompt`
//! names the key, and what it does, at the bottom of the screen.

use super::*;
use crate::game::ui_style::{DESCRIPTION_COLOR, key_badge};
use bevy::text::{FontSize, FontWeight};

/// Something used with Interact rather than by walking into it.
#[derive(Component)]
pub(super) struct Interactable {
    /// How far from its middle the player may stand, on each axis, in H.
    pub(super) reach_h: Vec2,
    /// What Interact does to it, written beside the key ("take", "read").
    pub(super) verb: &'static str,
}

/// The `Interactable` the player stands within reach of, the nearest when
/// there are several.
#[derive(Resource, Default)]
pub(super) struct InReach(pub(super) Option<Entity>);

/// The prompt at the bottom of the screen, shown while something is in reach.
#[derive(Component)]
pub(super) struct InteractPrompt;

/// The name of the key Interact is bound to, in the prompt's badge.
#[derive(Component)]
pub(super) struct InteractKey;

/// What Interact does to the thing in reach, beside the key.
#[derive(Component)]
pub(super) struct InteractVerb;

pub(super) fn mark_what_is_in_reach(
    session: Res<RunSession>,
    mut in_reach: ResMut<InReach>,
    player: Query<&Transform, With<PlayerBody>>,
    things: Query<(Entity, &Transform, &Interactable, Has<ExitDoor>)>,
) {
    let nearest = player.single().ok().and_then(|player| {
        things
            .iter()
            // A locked door is not something to use yet.
            .filter(|(.., is_door)| !is_door || session.door_open)
            .filter_map(|(entity, transform, thing, _)| {
                let offset = (transform.translation - player.translation).truncate();
                let reach = thing.reach_h * H_WORLD_UNITS;
                (offset.x.abs() <= reach.x && offset.y.abs() <= reach.y)
                    .then(|| (entity, offset.length_squared()))
            })
            .min_by(|(_, first), (_, second)| first.total_cmp(second))
            .map(|(entity, _)| entity)
    });
    // Written only when it changes, so nothing watching it wakes for nothing.
    if in_reach.0 != nearest {
        in_reach.0 = nearest;
    }
}

/// Lays the prompt out once, hidden, at the bottom middle of the screen.
pub(super) fn spawn_interact_prompt(mut commands: Commands, data: Res<GameData>) {
    let config = &data.gameplay.room_flow.interact;
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                right: px(0),
                bottom: px(config.prompt_margin_px),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                column_gap: px(config.prompt_key_px / 3.0),
                ..default()
            },
            Visibility::Hidden,
            Pickable::IGNORE,
            InteractPrompt,
        ))
        .with_children(|prompt| {
            prompt.spawn(key_badge(config.prompt_key_px)).with_child((
                Text::new(""),
                TextFont {
                    font_size: FontSize::Px(config.prompt_key_px * 0.5),
                    weight: FontWeight::BOLD,
                    ..default()
                },
                TextColor(Color::WHITE),
                Pickable::IGNORE,
                InteractKey,
            ));
            prompt.spawn((
                Text::new(""),
                TextFont {
                    font_size: FontSize::Px(config.prompt_key_px * 0.5),
                    ..default()
                },
                TextColor(DESCRIPTION_COLOR),
                Pickable::IGNORE,
                InteractVerb,
            ));
        });
}

/// Shows the prompt while something stands in reach during play, naming the
/// key Interact is bound to (so a rebind shows) and what it does there.
pub(super) fn show_the_interact_prompt(
    screen: Res<State<GameScreen>>,
    controls: Res<ControlBindings>,
    in_reach: Res<InReach>,
    things: Query<&Interactable>,
    mut prompts: Query<&mut Visibility, With<InteractPrompt>>,
    mut keys: Query<&mut Text, With<InteractKey>>,
    mut verbs: Query<&mut Text, (With<InteractVerb>, Without<InteractKey>)>,
) {
    let thing = in_reach
        .0
        .filter(|_| *screen.get() == GameScreen::Playing)
        .and_then(|target| things.get(target).ok());
    for mut visibility in &mut prompts {
        visibility.set_if_neq(if thing.is_some() {
            Visibility::Visible
        } else {
            Visibility::Hidden
        });
    }
    let Some(thing) = thing else {
        return;
    };
    let label = controls.primary_label(GameAction::Interact);
    for mut text in &mut keys {
        if text.0 != label {
            text.0.clone_from(&label);
        }
    }
    for mut text in &mut verbs {
        if text.0 != thing.verb {
            text.0 = thing.verb.to_string();
        }
    }
}
