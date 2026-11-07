//! Room collision shared by the player, enemies, and projectiles. A room is
//! built from one-way platforms: they hold up a body falling onto their top
//! and let it through from every other side, but they stop shots. Every body
//! is a box `PLAYER_COLLISION_WIDTH_H` wide and 1 H tall, positioned by the
//! center of its feet.

use super::*;

/// Gap, in world units, within which a body counts as standing on a platform.
const CONTACT_TOLERANCE: f32 = 1.0;

#[derive(Component, Clone, Copy, Debug)]
pub(super) struct PlatformSurface {
    pub(super) left: f32,
    pub(super) right: f32,
    pub(super) bottom: f32,
    pub(super) top: f32,
    pub(super) footstep: FootstepSound,
}

impl PlatformSurface {
    /// Whether a body centered at `x` overlaps this platform horizontally, so
    /// it can stand on it. Shared by physics and AI so they agree at ledges.
    pub(super) fn supports_body_at(&self, x: f32) -> bool {
        let half_width = PLAYER_COLLISION_WIDTH_H * H_WORLD_UNITS / 2.0;
        x + half_width > self.left && x - half_width < self.right
    }

    /// Whether a projectile sweeping from `start` to `end` runs into this
    /// platform. Bodies pass through platforms; shots do not.
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
    /// The highest platform top a body at `x` crossed while its feet fell from
    /// `previous_feet_y` to `feet_y`, unless it is dropping through them.
    pub(super) fn landing(
        &self,
        x: f32,
        previous_feet_y: f32,
        feet_y: f32,
        dropping_through: bool,
    ) -> Option<&'a PlatformSurface> {
        if dropping_through {
            return None;
        }
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

    /// The highest platform top a body at `x` would land on if it dropped
    /// through the one it stands on; `None` when there is nothing below, so
    /// dropping would take it out of the room.
    pub(super) fn floor_below(&self, x: f32, feet_y: f32) -> Option<f32> {
        self.0
            .iter()
            .filter(|platform| {
                platform.supports_body_at(x) && platform.top < feet_y - CONTACT_TOLERANCE
            })
            .map(|platform| platform.top)
            .max_by(f32::total_cmp)
    }

    /// The platform a grounded body at `x` stands on, with its feet at `feet_y`.
    pub(super) fn standing_on(
        &self,
        x: f32,
        feet_y: f32,
        dropping_through: bool,
    ) -> Option<&'a PlatformSurface> {
        if dropping_through {
            return None;
        }
        self.0.iter().copied().find(|platform| {
            platform.supports_body_at(x) && (platform.top - feet_y).abs() <= CONTACT_TOLERANCE
        })
    }
}
