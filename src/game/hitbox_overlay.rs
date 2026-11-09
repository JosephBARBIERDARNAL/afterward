//! Debug hitbox overlay (debug builds only): B draws every hit area in play,
//! computed by the same code combat uses. Sizes live in `gameplay.yaml`
//! (`player.hurtbox_h` and the `player.melee` steps) and
//! `enemies/enemy_types.yaml`.

use super::*;
use crate::game::{
    combat::{CombatProjectile, melee_blow_area, player_hurtbox},
    player::Thrown,
};

const PLAYER_HURTBOX_COLOR: Color = Color::srgb(0.3, 1.0, 0.45);
const ENEMY_HURTBOX_COLOR: Color = Color::srgb(1.0, 0.9, 0.2);
/// An attack winding up or landing.
const STRIKE_COLOR: Color = Color::srgb(1.0, 0.2, 0.3);
/// The attack an enemy will use next, where it would land.
const NEXT_STRIKE_COLOR: Color = Color::srgba(1.0, 0.2, 0.3, 0.35);
const PROJECTILE_COLOR: Color = Color::srgb(1.0, 0.55, 0.15);

#[derive(Resource, Default)]
pub(super) struct HitboxOverlay {
    pub(super) visible: bool,
}

pub(super) fn toggle_hitbox_overlay(
    keys: Res<ButtonInput<KeyCode>>,
    mut overlay: ResMut<HitboxOverlay>,
) {
    if keys.just_pressed(KeyCode::KeyB) {
        overlay.visible = !overlay.visible;
    }
}

pub(super) fn draw_hitbox_overlay(
    mut gizmos: Gizmos,
    overlay: Res<HitboxOverlay>,
    data: Res<GameData>,
    session: Res<RunSession>,
    players: Query<(&PlayerBody, &Transform)>,
    enemies: Query<(&EnemyBody, &Transform, &Sprite)>,
    projectiles: Query<(&Transform, &CombatProjectile)>,
) {
    if !overlay.visible {
        return;
    }
    let mut outline = |rect: Rect, color: Color| {
        gizmos.rect_2d(rect.center(), rect.size(), color);
    };
    let melee = &data.gameplay.player.melee;
    for (player, transform) in &players {
        outline(
            player_hurtbox(transform.translation, &data),
            PLAYER_HURTBOX_COLOR,
        );
        // The blow in the air, or faintly the one the next press would
        // throw, drawn where it would land. The chain is the run's own, so a
        // finisher bonus shows up here as the last blow it reaches.
        let feet = Vec2::new(
            transform.translation.x,
            transform.translation.y - H_WORLD_UNITS / 2.0,
        );
        let chain = melee.chain(&|bonus| session.has_bonus(bonus));
        let pointed =
            |hitbox, both_sides| melee_blow_area(hitbox, feet, player.facing, 1.0, both_sides);
        let (area, color) = match player.strike.map(|strike| strike.thrown) {
            Some(Thrown::Step(index)) => (
                chain
                    .get(index)
                    .map(|step| pointed(step.hitbox_h, step.both_sides)),
                STRIKE_COLOR,
            ),
            Some(Thrown::Charged) => (Some(pointed(melee.charge.hitbox_h, false)), STRIKE_COLOR),
            None if player.slamming => (Some(pointed(melee.slam.hitbox_h, true)), STRIKE_COLOR),
            None => (
                chain
                    .get(player.combo_step)
                    .map(|step| pointed(step.hitbox_h, step.both_sides)),
                NEXT_STRIKE_COLOR,
            ),
        };
        if let Some(area) = area {
            outline(area, color);
        }
    }
    for (enemy, transform, sprite) in &enemies {
        if enemy.health <= 0.0 {
            continue;
        }
        outline(enemy.hurtbox(transform), ENEMY_HURTBOX_COLOR);
        // Enemies face the player: their sprite flips to look left.
        let facing = if sprite.flip_x { -1.0 } else { 1.0 };
        let feet = Vec2::new(transform.translation.x, enemy.home_y - enemy.feet_offset);
        let striking = enemy.windup_remaining > 0.0 || enemy.jump.is_some_and(|jump| jump.slam);
        let (attack, color) = if striking {
            (enemy.attacks.get(enemy.active_attack), STRIKE_COLOR)
        } else {
            (enemy.attacks.get(enemy.next_attack), NEXT_STRIKE_COLOR)
        };
        if let Some(hitbox) = attack.and_then(|attack| attack.hitbox_at(feet, facing)) {
            outline(hitbox, color);
        }
    }
    for (transform, projectile) in &projectiles {
        outline(
            Rect::from_center_half_size(transform.translation.truncate(), projectile.half_size),
            PROJECTILE_COLOR,
        );
    }
}
