use super::*;

/// Launch velocities, in world units per second, for a ballistic jump.
pub(super) struct JumpArc {
    pub vertical_velocity: f32,
    pub horizontal_velocity: f32,
}

impl JumpArc {
    /// Carries feet from `from` to `to`, peaking `apex` above `from` (raised
    /// when needed so the arc always clears the target height).
    pub(super) fn between(from: Vec2, to: Vec2, apex: f32, gravity: f32) -> Self {
        let apex = apex.max(to.y - from.y).max(0.0);
        let vertical_velocity = (2.0 * gravity * apex).sqrt();
        let flight_time = (vertical_velocity / gravity
            + (2.0 * (from.y + apex - to.y) / gravity).sqrt())
        .max(f32::EPSILON);
        Self {
            vertical_velocity,
            horizontal_velocity: (to.x - from.x) / flight_time,
        }
    }
}

/// How a grounded enemy gets to the player's level from where it stands.
pub(super) enum JumpPlan {
    Leap(JumpArc),
    /// Walk to this take-off x on the current platform, then leap from there.
    WalkTo(f32),
}

/// What a grounded enemy knows when deciding whether a jump brings it to the
/// player's level. Positions are feet positions in world units.
pub(super) struct JumpRoute {
    pub feet: Vec2,
    pub target_feet: Vec2,
    pub direction: f32,
    pub jump_height: f32,
    pub air_speed: f32,
    pub gravity: f32,
    pub level_change_threshold: f32,
}

impl JumpRoute {
    /// The best route from `standing`: up onto the platform closest to the
    /// player that is no higher than them when the player stands higher, or
    /// across the gap at the edge the enemy walks toward when the player is not
    /// below. Dropping down needs no jump: enemies walk off the ledge instead.
    pub(super) fn plan(&self, standing: &PlatformSurface, terrain: &Terrain) -> Option<JumpPlan> {
        let half_width = PLAYER_COLLISION_WIDTH_H * H_WORLD_UNITS / 2.0;
        let inset = half_width + 0.1 * H_WORLD_UNITS;
        let rise_to_target = self.target_feet.y - self.feet.y;
        let climbing = rise_to_target > self.level_change_threshold;
        let edge = if self.direction > 0.0 {
            standing.right
        } else {
            standing.left
        };
        let at_edge = (edge - self.feet.x) * self.direction <= inset + 0.15 * H_WORLD_UNITS;
        if !climbing && (!at_edge || rise_to_target < -self.level_change_threshold) {
            return None;
        }
        let reachable = |from: Vec2, landing: Vec2| {
            let rise = landing.y - from.y;
            let arc = JumpArc::between(
                from,
                landing,
                (rise.max(0.0) + 0.5 * H_WORLD_UNITS).min(self.jump_height),
                self.gravity,
            );
            (arc.horizontal_velocity.abs() <= self.air_speed).then_some(arc)
        };
        terrain
            .0
            .iter()
            .filter(|platform| {
                !std::ptr::eq(**platform, standing) && platform.right - platform.left > 2.0 * inset
            })
            .filter_map(|platform| {
                let rise = platform.top - self.feet.y;
                if rise > self.jump_height - 0.1 * H_WORLD_UNITS {
                    return None;
                }
                if climbing {
                    if rise < 0.25 * H_WORLD_UNITS
                        || platform.top > self.target_feet.y + 0.25 * H_WORLD_UNITS
                    {
                        return None;
                    }
                    let landing = Vec2::new(
                        self.feet
                            .x
                            .clamp(platform.left + inset, platform.right - inset),
                        platform.top,
                    );
                    if terrain.blocks_body(landing.x, landing.y) {
                        return None;
                    }
                    let score = (landing.x - self.target_feet.x).abs()
                        + (self.target_feet.y - platform.top).abs();
                    if let Some(arc) = reachable(self.feet, landing) {
                        return Some((score, JumpPlan::Leap(arc)));
                    }
                    let takeoff_x = landing
                        .x
                        .clamp(standing.left + inset, standing.right - inset);
                    reachable(Vec2::new(takeoff_x, self.feet.y), landing)
                        .map(|_| (score, JumpPlan::WalkTo(takeoff_x)))
                } else {
                    let gap = if self.direction > 0.0 {
                        platform.left - standing.right
                    } else {
                        standing.left - platform.right
                    };
                    if gap < 0.0 || rise < -self.jump_height || (gap < inset && rise.abs() < 1.0) {
                        return None;
                    }
                    let landing = Vec2::new(edge + self.direction * (gap + inset), platform.top);
                    if terrain.blocks_body(landing.x, landing.y) {
                        return None;
                    }
                    reachable(self.feet, landing).map(|arc| (gap, JumpPlan::Leap(arc)))
                }
            })
            .min_by(|(first, _), (second, _)| first.total_cmp(second))
            .map(|(_, plan)| plan)
    }

    /// A hop onto the wall blocking the way at `x`, when its ledge is within
    /// jump height and there is room to land on it.
    pub(super) fn step_up(&self, x: f32, terrain: &Terrain) -> Option<JumpArc> {
        let reach = self.jump_height - 1.1 * H_WORLD_UNITS;
        let ledge = terrain.ledge(x, self.feet.y, self.direction, reach)?;
        let from = Vec2::new(x, self.feet.y);
        let arc = JumpArc::between(
            from,
            ledge,
            (ledge.y - from.y + 0.5 * H_WORLD_UNITS).min(self.jump_height),
            self.gravity,
        );
        Some(JumpArc {
            horizontal_velocity: arc
                .horizontal_velocity
                .clamp(-self.air_speed, self.air_speed),
            ..arc
        })
    }
}
