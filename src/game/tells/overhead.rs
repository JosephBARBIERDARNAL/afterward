//! What hangs over an enemy while a bonus has a hold on it, redrawn from its
//! state each frame:
//! - Marked Prey's reticle, with a bar emptying as the mark runs out;
//! - with Cold Read or Insight, a "!" over a wind-up, with a bar emptying
//!   until the blow comes;
//! - with Opportunist, a sleep over an enemy that has not noticed anyone;
//! - with Executioner, a line on the health bar where its bonus starts, and
//!   once the body is below it the bar flashing and a skull beside it (at
//!   50% on a marked body with Open Wound).
//!
//! The reticle is drawn white and tinted; the "!", the sleep and the skull
//! are the VFX packs' (`effects/vfx/`), played once and held or looped.

use super::*;
use crate::game::combat::DeadEnemy;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum OverheadKind {
    MarkIcon,
    MarkBar,
    WindupIcon,
    WindupBar,
    Unaware,
    ExecuteLine,
    ExecuteFlash,
    ExecuteSkull,
}

/// One piece hung over `enemy`. A bar keeps the fullest value it has shown,
/// which is what it empties from: a mark renewed or a fresh wind-up fills
/// it again.
#[derive(Component)]
pub(crate) struct Overhead {
    pub(crate) enemy: Entity,
    pub(crate) kind: OverheadKind,
    full: f32,
}

/// How a piece is drawn this frame.
struct Piece {
    kind: OverheadKind,
    /// The middle of the piece.
    at: Vec2,
    size: Vec2,
    color: Color,
    /// The icon's sheet, or none for a plain bar.
    sheet: Option<&'static str>,
    /// For a bar, what is left of it, emptying from its right.
    left: Option<f32>,
}

/// How wide a bar is drawn, in world units, and how many world units an
/// icon's pixel takes.
const BAR_WIDTH: f32 = 34.0;
/// The health bar's width and how far over the body it stands
/// (`feedback::update_health_bars`).
const HEALTH_BAR_WIDTH: f32 = 48.0;
const HEALTH_BAR_RISE: f32 = 6.0;

pub(crate) fn update_overheads(
    mut commands: Commands,
    data: Res<GameData>,
    session: Res<RunSession>,
    sheets: Res<SpriteSheets>,
    time: Res<Time>,
    enemies: Query<(Entity, &EnemyBody, &Transform), Without<DeadEnemy>>,
    mut overheads: Query<(Entity, &mut Overhead, &mut Transform, &mut Sprite), Without<EnemyBody>>,
) {
    let marks = session.has_bonus("marked_prey");
    let reads = session.has_bonus("cold_read") || session.has_bonus("insight");
    let ambushes = session.has_bonus("opportunist");
    let executes = session.has_bonus("executioner");
    let mut standing = overheads
        .iter()
        .map(|(entity, overhead, ..)| ((overhead.enemy, overhead.kind), entity))
        .collect::<BTreeMap<_, _>>();
    let pulse = 0.5 + 0.5 * (time.elapsed_secs() * 14.0).sin();
    for (enemy_entity, enemy, transform) in &enemies {
        if enemy.health <= 0.0 {
            continue;
        }
        let x = transform.translation.x;
        let health_bar_y = transform.translation.y + enemy.visible_top_offset + HEALTH_BAR_RISE;
        // Over the shield icons when there are any.
        let mut row_y = health_bar_y + if enemy.armor > 0 { 30.0 } else { 13.0 };
        let mut pieces = Vec::new();
        let icon_and_bar =
            |pieces: &mut Vec<Piece>, kinds, sheet: &'static str, size, color, left, y| {
                let (icon, bar) = kinds;
                pieces.push(Piece {
                    kind: icon,
                    at: Vec2::new(x - BAR_WIDTH / 2.0 - 4.0, y),
                    size,
                    // The packs' icons wear their own colours.
                    color: if sheet.starts_with("vfx_") {
                        Color::WHITE
                    } else {
                        color
                    },
                    sheet: Some(sheet),
                    left: None,
                });
                pieces.push(Piece {
                    kind: bar,
                    at: Vec2::new(x + 6.0, y),
                    size: Vec2::new(BAR_WIDTH, 3.0),
                    color,
                    sheet: None,
                    left: Some(left),
                });
            };
        if marks && enemy.marked_timer > 0.0 {
            icon_and_bar(
                &mut pieces,
                (OverheadKind::MarkIcon, OverheadKind::MarkBar),
                "tell_mark",
                Vec2::splat(20.0),
                Color::from(data.gameplay.effects.tells.insight_color),
                enemy.marked_timer,
                row_y,
            );
            row_y += 20.0;
        }
        if reads && enemy.windup_remaining > 0.0 {
            icon_and_bar(
                &mut pieces,
                (OverheadKind::WindupIcon, OverheadKind::WindupBar),
                "vfx_alert",
                Vec2::splat(24.0),
                Color::from(data.gameplay.effects.telegraph.color),
                enemy.windup_remaining,
                row_y,
            );
        }
        if ambushes && !enemy.engaged {
            // The frame is mostly air under its letters: raised to sit on
            // the row.
            pieces.push(Piece {
                kind: OverheadKind::Unaware,
                at: Vec2::new(x + 8.0, row_y + 12.0),
                size: Vec2::new(48.0, 64.0) * 0.9,
                color: Color::WHITE,
                sheet: Some("vfx_unaware"),
                left: None,
            });
        }
        if executes {
            let mut line = session.bonus_effect(&data, "executioner", "execute_health_fraction");
            if enemy.marked_timer > 0.0 && marks && session.has_bonus("open_wound") {
                line = line.max(session.bonus_effect(
                    &data,
                    "open_wound",
                    "marked_execute_health_fraction",
                ));
            }
            let left_edge = x - HEALTH_BAR_WIDTH / 2.0;
            pieces.push(Piece {
                kind: OverheadKind::ExecuteLine,
                at: Vec2::new(left_edge + HEALTH_BAR_WIDTH * line, health_bar_y),
                size: Vec2::new(2.0, 8.0),
                color: Color::WHITE,
                sheet: None,
                left: None,
            });
            let health = (enemy.health / enemy.max_health).clamp(0.0, 1.0);
            if health < line {
                pieces.push(Piece {
                    kind: OverheadKind::ExecuteFlash,
                    at: Vec2::new(left_edge + HEALTH_BAR_WIDTH * health / 2.0, health_bar_y),
                    size: Vec2::new(HEALTH_BAR_WIDTH * health.max(0.02), 4.0),
                    color: Color::WHITE.with_alpha(0.75 * pulse),
                    sheet: None,
                    left: None,
                });
                pieces.push(Piece {
                    kind: OverheadKind::ExecuteSkull,
                    at: Vec2::new(x + HEALTH_BAR_WIDTH / 2.0 + 13.0, health_bar_y + 2.0),
                    size: Vec2::splat(22.0),
                    color: Color::WHITE,
                    sheet: Some("vfx_skull"),
                    left: None,
                });
            }
        }

        for piece in pieces {
            let key = (enemy_entity, piece.kind);
            let Some(entity) = standing.remove(&key) else {
                let sheet = piece
                    .sheet
                    .and_then(|key| sheets.0.get(key).map(|sheet| (key, sheet)));
                let mut sprite = match sheet {
                    Some((_, sheet)) => Sprite::from_atlas_image(
                        sheet.image.clone(),
                        TextureAtlas {
                            layout: sheet.layout.clone(),
                            index: 0,
                        },
                    ),
                    None => Sprite::from_color(Color::WHITE, Vec2::ONE),
                };
                sprite.color = piece.color;
                sprite.custom_size = Some(piece.size);
                let mut spawned = commands.spawn((
                    sprite,
                    Transform::from_xyz(piece.at.x, piece.at.y, 9.0),
                    Overhead {
                        enemy: enemy_entity,
                        kind: piece.kind,
                        full: piece.left.unwrap_or(0.0),
                    },
                    RoomEntity,
                ));
                // An icon that moves pops in and holds, but the sleep, which
                // drifts for as long as it lasts.
                if let Some((key, sheet)) = sheet
                    && sheet.frames > 1
                {
                    spawned.insert(AnimatedSprite {
                        sheet: key.to_owned(),
                        frame: 0,
                        frames: sheet.frames,
                        timer: Timer::from_seconds(1.0 / sheet.fps, TimerMode::Repeating),
                        looping: key == "vfx_unaware",
                    });
                }
                continue;
            };
            let Ok((_, mut overhead, mut transform, mut sprite)) = overheads.get_mut(entity) else {
                continue;
            };
            sprite.color = piece.color;
            let (at, size) = match piece.left {
                Some(left) => {
                    overhead.full = overhead.full.max(left);
                    let fraction = (left / overhead.full.max(f32::EPSILON)).clamp(0.0, 1.0);
                    (
                        Vec2::new(
                            piece.at.x - piece.size.x * (1.0 - fraction) / 2.0,
                            piece.at.y,
                        ),
                        Vec2::new(piece.size.x * fraction, piece.size.y),
                    )
                }
                None => (piece.at, piece.size),
            };
            sprite.custom_size = Some(size);
            transform.translation = at.extend(9.0);
        }
    }
    for entity in standing.into_values() {
        commands.entity(entity).despawn();
    }
}
