//! Room collision shared by the player, enemies, and projectiles. A room is
//! built from solid platforms: a body lands on their top, is stopped by their
//! sides, and bumps its head on their underside, and they stop shots. Every
//! body is a box `PLAYER_COLLISION_WIDTH_H` wide and 1 H tall, positioned by
//! the center of its feet.
//!
//! A prop whose sprite has `solid_px` boxes is solid there in the same way,
//! each box a platform standing square.
//!
//! A platform a room turns (`rotation_deg`) is solid the same way, as the
//! tilted rectangle it is drawn as. The gentler of its two upper faces is a
//! slope bodies walk when it is no steeper than `player.walkable_slope_deg`;
//! every other face stops a body like a side, so one that lands on a steep
//! face slides down it. A platform left square keeps the exact arithmetic it
//! always had.

use super::*;
use crate::data::RoomConfig;

/// Gap, in world units, within which a body counts as touching a platform.
const CONTACT_TOLERANCE: f32 = 1.0;

/// Half the width of a body's collision box, in world units.
pub(super) const HALF_BODY_WIDTH: f32 = PLAYER_COLLISION_WIDTH_H * H_WORLD_UNITS / 2.0;

#[derive(Component, Clone, Copy, Debug)]
pub(super) struct PlatformSurface {
    // The platform's bounds. They are its whole shape while it stands square.
    pub(super) left: f32,
    pub(super) right: f32,
    pub(super) bottom: f32,
    pub(super) top: f32,
    pub(super) footstep: FootstepSound,
    /// The shape of a platform its room turned; `None` for one standing square.
    pub(super) tilt: Option<Tilt>,
}

/// The shape of a platform that does not stand square.
#[derive(Clone, Copy, Debug)]
pub(super) struct Tilt {
    /// Its corners in world units, counter-clockwise.
    corners: [Vec2; 4],
    /// The upper face bodies walk, from its left end to its right, when the
    /// platform has one gentle enough.
    floor: Option<(Vec2, Vec2)>,
}

impl Tilt {
    /// The corners with the one that follows each: the platform's four faces.
    fn faces(&self) -> impl Iterator<Item = (Vec2, Vec2)> + '_ {
        (0..4).map(|index| (self.corners[index], self.corners[(index + 1) % 4]))
    }

    /// The reach, along the other axis, of the part of the platform lying
    /// between `low` and `high` on `axis` (0 for x, 1 for y).
    fn reach_within(&self, axis: usize, low: f32, high: f32) -> Option<(f32, f32)> {
        let other = 1 - axis;
        let mut reach: Option<(f32, f32)> = None;
        let mut include = |value: f32| {
            reach = Some(reach.map_or((value, value), |(min, max)| {
                (min.min(value), max.max(value))
            }));
        };
        for (from, to) in self.faces() {
            if (low..=high).contains(&from[axis]) {
                include(from[other]);
            }
            for limit in [low, high] {
                if (from[axis] - limit) * (to[axis] - limit) < 0.0 {
                    let along = (limit - from[axis]) / (to[axis] - from[axis]);
                    include(from[other] + along * (to[other] - from[other]));
                }
            }
        }
        reach
    }
}

impl PlatformSurface {
    /// Every collider of `room`, in world units: its platforms, then the
    /// solid boxes of its props.
    pub(super) fn of_room(room: &RoomConfig, data: &GameData) -> Vec<Self> {
        let props = room
            .props
            .iter()
            .flat_map(|placed| data.props.solid_rects_h(placed))
            .map(|rect| Self {
                left: rect.x * H_WORLD_UNITS,
                right: rect.right() * H_WORLD_UNITS,
                bottom: rect.y * H_WORLD_UNITS,
                top: rect.top() * H_WORLD_UNITS,
                footstep: data.surfaces.color_footstep,
                tilt: None,
            });
        room.platforms
            .iter()
            .map(|block| {
                let corners = block.corners_h().map(|corner| corner * H_WORLD_UNITS);
                let low = corners.into_iter().reduce(Vec2::min).unwrap_or_default();
                let high = corners.into_iter().reduce(Vec2::max).unwrap_or_default();
                // A quarter turn leaves the platform square: its bounds are its shape.
                let tilt = (block.rotation_deg % 90.0 != 0.0).then(|| {
                    let steepest = data.gameplay.player.walkable_slope_deg.to_radians().tan();
                    let rise =
                        |(from, to): &(Vec2, Vec2)| ((to.y - from.y) / (to.x - from.x)).abs();
                    // Going round counter-clockwise, the faces that look up are the
                    // ones running right to left.
                    let floor = (0..4)
                        .map(|index| (corners[(index + 1) % 4], corners[index]))
                        .filter(|(from, to)| from.x < to.x)
                        .min_by(|first, second| rise(first).total_cmp(&rise(second)))
                        .filter(|face| rise(face) <= steepest);
                    Tilt { corners, floor }
                });
                Self {
                    left: low.x,
                    right: high.x,
                    bottom: low.y,
                    top: high.y,
                    footstep: data.surfaces.footstep(&block.fill),
                    tilt,
                }
            })
            .chain(props)
            .collect()
    }

    /// The height the feet of a body centered at `x` stand at on this
    /// platform: its top, or the slope under whichever foot is further up
    /// it. `None` where nothing of it is there to stand on. Shared by physics
    /// and AI so they agree at ledges.
    fn floor_at(&self, x: f32) -> Option<f32> {
        let (left, right) = (x - HALF_BODY_WIDTH, x + HALF_BODY_WIDTH);
        match &self.tilt {
            None => (right > self.left && left < self.right).then_some(self.top),
            Some(tilt) => {
                let (from, to) = tilt.floor?;
                (right > from.x && left < to.x).then(|| {
                    let at = if to.y > from.y {
                        right.min(to.x)
                    } else {
                        left.max(from.x)
                    };
                    from.y + (to.y - from.y) * (at - from.x) / (to.x - from.x)
                })
            }
        }
    }

    /// How far a body is carried up or down this platform's slope by a step
    /// `stride` long: nothing on one that stands square.
    fn climb(&self, stride: f32) -> f32 {
        self.tilt
            .and_then(|tilt| tilt.floor)
            .map_or(0.0, |(from, to)| {
                stride * ((to.y - from.y) / (to.x - from.x)).abs()
            })
    }

    /// The lowest and highest this platform reaches over a body centered at
    /// `x`: the underside it bumps its head on and the top above it.
    fn reach_over(&self, x: f32) -> Option<(f32, f32)> {
        let (left, right) = (x - HALF_BODY_WIDTH, x + HALF_BODY_WIDTH);
        match &self.tilt {
            None => (right > self.left && left < self.right).then_some((self.bottom, self.top)),
            Some(tilt) => tilt.reach_within(0, left, right),
        }
    }

    /// The sides of this platform that stop a body whose feet are at
    /// `feet_y`, where it stands beside the platform instead of walking past.
    /// A body resting on the top, or passing just under the bottom, meets
    /// none.
    fn sides_beside(&self, feet_y: f32) -> Option<(f32, f32)> {
        let low = feet_y + CONTACT_TOLERANCE;
        let high = feet_y + H_WORLD_UNITS - CONTACT_TOLERANCE;
        match &self.tilt {
            None => (low < self.top && high > self.bottom).then_some((self.left, self.right)),
            Some(tilt) => tilt.reach_within(1, low, high),
        }
    }

    /// Whether this platform stands between `from` and `to`, hiding one from
    /// the other. Platforms are solid on every side, so they are the room's
    /// only cover, for sight as much as for shots.
    pub(super) fn blocks_sight(&self, from: Vec2, to: Vec2) -> bool {
        let Some(tilt) = &self.tilt else {
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
            return entry <= exit;
        };
        // The same clipping, against each face of the tilted shape in turn:
        // the segment is hidden when some stretch of it lies inside all four.
        let delta = to - from;
        let mut entry: f32 = 0.0;
        let mut exit: f32 = 1.0;
        for (start, end) in tilt.faces() {
            let inward = (end - start).perp();
            let depth = (from - start).dot(inward);
            let step = delta.dot(inward);
            if step == 0.0 {
                if depth < 0.0 {
                    return false;
                }
            } else if step > 0.0 {
                entry = entry.max(-depth / step);
            } else {
                exit = exit.min(-depth / step);
            }
        }
        entry <= exit
    }

    /// Whether a projectile sweeping from `start` to `end` runs into this
    /// platform.
    pub(super) fn intersects_projectile(&self, start: Vec2, end: Vec2, half_size: Vec2) -> bool {
        let swept = Rect::from_corners(start.min(end) - half_size, start.max(end) + half_size);
        swept.min.x <= self.right
            && swept.max.x >= self.left
            && swept.min.y <= self.top
            && swept.max.y >= self.bottom
            // Inside the bounds of a tilted platform, the shot still misses
            // when it lies wholly outside one of its faces.
            && self.tilt.is_none_or(|tilt| {
                tilt.faces().all(|(from, to)| {
                    let inward = (to - from).perp();
                    [
                        swept.min,
                        swept.max,
                        Vec2::new(swept.min.x, swept.max.y),
                        Vec2::new(swept.max.x, swept.min.y),
                    ]
                    .into_iter()
                    .any(|corner| (corner - from).dot(inward) >= 0.0)
                })
            })
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

    /// The highest floor a body at `x` came down onto while its feet fell
    /// from `previous_feet_y` to `feet_y`, and the height it stands at there.
    /// `stride` is how far the body moved sideways to get to `x`: a slope
    /// lifts it that far up its face, and one that was `grounded` is held
    /// down on a face falling away under it instead of being left in the air.
    pub(super) fn landing(
        &self,
        x: f32,
        previous_feet_y: f32,
        feet_y: f32,
        stride: f32,
        grounded: bool,
    ) -> Option<(&'a PlatformSurface, f32)> {
        self.0
            .iter()
            .copied()
            .filter_map(|platform| {
                let floor = platform.floor_at(x)?;
                let climb = platform.climb(stride);
                let held = if grounded { climb } else { 0.0 };
                (previous_feet_y >= floor - CONTACT_TOLERANCE - climb && feet_y <= floor + held)
                    .then_some((platform, floor))
            })
            .max_by(|first, second| first.1.total_cmp(&second.1))
    }

    /// The lowest platform underside a body at `x` crossed while its head rose
    /// from `previous_head_y` to `head_y`: where a jump stops short.
    pub(super) fn ceiling(&self, x: f32, previous_head_y: f32, head_y: f32) -> Option<f32> {
        self.0
            .iter()
            .filter_map(|platform| platform.reach_over(x))
            .map(|(underside, _)| underside)
            .filter(|underside| {
                previous_head_y <= underside + CONTACT_TOLERANCE && head_y > *underside
            })
            .min_by(f32::total_cmp)
    }

    /// Where a body with its feet at `feet_y` ends up after moving from
    /// `previous_x` to `x`: against the side of the first platform in its way,
    /// or at `x` when nothing blocks it. A body already inside a platform is
    /// free to walk out of it — unless the platform is tilted, whose faces
    /// lean over a body and so push out one they find inside them.
    pub(super) fn blocked_x(&self, previous_x: f32, x: f32, feet_y: f32) -> f32 {
        // A slope is walked rather than run into: the step is measured from
        // where its face will have carried the feet by the end of it.
        let stride = (x - previous_x).abs();
        let feet_y = self
            .0
            .iter()
            .filter_map(|platform| {
                platform
                    .floor_at(x)
                    .filter(|floor| floor - feet_y <= CONTACT_TOLERANCE + platform.climb(stride))
            })
            .fold(feet_y, f32::max);
        let mut blocked = x;
        for platform in &self.0 {
            let Some((left, right)) = platform.sides_beside(feet_y) else {
                continue;
            };
            if platform.tilt.is_some() {
                if blocked + HALF_BODY_WIDTH > left && blocked - HALF_BODY_WIDTH < right {
                    blocked = if previous_x < (left + right) / 2.0 {
                        left - HALF_BODY_WIDTH
                    } else {
                        right + HALF_BODY_WIDTH
                    };
                }
            } else if previous_x + HALF_BODY_WIDTH <= left && blocked + HALF_BODY_WIDTH > left {
                blocked = left - HALF_BODY_WIDTH;
            } else if previous_x - HALF_BODY_WIDTH >= right && blocked - HALF_BODY_WIDTH < right {
                blocked = right + HALF_BODY_WIDTH;
            }
        }
        blocked
    }

    /// Whether a platform's side stands against a body at `x` whose feet are
    /// at `feet_y`, on the side `direction` points to: the wall a body in
    /// mid-air clings to and pushes off.
    pub(super) fn wall_beside(&self, x: f32, feet_y: f32, direction: f32) -> bool {
        self.0
            .iter()
            .filter_map(|platform| platform.sides_beside(feet_y))
            .any(|(left, right)| {
                if direction < 0.0 {
                    (x - HALF_BODY_WIDTH - right).abs() <= CONTACT_TOLERANCE
                } else {
                    (left - x - HALF_BODY_WIDTH).abs() <= CONTACT_TOLERANCE
                }
            })
    }

    /// The highest floor under a body at `x` whose feet are at `feet_y`: where
    /// it would land if it stepped off what it stands on there.
    pub(super) fn floor_below(&self, x: f32, feet_y: f32) -> Option<f32> {
        self.0
            .iter()
            .filter_map(|platform| platform.floor_at(x))
            .filter(|floor| *floor <= feet_y + CONTACT_TOLERANCE)
            .max_by(f32::total_cmp)
    }

    /// The platform a grounded body at `x` stands on, with its feet at
    /// `feet_y`, and the height it stands at there. `stride` is how far the
    /// body may have stepped since its feet were put there: a slope has
    /// carried it that far up or down its face.
    pub(super) fn standing_on(
        &self,
        x: f32,
        feet_y: f32,
        stride: f32,
    ) -> Option<(&'a PlatformSurface, f32)> {
        self.0
            .iter()
            .copied()
            .filter_map(|platform| {
                let floor = platform.floor_at(x)?;
                ((floor - feet_y).abs() <= CONTACT_TOLERANCE + platform.climb(stride))
                    .then_some((platform, floor))
            })
            .max_by(|first, second| first.1.total_cmp(&second.1))
    }
}
