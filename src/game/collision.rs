//! Room collision shared by the player, enemies, and projectiles. A room is
//! built from solid platforms: a body lands on their top, is stopped by their
//! sides, and bumps its head on their underside, and they stop shots. Every
//! body is a box `PLAYER_COLLISION_WIDTH_H` wide and 1 H tall, positioned by
//! the center of its feet.

use super::*;
use crate::data::PlatformConfig;

/// Gap, in world units, within which a body counts as touching a platform.
const CONTACT_TOLERANCE: f32 = 1.0;

/// Half the width of a body's collision box, in world units.
pub(super) const HALF_BODY_WIDTH: f32 = PLAYER_COLLISION_WIDTH_H * H_WORLD_UNITS / 2.0;

#[derive(Component, Clone, Copy, Debug)]
pub(super) struct PlatformSurface {
    pub(super) left: f32,
    pub(super) right: f32,
    pub(super) bottom: f32,
    pub(super) top: f32,
    pub(super) footstep: FootstepSound,
}

impl PlatformSurface {
    /// The collider of an authored platform, in world units.
    pub(super) fn of(block: &PlatformConfig, data: &GameData) -> Self {
        let rect = block.rect_h;
        Self {
            left: rect.x * H_WORLD_UNITS,
            right: rect.right() * H_WORLD_UNITS,
            bottom: rect.y * H_WORLD_UNITS,
            top: rect.top() * H_WORLD_UNITS,
            footstep: data.surfaces.footstep(&block.fill),
        }
    }

    /// Whether a body centered at `x` overlaps this platform horizontally, so
    /// it can stand on its top or bump its head on its underside. Shared by
    /// physics and AI so they agree at ledges.
    pub(super) fn supports_body_at(&self, x: f32) -> bool {
        x + HALF_BODY_WIDTH > self.left && x - HALF_BODY_WIDTH < self.right
    }

    /// Whether a body whose feet are at `feet_y` stands beside this platform,
    /// where its sides stop the body instead of letting it walk past. A body
    /// resting on the top, or passing just under the bottom, does not.
    fn blocks_body_at(&self, feet_y: f32) -> bool {
        feet_y < self.top - CONTACT_TOLERANCE
            && feet_y + H_WORLD_UNITS > self.bottom + CONTACT_TOLERANCE
    }

    /// Whether this platform stands between `from` and `to`, hiding one from
    /// the other. Platforms are solid on every side, so they are the room's
    /// only cover, for sight as much as for shots.
    pub(super) fn blocks_sight(&self, from: Vec2, to: Vec2) -> bool {
        // Clip the segment against the platform's two slabs: it is hidden
        // behind the platform when some stretch of it lies inside both.
        let delta = to - from;
        let mut entry: f32 = 0.0;
        let mut exit: f32 = 1.0;
        for (origin, step, min, max) in [
            (from.x, delta.x, self.left, self.right),
            (from.y, delta.y, self.bottom, self.top),
        ] {
            if step == 0.0 {
                if origin < min || origin > max {
                    return false;
                }
                continue;
            }
            let (near, far) = ((min - origin) / step, (max - origin) / step);
            entry = entry.max(near.min(far));
            exit = exit.min(near.max(far));
        }
        entry <= exit
    }

    /// Whether a projectile sweeping from `start` to `end` runs into this
    /// platform.
    pub(super) fn intersects_projectile(&self, start: Vec2, end: Vec2, half_size: Vec2) -> bool {
        let projectile_left = start.x.min(end.x) - half_size.x;
        let projectile_right = start.x.max(end.x) + half_size.x;
        let projectile_bottom = start.y.min(end.y) - half_size.y;
        let projectile_top = start.y.max(end.y) + half_size.y;

        projectile_left <= self.right
            && projectile_right >= self.left
            && projectile_bottom <= self.top
            && projectile_top >= self.bottom
    }
}

/// Every platform of the current room, gathered once per system run.
pub(super) struct Platforms<'a>(pub(super) Vec<&'a PlatformSurface>);

impl<'a> Platforms<'a> {
    /// Whether any platform hides `to` from an eye at `from`.
    pub(super) fn blocks_sight(&self, from: Vec2, to: Vec2) -> bool {
        self.0
            .iter()
            .any(|platform| platform.blocks_sight(from, to))
    }

    /// The highest platform top a body at `x` crossed while its feet fell from
    /// `previous_feet_y` to `feet_y`.
    pub(super) fn landing(
        &self,
        x: f32,
        previous_feet_y: f32,
        feet_y: f32,
    ) -> Option<&'a PlatformSurface> {
        self.0
            .iter()
            .copied()
            .filter(|platform| {
                platform.supports_body_at(x)
                    && previous_feet_y >= platform.top - CONTACT_TOLERANCE
                    && feet_y <= platform.top
            })
            .max_by(|first, second| first.top.total_cmp(&second.top))
    }

    /// The lowest platform underside a body at `x` crossed while its head rose
    /// from `previous_head_y` to `head_y`: where a jump stops short.
    pub(super) fn ceiling(&self, x: f32, previous_head_y: f32, head_y: f32) -> Option<f32> {
        self.0
            .iter()
            .filter(|platform| {
                platform.supports_body_at(x)
                    && previous_head_y <= platform.bottom + CONTACT_TOLERANCE
                    && head_y > platform.bottom
            })
            .map(|platform| platform.bottom)
            .min_by(f32::total_cmp)
    }

    /// Where a body with its feet at `feet_y` ends up after moving from
    /// `previous_x` to `x`: against the side of the first platform in its way,
    /// or at `x` when nothing blocks it. A body already inside a platform is
    /// free to walk out of it.
    pub(super) fn blocked_x(&self, previous_x: f32, x: f32, feet_y: f32) -> f32 {
        let mut blocked = x;
        for platform in &self.0 {
            if !platform.blocks_body_at(feet_y) {
                continue;
            }
            if previous_x + HALF_BODY_WIDTH <= platform.left
                && blocked + HALF_BODY_WIDTH > platform.left
            {
                blocked = platform.left - HALF_BODY_WIDTH;
            } else if previous_x - HALF_BODY_WIDTH >= platform.right
                && blocked - HALF_BODY_WIDTH < platform.right
            {
                blocked = platform.right + HALF_BODY_WIDTH;
            }
        }
        blocked
    }

    /// Whether anything holds up a body at `x` whose feet are at `feet_y`: the
    /// platform it stands on, or one further down it would land on. Nothing
    /// means a bottomless gap, which a body falls out of the room through.
    pub(super) fn has_ground_below(&self, x: f32, feet_y: f32) -> bool {
        self.0.iter().any(|platform| {
            platform.supports_body_at(x) && platform.top <= feet_y + CONTACT_TOLERANCE
        })
    }

    /// The platform a grounded body at `x` stands on, with its feet at `feet_y`.
    pub(super) fn standing_on(&self, x: f32, feet_y: f32) -> Option<&'a PlatformSurface> {
        self.0.iter().copied().find(|platform| {
            platform.supports_body_at(x) && (platform.top - feet_y).abs() <= CONTACT_TOLERANCE
        })
    }
}
