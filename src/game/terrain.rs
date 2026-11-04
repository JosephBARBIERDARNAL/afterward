//! Room collision shared by the player, enemies, and projectiles. Solid
//! blocks stop bodies from every side; one-way platforms only hold up bodies
//! falling onto their top. Every body is a box `PLAYER_COLLISION_WIDTH_H` wide
//! and 1 H tall, positioned by the center of its feet.

use super::*;

/// Gap, in world units, within which a body counts as touching a wall.
const CONTACT_TOLERANCE: f32 = 1.0;
/// Overlap, in world units, ignored so bodies resting on or beside a block
/// are not treated as inside it.
const OVERLAP_TOLERANCE: f32 = 0.5;

#[derive(Component, Clone, Copy, Debug)]
pub(super) struct PlatformSurface {
    pub(super) left: f32,
    pub(super) right: f32,
    pub(super) bottom: f32,
    pub(super) top: f32,
    pub(super) material: PlatformMaterial,
    pub(super) one_way: bool,
}

impl PlatformSurface {
    /// Whether a body centered at `x` overlaps this block horizontally, so it
    /// can stand on it. Shared by physics and AI so they agree at ledges.
    pub(super) fn supports_body_at(&self, x: f32) -> bool {
        let half_width = PLAYER_COLLISION_WIDTH_H * H_WORLD_UNITS / 2.0;
        x + half_width > self.left && x - half_width < self.right
    }

    /// Whether this solid block overlaps a body vertically, given its feet.
    fn spans_body_height(&self, feet_y: f32) -> bool {
        !self.one_way
            && feet_y < self.top - OVERLAP_TOLERANCE
            && feet_y + H_WORLD_UNITS > self.bottom + OVERLAP_TOLERANCE
    }

    fn blocks_body(&self, x: f32, feet_y: f32) -> bool {
        let half_width = PLAYER_COLLISION_WIDTH_H * H_WORLD_UNITS / 2.0;
        self.spans_body_height(feet_y)
            && x + half_width > self.left + OVERLAP_TOLERANCE
            && x - half_width < self.right - OVERLAP_TOLERANCE
    }

    /// One-way platforms let projectiles through; solid blocks stop them.
    pub(super) fn intersects_projectile(&self, start: Vec2, end: Vec2, half_size: Vec2) -> bool {
        let projectile_left = start.x.min(end.x) - half_size.x;
        let projectile_right = start.x.max(end.x) + half_size.x;
        let projectile_bottom = start.y.min(end.y) - half_size.y;
        let projectile_top = start.y.max(end.y) + half_size.y;

        !self.one_way
            && projectile_left <= self.right
            && projectile_right >= self.left
            && projectile_bottom <= self.top
            && projectile_top >= self.bottom
    }
}

/// Every block of the current room, gathered once per system run.
pub(super) struct Terrain<'a>(pub(super) Vec<&'a PlatformSurface>);

impl<'a> Terrain<'a> {
    /// Whether a body at `x` with its feet at `feet_y` overlaps solid terrain.
    pub(super) fn blocks_body(&self, x: f32, feet_y: f32) -> bool {
        self.0.iter().any(|block| block.blocks_body(x, feet_y))
    }

    /// Moves a body horizontally from `from_x` to `to_x`, stopping it against
    /// the first solid side in the way. Returns the reached x and, when
    /// stopped, the side (-1 left, 1 right) of the wall it hit.
    pub(super) fn move_horizontally(
        &self,
        from_x: f32,
        to_x: f32,
        feet_y: f32,
    ) -> (f32, Option<f32>) {
        let half_width = PLAYER_COLLISION_WIDTH_H * H_WORLD_UNITS / 2.0;
        let mut reached = to_x;
        let mut wall = None;
        for block in self
            .0
            .iter()
            .filter(|block| block.spans_body_height(feet_y))
        {
            if to_x > from_x
                && block.left >= from_x + half_width - OVERLAP_TOLERANCE
                && block.left < reached + half_width
            {
                reached = block.left - half_width;
                wall = Some(1.0);
            } else if to_x < from_x
                && block.right <= from_x - half_width + OVERLAP_TOLERANCE
                && block.right > reached - half_width
            {
                reached = block.right + half_width;
                wall = Some(-1.0);
            }
        }
        (reached, wall)
    }

    /// The side (-1 left, 1 right) of a solid wall the body is touching.
    pub(super) fn wall_contact(&self, x: f32, feet_y: f32) -> Option<f32> {
        let half_width = PLAYER_COLLISION_WIDTH_H * H_WORLD_UNITS / 2.0;
        self.0
            .iter()
            .filter(|block| block.spans_body_height(feet_y))
            .find_map(|block| {
                if (block.left - (x + half_width)).abs() <= CONTACT_TOLERANCE {
                    Some(1.0)
                } else if (block.right - (x - half_width)).abs() <= CONTACT_TOLERANCE {
                    Some(-1.0)
                } else {
                    None
                }
            })
    }

    /// The highest block top a body at `x` crossed while its feet fell from
    /// `previous_feet_y` to `feet_y`. One-way platforms count unless the body
    /// is dropping through them.
    pub(super) fn landing(
        &self,
        x: f32,
        previous_feet_y: f32,
        feet_y: f32,
        through_one_way: bool,
    ) -> Option<&'a PlatformSurface> {
        self.0
            .iter()
            .copied()
            .filter(|block| {
                (!block.one_way || !through_one_way)
                    && block.supports_body_at(x)
                    && previous_feet_y >= block.top - CONTACT_TOLERANCE
                    && feet_y <= block.top
            })
            .max_by(|first, second| first.top.total_cmp(&second.top))
    }

    /// The block a grounded body at `x` stands on, with its feet at `feet_y`.
    pub(super) fn standing_on(
        &self,
        x: f32,
        feet_y: f32,
        through_one_way: bool,
    ) -> Option<&'a PlatformSurface> {
        self.0.iter().copied().find(|block| {
            (!block.one_way || !through_one_way)
                && block.supports_body_at(x)
                && (block.top - feet_y).abs() <= CONTACT_TOLERANCE
        })
    }

    /// The bottom of the lowest solid ceiling a rising body at `x` hit while
    /// its head rose from `previous_head_y` to `head_y`.
    pub(super) fn ceiling(&self, x: f32, previous_head_y: f32, head_y: f32) -> Option<f32> {
        self.0
            .iter()
            .filter(|block| {
                !block.one_way
                    && block.supports_body_at(x)
                    && previous_head_y <= block.bottom + CONTACT_TOLERANCE
                    && head_y > block.bottom
            })
            .map(|block| block.bottom)
            .min_by(f32::total_cmp)
    }

    /// Where an airborne body pressing against a wall on `side` ends up after
    /// mantling it: on top of the wall's ledge, just past its edge. The ledge
    /// must be above the feet and at most `reach` above the head, with room
    /// to climb straight up beside it and to stand on it.
    pub(super) fn ledge(&self, x: f32, feet_y: f32, side: f32, reach: f32) -> Option<Vec2> {
        let half_width = PLAYER_COLLISION_WIDTH_H * H_WORLD_UNITS / 2.0;
        let inset = half_width + 0.1 * H_WORLD_UNITS;
        self.0
            .iter()
            .filter(|block| {
                block.spans_body_height(feet_y)
                    && block.top > feet_y
                    && block.top <= feet_y + H_WORLD_UNITS + reach
            })
            .find_map(|block| {
                let target_x = if side > 0.0 {
                    if (block.left - (x + half_width)).abs() > CONTACT_TOLERANCE {
                        return None;
                    }
                    block.left + inset
                } else {
                    if (block.right - (x - half_width)).abs() > CONTACT_TOLERANCE {
                        return None;
                    }
                    block.right - inset
                };
                (!self.blocks_body(x, block.top) && !self.blocks_body(target_x, block.top))
                    .then_some(Vec2::new(target_x, block.top))
            })
    }

    /// Whether a body walking off the `side` edge of `block` drops freely
    /// instead of running into a wall beside it.
    pub(super) fn edge_is_open(&self, block: &PlatformSurface, side: f32) -> bool {
        let half_width = PLAYER_COLLISION_WIDTH_H * H_WORLD_UNITS / 2.0;
        let beyond_x = if side > 0.0 {
            block.right + half_width + CONTACT_TOLERANCE
        } else {
            block.left - half_width - CONTACT_TOLERANCE
        };
        !self.blocks_body(beyond_x, block.top)
    }
}
