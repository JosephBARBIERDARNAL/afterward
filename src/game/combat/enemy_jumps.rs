use super::*;

/// How far past the nearest spot a climb can start from an enemy is sent to
/// take off, in world units: its walk there stops within a unit of the spot.
const TAKEOFF_MARGIN: f32 = 2.0;

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
    pub(super) fn plan(
        &self,
        standing: &PlatformSurface,
        platforms: &Platforms,
    ) -> Option<JumpPlan> {
        let inset = HALF_BODY_WIDTH + 0.1 * H_WORLD_UNITS;
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
        platforms
            .0
            .iter()
            // A tilted platform is walked onto, never leapt at: its bounds say
            // nothing of where its slope could be landed on.
            .filter(|platform| {
                !std::ptr::eq(**platform, standing)
                    && platform.tilt.is_none()
                    && platform.right - platform.left > 2.0 * inset
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
                    // A solid block is climbed at an edge: the enemy takes off
                    // clear of it and lands just inside the edge it comes from,
                    // instead of rising through its underside. Clear means far
                    // enough out that its feet are over the top by the time
                    // its body is over the edge: the leap covers the run-up
                    // while it rises and the way in once it is above.
                    let from_left = self.feet.x < (platform.left + platform.right) / 2.0;
                    let apex = (rise + 0.5 * H_WORLD_UNITS).min(self.jump_height);
                    let rise_time = (2.0 * apex / self.gravity).sqrt();
                    let over_the_top = (2.0 * (apex - rise) / self.gravity).sqrt();
                    let run_up = (HALF_BODY_WIDTH + inset) * (rise_time - over_the_top)
                        / (2.0 * over_the_top).max(f32::EPSILON);
                    let clear_of = |x: f32| {
                        if from_left {
                            x + HALF_BODY_WIDTH + run_up <= platform.left
                        } else {
                            x - HALF_BODY_WIDTH - run_up >= platform.right
                        }
                    };
                    let landing = Vec2::new(
                        if from_left {
                            platform.left + inset
                        } else {
                            platform.right - inset
                        },
                        platform.top,
                    );
                    let score = (landing.x - self.target_feet.x).abs()
                        + (self.target_feet.y - platform.top).abs();
                    if clear_of(self.feet.x)
                        && let Some(arc) = reachable(self.feet, landing)
                    {
                        return Some((score, JumpPlan::Leap(arc)));
                    }
                    // Walk out from under the block first. A block narrower
                    // than a body has room for one take-off spot only.
                    // The walk there stops a step short, so the spot is put
                    // that much further out than it has to be.
                    let takeoff_x = if standing.right - standing.left > 2.0 * inset {
                        if from_left {
                            platform.left - HALF_BODY_WIDTH - run_up - TAKEOFF_MARGIN
                        } else {
                            platform.right + HALF_BODY_WIDTH + run_up + TAKEOFF_MARGIN
                        }
                        .clamp(standing.left + inset, standing.right - inset)
                    } else {
                        (standing.left + standing.right) / 2.0
                    };
                    (clear_of(takeoff_x)
                        && reachable(Vec2::new(takeoff_x, self.feet.y), landing).is_some())
                    .then_some((score, JumpPlan::WalkTo(takeoff_x)))
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
                    reachable(self.feet, landing).map(|arc| (gap, JumpPlan::Leap(arc)))
                }
            })
            .min_by(|(first, _), (second, _)| first.total_cmp(second))
            .map(|(_, plan)| plan)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_enemy_on_a_block_narrower_than_its_body_plans_a_jump_without_panicking() {
        let h = H_WORLD_UNITS;
        let block = |left: f32, right: f32, top: f32| PlatformSurface {
            left: left * h,
            right: right * h,
            bottom: (top - 0.5) * h,
            top: top * h,
            footstep: FootstepSound::Stone,
            tilt: None,
        };
        // A 0.5 H wide stalagmite, and a ledge 2 H up and 3 H away.
        let standing = block(10.0, 10.5, 1.0);
        let ledge = block(13.0, 17.0, 3.0);
        let platforms = Platforms(vec![&standing, &ledge]);
        let route = JumpRoute {
            feet: Vec2::new(10.25 * h, h),
            target_feet: Vec2::new(15.0 * h, 3.0 * h),
            direction: 1.0,
            jump_height: 2.25 * h,
            air_speed: 2.0 * h,
            gravity: 32.0 * h,
            level_change_threshold: h,
        };
        assert!(matches!(
            route.plan(&standing, &platforms),
            Some(JumpPlan::WalkTo(_)) | None
        ));
    }
}
