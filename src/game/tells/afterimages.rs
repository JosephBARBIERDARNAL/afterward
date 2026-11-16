//! The fading copies a body leaves behind while a bonus speeds or throws it:
//! the chain thrown faster, a run faster than the legs alone allow, Meteor's
//! drop, a body Battering Ram threw or Undertow dragged. The middle of a dash,
//! where nothing reaches the player, leaves them too, for longer with
//! Slippery.

use super::*;
use bevy::sprite::Anchor;

/// A copy of a body left where it stood a moment ago.
#[derive(Component)]
pub(crate) struct Afterimage;

/// A sprite fading out from `alpha` over its timer, then taken away.
#[derive(Component)]
pub(crate) struct Fading {
    pub(crate) timer: Timer,
    pub(crate) alpha: f32,
}

pub(crate) fn update_fading(
    mut commands: Commands,
    time: Res<Time>,
    mut fading: Query<(Entity, &mut Sprite, &mut Fading)>,
) {
    for (entity, mut sprite, mut fade) in &mut fading {
        fade.timer.tick(time.delta());
        if fade.timer.is_finished() {
            commands.entity(entity).despawn();
            continue;
        }
        sprite
            .color
            .set_alpha(fade.alpha * fade.timer.fraction_remaining());
    }
}

type BodiesQuery<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static Sprite,
        Option<&'static Anchor>,
        &'static Transform,
        Option<&'static PlayerBody>,
        Option<&'static EnemyBody>,
    ),
    Or<(With<PlayerBody>, With<EnemyBody>)>,
>;

pub(crate) fn leave_afterimages(
    mut commands: Commands,
    time: Res<Time>,
    data: Res<GameData>,
    session: Res<RunSession>,
    bodies: BodiesQuery,
    // When each body last left a copy behind.
    mut clocks: Local<BTreeMap<Entity, f32>>,
) {
    let config = &data.gameplay.effects.tells;
    let tuning = &data.gameplay.player;
    let now = time.elapsed_secs();
    clocks.retain(|entity, _| bodies.contains(*entity));
    for (entity, sprite, anchor, transform, player, enemy) in &bodies {
        let tint = if let Some(player) = player {
            let dash_untouchable_to_s = tuning.dash.invulnerable_from_s
                + (tuning.dash.invulnerable_to_s - tuning.dash.invulnerable_from_s)
                    * session.bonus_multiplier(
                        &data,
                        "slippery",
                        "dash_invulnerability_multiplier",
                    );
            let untouchable = player.dash_s.is_some_and(|elapsed_s| {
                (tuning.dash.invulnerable_from_s..=dash_untouchable_to_s).contains(&elapsed_s)
            });
            let fast_chain = matches!(
                player.action,
                Some(PlayerAction::Strike { speed, .. }) if speed > 1.02
            );
            let fast_run = player.grounded
                && player.velocity.x.abs() > tuning.max_speed_h_s * H_WORLD_UNITS * 1.02;
            if untouchable {
                Some(Color::from(config.speed_color).mix(&Color::WHITE, 0.4))
            } else if player.slamming && session.has_bonus("meteor") {
                Some(Color::from(config.power_color))
            } else if fast_chain {
                Some(Color::from(config.boosted_color))
            } else if fast_run {
                Some(Color::from(config.speed_color))
            } else {
                None
            }
        } else {
            // A body thrown or dragged harder than a blow shoves it.
            enemy
                .filter(|enemy| {
                    enemy.health > 0.0
                        && enemy.knockback_velocity.abs()
                            > tuning.melee.knockback_h_s * H_WORLD_UNITS * 1.2
                        && ["battering_ram", "undertow", "riptide"]
                            .iter()
                            .any(|bonus| session.has_bonus(bonus))
                })
                .map(|_| Color::from(config.power_color))
        };
        let Some(tint) = tint else {
            continue;
        };
        if clocks
            .get(&entity)
            .is_some_and(|last| now - last < config.afterimage_every_s)
        {
            continue;
        }
        clocks.insert(entity, now);
        let mut copy = sprite.clone();
        copy.color = tint.with_alpha(config.afterimage_alpha);
        commands.spawn((
            copy,
            anchor.copied().unwrap_or_default(),
            Transform::from_translation(transform.translation - Vec3::Z * 0.2),
            Afterimage,
            Fading {
                timer: Timer::from_seconds(config.afterimage_s, TimerMode::Once),
                alpha: config.afterimage_alpha,
            },
            RoomEntity,
        ));
    }
}
