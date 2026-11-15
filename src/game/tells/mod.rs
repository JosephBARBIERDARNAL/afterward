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

pub(super) use afterimages::{leave_afterimages, update_fading};
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
    /// Where that health was drawn from: an orb flies from there to the
    /// player.
    pub(crate) drawn_from: Option<Vec3>,
}

/// Where a tell is shown.
#[derive(Clone, Copy, Debug)]
pub(crate) enum TellAt {
    /// Over the player, wherever they stand when it is shown.
    Player,
    /// On that spot of the room: the middle of the body it concerns.
    Spot(Vec3),
}

/// A sheet of `effects/vfx/` (`scripts/cut_vfx.py`) played once where a
/// tell is shown.
#[derive(Clone, Copy, Debug)]
pub(crate) struct TellEffect {
    /// Its name in `effects/vfx/`, without the `vfx_` of its sheet's key.
    pub(crate) sheet: &'static str,
    /// How tall it is drawn, in H.
    pub(crate) size_h: f32,
    /// Where its middle stands above the spot, in H: the player's spot is
    /// the middle of their body, half an H over their feet.
    pub(crate) rise_h: f32,
    /// Drawn facing left rather than right.
    pub(crate) flip: bool,
}

impl TellEffect {
    /// `sheet`, `size_h` tall, its middle on the spot.
    pub(crate) fn new(sheet: &'static str, size_h: f32) -> Self {
        Self {
            sheet,
            size_h,
            rise_h: 0.0,
            flip: false,
        }
    }

    /// Standing on the floor under the player's spot.
    pub(crate) fn on_the_floor(mut self) -> Self {
        self.rise_h = self.size_h / 2.0 - 0.5;
        self
    }

    /// Lying flat at the player's feet: a ring or a pool seen edge on.
    pub(crate) fn at_the_feet(mut self) -> Self {
        self.rise_h = -0.5;
        self
    }

    pub(crate) fn flipped(mut self, flip: bool) -> Self {
        self.flip = flip;
        self
    }
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
            drawn_from: None,
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

    /// The health it gave back flies to the player from `from`.
    pub(crate) fn drawn_from(mut self, from: Vec3) -> Self {
        self.drawn_from = Some(from);
        self
    }
}

/// An orb of health flying from where it was drawn to the player.
#[derive(Component)]
pub(super) struct LifeOrb {
    from: Vec3,
    timer: Timer,
}

/// How long an orb takes to reach the player, and how tall it is in H.
const LIFE_ORB_S: f32 = 0.35;
const LIFE_ORB_H: f32 = 0.4;

/// Carries each orb to the player, faster as it closes in, and takes it
/// away as it arrives.
pub(super) fn fly_life_orbs(
    mut commands: Commands,
    time: Res<Time>,
    players: Query<&Transform, (With<PlayerBody>, Without<LifeOrb>)>,
    mut orbs: Query<(Entity, &mut Transform, &mut LifeOrb)>,
) {
    let Ok(player) = players.single() else {
        return;
    };
    for (entity, mut transform, mut orb) in &mut orbs {
        orb.timer.tick(time.delta());
        if orb.timer.is_finished() {
            commands.entity(entity).despawn();
            continue;
        }
        let progress = orb.timer.fraction();
        let to = player.translation.with_z(orb.from.z);
        // A small arc up on the way, so a row of them reads as a stream.
        let arc = Vec3::Y * (progress * (1.0 - progress)) * 1.2 * H_WORLD_UNITS;
        transform.translation = orb.from.lerp(to, progress * progress) + arc;
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
        if let Some(from) = tell.drawn_from
            && let Some(sheet) = sheets.0.get("vfx_life_orb")
        {
            let from = from.with_z(20.0);
            let mut orb = Sprite::from_atlas_image(
                sheet.image.clone(),
                TextureAtlas {
                    layout: sheet.layout.clone(),
                    index: 0,
                },
            );
            orb.custom_size = Some(Vec2::splat(LIFE_ORB_H * H_WORLD_UNITS));
            commands.spawn((
                orb,
                Transform::from_translation(from),
                AnimatedSprite {
                    sheet: "vfx_life_orb".into(),
                    frame: 0,
                    frames: sheet.frames,
                    timer: Timer::from_seconds(1.0 / sheet.fps, TimerMode::Repeating),
                    looping: true,
                },
                LifeOrb {
                    from,
                    timer: Timer::from_seconds(LIFE_ORB_S, TimerMode::Once),
                },
                RoomEntity,
            ));
        }
        if let Some(effect) = tell.effect {
            play_effect(
                &mut commands,
                &sheets,
                &format!("vfx_{}", effect.sheet),
                Vec3::new(at.x, at.y + effect.rise_h * H_WORLD_UNITS, 6.5),
                effect.size_h,
                effect.flip,
            );
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
