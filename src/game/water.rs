//! The flood standing in a room's `water:` rectangles. It is the same water
//! the main menu is drawn over — the city drowned, and the run is a climb out
//! of it — so a pool is the flood shader laid over the room rather than a
//! colour of its own. Nothing about it is solid: a body falls in, and inside
//! it swims (`player/mod.rs`) instead of walking.

use super::*;
use crate::data::WaterPool;

/// In front of the player, so a swimmer is seen through the water, but behind
/// the panels a room hangs over the action.
const WATER_Z: f32 = 5.0;

#[derive(Component, Clone, Copy, Debug)]
pub(super) struct WaterSurface {
    pub(super) left: f32,
    pub(super) right: f32,
    pub(super) bottom: f32,
    pub(super) top: f32,
}

impl WaterSurface {
    /// Whether the point sits in this pool.
    fn holds(&self, point: Vec2) -> bool {
        point.x > self.left && point.x < self.right && point.y > self.bottom && point.y < self.top
    }
}

/// Every pool of the current room, gathered once per system run.
pub(super) struct Waters<'a>(pub(super) Vec<&'a WaterSurface>);

impl<'a> Waters<'a> {
    /// The pool a body is swimming in: the one holding the middle of its
    /// collision box, half an H above `feet_y`.
    pub(super) fn around(&self, x: f32, feet_y: f32) -> Option<&'a WaterSurface> {
        let center = Vec2::new(x, feet_y + H_WORLD_UNITS / 2.0);
        self.0.iter().copied().find(|pool| pool.holds(center))
    }
}

/// Lays the room's pools: one flood quad each, which also carries the pool's
/// collider.
pub(super) fn spawn_room_water(
    commands: &mut Commands,
    pools: &[WaterPool],
    data: &GameData,
    shaders: &mut ShaderAssets,
) {
    let config = &data.gameplay.effects.flood;
    for pool in pools {
        let rect = pool.rect_h;
        let size_h = Vec2::new(rect.width, rect.height);
        let material = shaders.flood.add(flood_material(
            config,
            size_h,
            data.gameplay.player.water.opacity,
        ));
        commands.spawn((
            Mesh2d(shaders.quad.0.clone()),
            MeshMaterial2d(material),
            Transform::from_xyz(
                (rect.x + rect.width / 2.0) * H_WORLD_UNITS,
                (rect.y + rect.height / 2.0) * H_WORLD_UNITS,
                WATER_Z,
            )
            .with_scale((size_h * H_WORLD_UNITS).extend(1.0)),
            WaterSurface {
                left: rect.x * H_WORLD_UNITS,
                right: rect.right() * H_WORLD_UNITS,
                bottom: rect.y * H_WORLD_UNITS,
                top: rect.top() * H_WORLD_UNITS,
            },
            RoomEntity,
        ));
    }
}
