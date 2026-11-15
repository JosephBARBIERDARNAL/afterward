//! How a bonus shows it is at work, so that owning one is felt as well as
//! read on the bonus screen. Gameplay code writes a `BonusTell` where a bonus
//! does its thing; `show_bonus_tells` calls the bonus out by name in its
//! category's colour and plays what goes with it. What holds over time is
//! drawn from the run's state each frame instead: the marks and wind-ups over
//! the enemies (`overhead.rs`), the glow and the fading copies of the player
//! (`aura.rs`, `afterimages.rs`), and the row of what is in force under the
//! health bar (`tray.rs`). Tuned from `effects.tells` of `config/gameplay.yaml`.

use super::*;
use crate::game::combat::{Number, play_effect, spawn_damage_number};
use bevy::text::FontSize;

mod afterimages;
mod aura;
mod overhead;
mod tray;

pub(super) use afterimages::{Fading, leave_afterimages, update_fading};
pub(super) use aura::{show_the_combo_window, wear_the_aura};
pub(super) use overhead::update_overheads;
#[cfg(test)]
pub(super) use overhead::{Overhead, OverheadKind};
pub(super) use tray::{spawn_the_tray, update_the_tray};

/// A bonus has just done its thing.
#[derive(Message, Clone, Debug)]
pub(crate) struct BonusTell {
    /// The bonus's id in `config/bonuses.yaml`: its name is what is called
    /// out, its category's colour what everything is tinted.
    pub(crate) bonus: String,
    pub(crate) at: TellAt,
    /// Whether its name is called out; a bonus that fires on every jump only
    /// plays its effect.
    pub(crate) callout: bool,
    pub(crate) effect: Option<TellEffect>,
    /// Health it gave back, written over the player.
    pub(crate) healed: Option<f32>,
}

/// Where a tell is shown.
#[derive(Clone, Copy, Debug)]
pub(crate) enum TellAt {
    /// Over the player, wherever they stand when it is shown.
    Player,
    /// On that spot of the room: the middle of the body it concerns.
    Spot(Vec3),
}

/// What is played where a tell is shown, tinted in the bonus's colour.
#[derive(Clone, Copy, Debug)]
pub(crate) enum TellEffect {
    /// A ring opening out to that many H across.
    Ring(f32),
    /// A flat ring under the feet, that many H across: air pushed off.
    AirRing(f32),
    /// The burst of a heavy blow, that many H tall, untinted.
    Burst(f32),
    /// A puff of dust kicked up from under the feet, that many H tall.
    Dust(f32),
}

impl BonusTell {
    /// The bonus's name called out at `at`.
    pub(crate) fn new(bonus: impl Into<String>, at: TellAt) -> Self {
        Self {
            bonus: bonus.into(),
            at,
            callout: true,
            effect: None,
            healed: None,
        }
    }

    /// Plays only its effect, without calling the bonus out.
    pub(crate) fn quiet(mut self) -> Self {
        self.callout = false;
        self
    }

    pub(crate) fn with(mut self, effect: TellEffect) -> Self {
        self.effect = Some(effect);
        self
    }

    pub(crate) fn healing(mut self, healed: f32) -> Self {
        self.healed = Some(healed);
        self
    }
}

/// A bonus's name rising over the spot it fired on and fading out.
#[derive(Component)]
pub(super) struct Callout {
    timer: Timer,
    color: Color,
}

pub(super) fn show_bonus_tells(
    mut commands: Commands,
    mut tells: MessageReader<BonusTell>,
    data: Res<GameData>,
    sheets: Res<SpriteSheets>,
    time: Res<Time<Real>>,
    players: Query<&Transform, With<PlayerBody>>,
    // When each bonus was last called out, so one firing on every blow is
    // not called out on every blow.
    mut called: Local<BTreeMap<String, f32>>,
) {
    let config = &data.gameplay.effects.tells;
    let now = time.elapsed_secs();
    // Callouts shown together stand one above the other.
    let mut stacked = 0.0;
    for tell in tells.read() {
        let Some(bonus) = data
            .bonuses
            .bonuses
            .iter()
            .find(|bonus| bonus.id == tell.bonus)
        else {
            continue;
        };
        let color = data
            .bonuses
            .categories
            .get(&bonus.category)
            .map_or(Color::WHITE, |category| Color::from(category.color));
        let at = match tell.at {
            TellAt::Player => match players.single() {
                Ok(transform) => transform.translation,
                Err(_) => continue,
            },
            TellAt::Spot(at) => at,
        };
        if let Some(healed) = tell.healed {
            spawn_damage_number(&mut commands, config, at, Number::Healed(healed));
        }
        if let Some(effect) = tell.effect {
            let (key, size_h, tinted, z) = match effect {
                TellEffect::Ring(size_h) => ("effect_tell_ring", size_h, true, 6.5),
                TellEffect::AirRing(width_h) => (
                    "effect_air_ring",
                    // The sheet is 32 x 12: drawn `width_h` across.
                    width_h * 12.0 / 32.0,
                    true,
                    3.8,
                ),
                TellEffect::Burst(size_h) => ("effect_hit_burst", size_h, false, 6.5),
                TellEffect::Dust(size_h) => ("effect_dust", size_h, false, 3.8),
            };
            let feet = match effect {
                TellEffect::AirRing(_) => -0.5 * H_WORLD_UNITS,
                TellEffect::Dust(size_h) => (size_h / 2.0 - 0.5) * H_WORLD_UNITS,
                _ => 0.0,
            };
            if let Some(played) = play_effect(
                &mut commands,
                &sheets,
                key,
                Vec3::new(at.x, at.y + feet, z),
                size_h,
                false,
            ) && tinted
            {
                commands
                    .entity(played)
                    .entry::<Sprite>()
                    .and_modify(move |mut sprite| sprite.color = color);
            }
        }
        if !tell.callout
            || called
                .get(&tell.bonus)
                .is_some_and(|last| now - last < config.callout_repeat_s)
        {
            continue;
        }
        called.insert(tell.bonus.clone(), now);
        commands.spawn((
            Text2d::new(bonus.name.to_uppercase()),
            TextFont {
                font_size: FontSize::Px(config.callout_px),
                ..default()
            },
            TextColor(color),
            FontRole::BoldItalic,
            Transform::from_translation(Vec3::new(
                at.x,
                at.y + (1.15 + stacked) * H_WORLD_UNITS,
                21.0,
            )),
            Callout {
                timer: Timer::from_seconds(config.callout_s, TimerMode::Once),
                color,
            },
            RoomEntity,
        ));
        stacked += 0.4;
    }
}

/// Lifts each callout and fades it out, after a pop as it appears.
pub(super) fn update_callouts(
    mut commands: Commands,
    time: Res<Time>,
    data: Res<GameData>,
    mut callouts: Query<(Entity, &mut Transform, &mut TextColor, &mut Callout)>,
) {
    let rise = data.gameplay.effects.tells.callout_rise_h_s * H_WORLD_UNITS;
    for (entity, mut transform, mut color, mut callout) in &mut callouts {
        callout.timer.tick(time.delta());
        if callout.timer.is_finished() {
            commands.entity(entity).despawn();
            continue;
        }
        let progress = callout.timer.fraction();
        transform.translation.y += rise * time.delta_secs();
        transform.scale = Vec3::splat(1.0 + 0.45 * (1.0 - progress / 0.15).max(0.0));
        color.0 = callout
            .color
            .with_alpha((1.0 - (progress - 0.6).max(0.0) / 0.4).clamp(0.0, 1.0));
    }
}
