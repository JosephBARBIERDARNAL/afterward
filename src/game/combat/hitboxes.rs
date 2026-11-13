//! Hit areas in world units. Combat and the debug hitbox overlay both use
//! these, so the overlay draws exactly what hits.

use super::*;
use crate::data::{MeleeComboStep, RectH};

/// Where enemy strikes and projectiles hit the player whose body is centered
/// on `center` (half an H above the feet).
pub(crate) fn player_hurtbox(center: Vec3, data: &GameData) -> Rect {
    let size = data.gameplay.player.hurtbox_h;
    let feet = Vec2::new(center.x, center.y - 0.5 * H_WORLD_UNITS);
    Rect::from_corners(
        feet - Vec2::new(size.width / 2.0, 0.0) * H_WORLD_UNITS,
        feet + Vec2::new(size.width / 2.0, size.height) * H_WORLD_UNITS,
    )
}

impl EnemyBody {
    /// Where bullets hit this enemy, standing at `transform`.
    pub(crate) fn hurtbox(&self, transform: &Transform) -> Rect {
        let feet = Vec2::new(
            transform.translation.x,
            transform.translation.y - self.feet_offset,
        );
        Rect::from_corners(
            feet - Vec2::new(self.hurtbox_size.x / 2.0, 0.0),
            feet + Vec2::new(self.hurtbox_size.x / 2.0, self.hurtbox_size.y),
        )
    }
}

/// Where a hit area measured in H from a body's feet lands in the world. `x`
/// points the way `facing` does, `reach` stretches it, and a blow landing on
/// `both_sides` is mirrored around the feet instead of pointed. Every blow the
/// player throws — the chain's, the charged one, the slam — and the debug
/// overlay that draws them all come through here.
pub(crate) fn blow_area(
    hitbox: RectH,
    feet: Vec2,
    facing: f32,
    reach: f32,
    both_sides: bool,
) -> Rect {
    let (near, far) = if both_sides {
        let reach_h = (hitbox.x + hitbox.width) * reach;
        (-reach_h, reach_h)
    } else if facing < 0.0 {
        (-(hitbox.x + hitbox.width) * reach, -hitbox.x * reach)
    } else {
        (hitbox.x * reach, (hitbox.x + hitbox.width) * reach)
    };
    Rect::new(
        feet.x + near * H_WORLD_UNITS,
        feet.y + hitbox.y * H_WORLD_UNITS,
        feet.x + far * H_WORLD_UNITS,
        feet.y + (hitbox.y + hitbox.height) * H_WORLD_UNITS,
    )
}

impl MeleeComboStep {
    /// Where this blow of the player's chain lands, thrown from `feet`.
    pub(crate) fn hitbox_at(&self, feet: Vec2, facing: f32, reach: f32) -> Rect {
        blow_area(self.hitbox_h, feet, facing, reach, self.both_sides)
    }
}

impl EnemyAttack {
    /// Where this attack hurts, struck from `feet` toward `facing` (the sign
    /// of x); `None` for projectile attacks.
    pub(crate) fn hitbox_at(&self, feet: Vec2, facing: f32) -> Option<Rect> {
        self.hitbox.map(|hitbox| {
            let (left, right) = if facing < 0.0 {
                (-hitbox.max.x, -hitbox.min.x)
            } else {
                (hitbox.min.x, hitbox.max.x)
            };
            Rect::new(
                feet.x + left,
                feet.y + hitbox.min.y,
                feet.x + right,
                feet.y + hitbox.max.y,
            )
        })
    }

    /// Whether this attack, struck from `feet` toward `facing`, reaches `target`.
    pub(crate) fn hits(&self, feet: Vec2, facing: f32, target: Rect) -> bool {
        self.hitbox_at(feet, facing)
            .is_some_and(|hitbox| !hitbox.intersect(target).is_empty())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn attack_hitboxes_point_the_way_the_enemy_faces() {
        let attack = EnemyAttack {
            kind: String::from("melee"),
            damage: 1.0,
            hits: 1,
            reach_h: 1.0,
            hitbox: Some(Rect::new(-0.5, 0.0, 2.0, 1.0)),
            projectile_speed_h_s: 0.0,
            projectile_size: Vec2::ZERO,
            cooldown_s: 1.0,
            windup_s: 0.1,
            clip: None,
            strike_frame: None,
            guard: None,
            start_range_h: None,
            min_range_h: 0.0,
            shockwave: None,
            hazard: None,
        };
        let feet = Vec2::new(10.0, 5.0);
        assert_eq!(
            attack.hitbox_at(feet, 1.0),
            Some(Rect::new(9.5, 5.0, 12.0, 6.0))
        );
        assert_eq!(
            attack.hitbox_at(feet, -1.0),
            Some(Rect::new(8.0, 5.0, 10.5, 6.0))
        );
        // A target in front is hit; the same target behind is not.
        let ahead = Rect::new(11.5, 5.0, 13.0, 7.0);
        assert!(attack.hits(feet, 1.0, ahead));
        assert!(!attack.hits(feet, -1.0, ahead));
    }
}
