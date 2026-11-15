//! Room layouts from `rooms/room_NNN.yaml`.

use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use bevy::math::{Rect, Vec2};
use serde::Deserialize;

use super::{
    Backdrop, BackgroundLayerRef, ColorwayId, DeviceConfig, EnemyTypeId, Fill, ProfileLayer, PropId,
};

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct RoomConfig {
    pub room_id: u16,
    pub chapter_id: u8,
    pub size: RoomSizeConfig,
    /// The background profiles mixed into the room's backdrop: their skies
    /// blended, their layers dealt together from the furthest to the nearest.
    /// An entry can also be a plain color, a sky with no layers.
    pub background_profiles: Vec<Backdrop>,
    /// Layers of those profiles the room leaves out; every layer is drawn
    /// when the list is left out.
    #[serde(default)]
    pub hidden_background_layers: Vec<BackgroundLayerRef>,
    /// How opaque the room draws layers of those profiles, from 0 to 1;
    /// a layer left out is drawn whole.
    #[serde(default)]
    pub layer_opacity: BTreeMap<BackgroundLayerRef, f32>,
    pub player_spawn: SpawnPosition,
    /// Solid platforms: bodies land on their top, are stopped by their sides,
    /// and bump their head on their underside. They stop shots too.
    pub platforms: Vec<PlatformConfig>,
    /// Non-solid panels behind the platforms or in front of the bodies.
    #[serde(default)]
    pub decorations: Vec<DecorationConfig>,
    /// Sprites standing at a spot: crates, lamps, lasers. Scenery only.
    #[serde(default)]
    pub props: Vec<PropConfig>,
    /// Traps, moving and folding platforms, buttons, doors and ladders.
    #[serde(default)]
    pub devices: Vec<DeviceConfig>,
    pub enemy_spawns: Vec<EnemySpawnConfig>,
    /// Where the room's bonus item lies once every enemy is down.
    pub bonus: SpawnPosition,
    pub exit: SpawnPosition,
    pub next_room_id: u16,
}

impl RoomConfig {
    /// The last room of a run: ten chapters of ten rooms after the opening
    /// room 0.
    pub(crate) const LAST_ID: u16 = 100;

    /// Whether the room draws `layer` of its background profile `profile`.
    pub(crate) fn shows_layer(&self, profile: &str, layer: ProfileLayer) -> bool {
        !self
            .hidden_background_layers
            .iter()
            .any(|hidden| hidden.profile == profile && hidden.layer == layer)
    }

    /// How opaque the room draws `layer` of its background profile `profile`.
    pub(crate) fn layer_opacity(&self, profile: &str, layer: ProfileLayer) -> f32 {
        self.layer_opacity
            .iter()
            .find(|(faded, _)| faded.profile == profile && faded.layer == layer)
            .map_or(1.0, |(_, opacity)| *opacity)
    }

    /// Fades `layer` in or out by `step`, within 0..=1; one drawn whole is
    /// left out of the map.
    pub(crate) fn fade_layer(&mut self, layer: BackgroundLayerRef, step: f32) {
        let opacity = self.layer_opacity(&layer.profile, layer.layer);
        // Rounded to tenths, so repeated presses don't drift.
        let faded = ((opacity + step) * 10.0).round().clamp(0.0, 10.0) / 10.0;
        if faded == 1.0 {
            self.layer_opacity.remove(&layer);
        } else {
            self.layer_opacity.insert(layer, faded);
        }
    }

    /// Hides a drawn layer of one of the room's profiles, or shows a hidden
    /// one again.
    pub(crate) fn toggle_layer(&mut self, layer: BackgroundLayerRef) {
        match self
            .hidden_background_layers
            .iter()
            .position(|hidden| *hidden == layer)
        {
            Some(index) => {
                self.hidden_background_layers.remove(index);
            }
            None => {
                self.hidden_background_layers.push(layer);
                self.hidden_background_layers.sort();
            }
        }
    }

    /// Forgets hidden and faded layers of profiles the room no longer draws.
    pub(crate) fn forget_unused_layers(&mut self) {
        let profiles = &self.background_profiles;
        let drawn = |layer: &BackgroundLayerRef| {
            profiles
                .iter()
                .any(|backdrop| matches!(backdrop, Backdrop::Profile(profile) if *profile == layer.profile))
        };
        self.hidden_background_layers.retain(drawn);
        self.layer_opacity.retain(|layer, _| drawn(layer));
    }

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
    /// Mirrored, turned about the middle of `rect_h`, and faded.
    #[serde(flatten)]
    pub placement: Placement,
}

impl PlatformConfig {
    /// The platform's corners in H once it is turned, counter-clockwise from
    /// the one that is its lower-left while it stands square.
    pub(crate) fn corners_h(&self) -> [Vec2; 4] {
        let rect = self.rect_h;
        self.placement
            .corners(Rect::new(rect.x, rect.y, rect.right(), rect.top()))
    }
}

/// How an item a room places is mirrored, turned, faded and recolored: every
/// platform, decoration, prop and device has one, left out of its room entry
/// while it stands as drawn.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(default)]
pub struct Placement {
    /// Flipped left to right about its middle, what holds a body up or hurts
    /// it included.
    pub mirrored: bool,
    /// How far it is turned about its middle, in degrees counter-clockwise.
    pub rotation_deg: f32,
    /// How opaque it is drawn, from 0 (unseen) to 1.
    pub opacity: f32,
    /// The colorway of `surfaces/colorways.yaml` its images are redrawn in;
    /// left out, it keeps the colors it was drawn with.
    pub colorway: Option<ColorwayId>,
}

impl Default for Placement {
    /// As drawn: neither mirrored nor turned, fully opaque, in its own colors.
    fn default() -> Self {
        Self {
            mirrored: false,
            rotation_deg: 0.0,
            opacity: 1.0,
            colorway: None,
        }
    }
}

impl Placement {
    /// Where `point` lands once mirrored and turned about `pivot`.
    pub(crate) fn place(&self, point: Vec2, pivot: Vec2) -> Vec2 {
        let mirrored = if self.mirrored {
            Vec2::new(2.0 * pivot.x - point.x, point.y)
        } else {
            point
        };
        // An item left square keeps the points it was given, exactly:
        // turning them by nothing would still round them.
        if self.rotation_deg == 0.0 {
            return mirrored;
        }
        pivot + Vec2::from_angle(self.rotation_deg.to_radians()).rotate(mirrored - pivot)
    }

    /// The corners of `rect`, a part of an item whose middle is the middle
    /// of `rect`, once placed: counter-clockwise from its lower-left.
    pub(crate) fn corners(&self, rect: Rect) -> [Vec2; 4] {
        self.corners_about(rect, rect.center())
    }

    /// The corners of `rect`, a part of an item whose middle is `pivot`, once
    /// placed: counter-clockwise from its lower-left, so a mirrored one is
    /// read the other way round.
    pub(crate) fn corners_about(&self, rect: Rect, pivot: Vec2) -> [Vec2; 4] {
        let corners = [
            rect.min,
            Vec2::new(rect.max.x, rect.min.y),
            rect.max,
            Vec2::new(rect.min.x, rect.max.y),
        ]
        .map(|corner| self.place(corner, pivot));
        if self.mirrored {
            [corners[1], corners[0], corners[3], corners[2]]
        } else {
            corners
        }
    }
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
    #[serde(flatten)]
    pub placement: Placement,
}

/// A prop placed in a room: `prop` of `props/props.yaml`, drawn once at its
/// own size, with the middle of its bottom edge on `position_h`.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct PropConfig {
    #[serde(default)]
    pub label: Option<String>,
    pub prop: PropId,
    pub position_h: SpawnPosition,
    /// Behind the bodies (over the platforms) when left out, or over them.
    #[serde(default)]
    pub layer: DecorationLayer,
    /// Whether bodies stand on it and bump into it, over what its sprite says
    /// (`PropCatalog::is_solid`): left out, it is as its sprite is.
    #[serde(default)]
    pub solid: Option<bool>,
    #[serde(flatten)]
    pub placement: Placement,
}

/// Where a decoration panel is drawn: behind the platforms, or over the
/// bodies. A prop's background is behind the bodies but over the platforms.
#[derive(Debug, Clone, Copy, Default, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DecorationLayer {
    #[default]
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
    /// The way it looks, and so watches, until it spots the player.
    #[serde(default)]
    pub facing: Facing,
    /// How many layers of armor it walks in with: none when left out.
    #[serde(default)]
    pub armor_layers: u8,
    /// Which wave it arrives with. Wave 0, left out of the file, stands in
    /// the room as it opens; each later wave arrives once every enemy of the
    /// waves before it is down, and arrives knowing where the player is.
    #[serde(default)]
    pub wave: u8,
}

/// The way an enemy looks while it idles. It defaults to `Left`, the way the
/// player walks in from; a chasing enemy turns toward the player instead.
#[derive(Debug, Clone, Copy, Default, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Facing {
    #[default]
    Left,
    Right,
}

/// The sign combat, sight, and sprite flipping work in: -1 left, 1 right.
impl From<Facing> for f32 {
    fn from(facing: Facing) -> Self {
        match facing {
            Facing::Left => -1.0,
            Facing::Right => 1.0,
        }
    }
}
