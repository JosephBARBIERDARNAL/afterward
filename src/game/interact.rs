//! What the Interact key acts on. The bonus item and the unlocked door carry
//! `Interactable`; `mark_what_is_in_reach` names the nearest one the player
//! stands within reach of, the systems owning those things act on it when
//! Interact is pressed, and `show_the_interact_badge` stands the key over it.

use super::*;
use crate::game::ui_style::ACTIVE_BORDER;
use bevy::text::{FontSize, FontWeight};

/// The inside of the badge: dark, so the key reads over any room.
const BADGE_FILL: Color = Color::srgba(0.05, 0.05, 0.1, 0.9);
/// How thick the badge's edge is, as a share of its height.
const BADGE_EDGE: f32 = 0.1;
/// How tall the key's name is, as a share of the badge's height.
const BADGE_TEXT: f32 = 0.6;
/// In front of the bodies standing in the room.
const BADGE_Z: f32 = 10.0;

/// Something used with Interact rather than by walking into it.
#[derive(Component)]
pub(super) struct Interactable {
    /// How far from its middle the player may stand, on each axis, in H.
    pub(super) reach_h: Vec2,
    /// How far its top stands above its middle, in H: where the badge goes.
    pub(super) top_h: f32,
}

/// The `Interactable` the player stands within reach of, the nearest when
/// there are several.
#[derive(Resource, Default)]
pub(super) struct InReach(pub(super) Option<Entity>);

/// The key standing over the thing in reach, as its child.
#[derive(Component)]
pub(super) struct InteractBadge {
    age_s: f32,
    /// Where it hovers around, above its parent's middle, in world units.
    rest_y: f32,
}

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

/// Stands the badge over the thing in reach, hovers it, and takes it away
/// once that thing is out of reach or gone.
pub(super) fn show_the_interact_badge(
    mut commands: Commands,
    time: Res<Time>,
    data: Res<GameData>,
    controls: Res<ControlBindings>,
    in_reach: Res<InReach>,
    things: Query<&Interactable>,
    mut badges: Query<(Entity, &ChildOf, &mut Transform, &mut InteractBadge)>,
) {
    let config = &data.gameplay.room_flow.interact;
    let mut standing = false;
    for (entity, parent, mut transform, mut badge) in &mut badges {
        if Some(parent.parent()) != in_reach.0 {
            commands.entity(entity).despawn();
            continue;
        }
        standing = true;
        badge.age_s += time.delta_secs();
        let turn = std::f32::consts::TAU * badge.age_s / config.bob_period_s;
        transform.translation.y = badge.rest_y + turn.sin() * config.bob_height_h * H_WORLD_UNITS;
        transform.scale = Vec3::splat((badge.age_s / config.pop_s).min(1.0));
    }
    if standing {
        return;
    }
    let Some((target, thing)) = in_reach
        .0
        .and_then(|target| Some((target, things.get(target).ok()?)))
    else {
        return;
    };
    let label = controls.primary_label(GameAction::Interact);
    let height = config.badge_size_h * H_WORLD_UNITS;
    // Square for a single letter, and wider for a name such as ENTER.
    let width = height * (1.0 + 0.5 * label.chars().count().saturating_sub(1) as f32);
    let edge = height * BADGE_EDGE;
    let rest_y = (thing.top_h + config.badge_gap_h) * H_WORLD_UNITS + height / 2.0;
    commands.entity(target).with_child((
        Sprite::from_color(ACTIVE_BORDER, Vec2::new(width, height)),
        Transform::from_xyz(0.0, rest_y, BADGE_Z).with_scale(Vec3::ZERO),
        InteractBadge { age_s: 0.0, rest_y },
        children![
            (
                Sprite::from_color(
                    BADGE_FILL,
                    Vec2::new(width - 2.0 * edge, height - 2.0 * edge)
                ),
                Transform::from_xyz(0.0, 0.0, 0.1),
            ),
            (
                Text2d::new(label),
                TextFont {
                    font_size: FontSize::Px(height * BADGE_TEXT),
                    weight: FontWeight::BOLD,
                    ..default()
                },
                TextColor(Color::WHITE),
                Transform::from_xyz(0.0, 0.0, 0.2),
            ),
        ],
    ));
}
