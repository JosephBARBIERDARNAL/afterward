//! Room layouts from `rooms/room_NNN.yaml`.

use std::path::{Path, PathBuf};

use serde::Deserialize;

use super::{EnemyTypeId, Fill};

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
    /// One-way platforms: bodies land on their top, pass through them from
    /// every other side, and drop through them on purpose. They stop shots.
    pub platforms: Vec<PlatformConfig>,
    /// Non-solid panels behind the platforms or in front of the bodies.
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
}

/// The room's extent in H. Both sides are at least one viewport, so the view
/// never shows past the room.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct RoomSizeConfig {
    pub width_h: f32,
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
pub struct PlatformConfig {
    /// What the block is ("floor", "upper walkway"), shown in the debug editor.
    #[serde(default)]
    pub label: Option<String>,
    /// What it is painted with: a material or a plain color.
    #[serde(flatten)]
    pub fill: Fill,
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
}

/// A non-solid panel, such as a back wall behind the action or a sign
/// hanging in front of it.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct DecorationConfig {
    /// What the panel is ("back wall"), shown in the debug editor.
    #[serde(default)]
    pub label: Option<String>,
    #[serde(flatten)]
    pub fill: Fill,
    pub layer: DecorationLayer,
    pub rect_h: RectH,
}

/// Where a decoration panel is drawn: behind the platforms, or over the bodies.
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
