//! Room layouts from `rooms/room_NNN.yaml`.

use std::path::{Path, PathBuf};

use serde::Deserialize;

use super::{EnemyTypeId, GameplayConfig, MaterialId, PLAYER_COLLISION_WIDTH_H};

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct RoomConfig {
    pub room_id: u16,
    pub chapter_id: u8,
    /// A short note on the layout's idea, for whoever edits the room.
    #[serde(default)]
    pub description: Option<String>,
    pub size: RoomSizeConfig,
    pub background_profile: String,
    pub player_spawn: SpawnPosition,
    /// Solid blocks (floors, walls, ceilings, pillars): they block bodies and
    /// projectiles from every side.
    pub terrain: Vec<TerrainBlockConfig>,
    /// One-way platforms: bodies land on their top, jump through them from
    /// below, and drop through them on purpose.
    #[serde(default)]
    pub platforms: Vec<TerrainBlockConfig>,
    /// Non-solid textured panels behind the terrain or in front of the bodies.
    #[serde(default)]
    pub decorations: Vec<DecorationConfig>,
    pub enemy_spawns: Vec<EnemySpawnConfig>,
    pub exit: SpawnPosition,
    pub next_room_id: u16,
}

impl RoomConfig {
    /// Where room `room_id` is stored under the asset root.
    pub(crate) fn file_path(asset_root: &Path, room_id: u16) -> PathBuf {
        asset_root.join(format!("rooms/room_{room_id:03}.yaml"))
    }

    pub(crate) fn width_h(&self, gameplay: &GameplayConfig) -> f32 {
        f32::from(self.size.width_viewports) * gameplay.viewport_width_h()
    }

    /// Whether feet at (`x_h`, `y_h`) stand on a block top with a body's
    /// worth of room above them, free of solid terrain.
    pub(crate) fn has_footing_at(&self, x_h: f32, y_h: f32) -> bool {
        let stands =
            self.terrain.iter().chain(&self.platforms).any(|block| {
                block.rect_h.covers_x(x_h) && (block.rect_h.top() - y_h).abs() <= 0.01
            });
        stands && self.body_is_clear(x_h, y_h)
    }

    /// Whether a 1 H tall collision box centered on `x_h`, with its feet at
    /// `y_h`, overlaps no solid terrain.
    pub(crate) fn body_is_clear(&self, x_h: f32, y_h: f32) -> bool {
        let body = RectH {
            x: x_h - PLAYER_COLLISION_WIDTH_H / 2.0,
            y: y_h,
            width: PLAYER_COLLISION_WIDTH_H,
            height: 1.0,
        };
        !self
            .terrain
            .iter()
            .any(|block| block.rect_h.overlaps(&body))
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct RoomSizeConfig {
    pub width_viewports: u8,
    pub height_h: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
pub struct SpawnPosition {
    #[serde(alias = "x")]
    pub x_h: f32,
    #[serde(alias = "y")]
    pub y_h: f32,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct TerrainBlockConfig {
    /// What the block is ("floor", "hanging rock"), shown in the debug editor.
    #[serde(default)]
    pub label: Option<String>,
    /// A material from `terrain/materials.yaml`: its look and footsteps.
    pub material: MaterialId,
    pub rect_h: RectH,
}

/// An axis-aligned rectangle in H; `x`/`y` is its lower-left corner.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
pub struct RectH {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl RectH {
    pub(crate) fn right(&self) -> f32 {
        self.x + self.width
    }

    pub(crate) fn top(&self) -> f32 {
        self.y + self.height
    }

    pub(crate) fn covers_x(&self, x: f32) -> bool {
        (self.x..=self.right()).contains(&x)
    }

    /// Strict overlap: rectangles that only share an edge do not overlap.
    pub(crate) fn overlaps(&self, other: &Self) -> bool {
        const EPSILON: f32 = 0.001;
        self.x < other.right() - EPSILON
            && other.x < self.right() - EPSILON
            && self.y < other.top() - EPSILON
            && other.y < self.top() - EPSILON
    }
}

/// A non-solid panel tiled with a terrain material, such as a ruined back
/// wall or foliage hanging in front of the action.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct DecorationConfig {
    /// What the panel is ("back wall"), shown in the debug editor.
    #[serde(default)]
    pub label: Option<String>,
    pub material: MaterialId,
    pub layer: DecorationLayer,
    pub rect_h: RectH,
}

/// Where a decoration panel is drawn: behind the terrain, or over the bodies.
#[derive(Debug, Clone, Copy, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DecorationLayer {
    Background,
    Foreground,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct EnemySpawnConfig {
    /// What the enemy is there for ("sniper perch"), shown in the debug editor.
    #[serde(default)]
    pub label: Option<String>,
    pub type_id: EnemyTypeId,
    pub position_h: SpawnPosition,
    pub stat_overrides: Option<EnemyStatOverrides>,
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
pub struct EnemyStatOverrides {
    pub health: Option<f32>,
    pub movement_speed_h_s: Option<f32>,
    pub attack_damage: Option<f32>,
}
