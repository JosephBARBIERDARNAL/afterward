//! The inspector: the editable fields of the selection (or of the room when
//! nothing is selected), changed with Left/Right, typed into, or cleared.
//! With several items selected, Fill repaints all of them and X/Y move them
//! together. Fill and the two backgrounds take a typed hex code (`#4C5A74`,
//! `4c5a74`, `#ABC`) as well as a material or profile name.

use super::*;
use crate::data::{Backdrop, DecorationLayer, Facing, HexColor, MaterialId};

/// How far one Left/Right press grows or shrinks the room, in H.
const ROOM_SIZE_STEP_H: f32 = 0.5;
/// How far one Left/Right press turns a platform, in degrees.
const ROTATION_STEP_DEG: f32 = 5.0;
/// The most layers of armor Left/Right gives an enemy: more than this and the
/// row of icons over its health bar outgrows the bar by too much to read.
const MAX_ARMOR_LAYERS: u8 = 5;
const MAX_WAVE: u8 = 9;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Field {
    Room,
    RoomWidth,
    RoomHeight,
    Background,
    BackgroundOver,
    Label,
    Fill,
    Layer,
    EnemyType,
    EnemyFacing,
    X,
    Y,
    Width,
    Height,
    Rotation,
    Armor,
    Wave,
    // Shared by every enemy of a type or by the player, in every room
    // (hitboxes.rs): the size the player is drawn at, then the hitboxes.
    BodyHeight,
    HurtboxWidth,
    HurtboxHeight,
    Attack,
    HitboxX,
    HitboxY,
    HitboxWidth,
    HitboxHeight,
}

impl Field {
    pub(super) fn for_selection(selection: &[EditorItem]) -> &'static [Self] {
        let item = match selection {
            [] => None,
            [item] => Some(*item),
            _ if selection.iter().any(|item| {
                matches!(item, EditorItem::Platform(_) | EditorItem::Decoration(_))
            }) =>
            {
                return &[Self::Fill, Self::X, Self::Y];
            }
            _ => return &[Self::X, Self::Y],
        };
        match item {
            None => &[
                Self::Room,
                Self::RoomWidth,
                Self::RoomHeight,
                Self::Background,
                Self::BackgroundOver,
            ],
            Some(EditorItem::Platform(_)) => &[
                Self::Label,
                Self::Fill,
                Self::X,
                Self::Y,
                Self::Width,
                Self::Height,
                Self::Rotation,
            ],
            Some(EditorItem::Decoration(_)) => &[
                Self::Label,
                Self::Fill,
                Self::Layer,
                Self::X,
                Self::Y,
                Self::Width,
                Self::Height,
            ],
            Some(EditorItem::Enemy(_)) => &[
                Self::Label,
                Self::EnemyType,
                Self::EnemyFacing,
                Self::X,
                Self::Y,
                Self::Armor,
                Self::Wave,
                Self::HurtboxWidth,
                Self::HurtboxHeight,
                Self::Attack,
                Self::HitboxX,
                Self::HitboxY,
                Self::HitboxWidth,
                Self::HitboxHeight,
            ],
            Some(EditorItem::PlayerSpawn) => &[
                Self::X,
                Self::Y,
                Self::BodyHeight,
                Self::HurtboxWidth,
                Self::HurtboxHeight,
            ],
            Some(EditorItem::Exit | EditorItem::Bonus) => &[Self::X, Self::Y],
        }
    }

    pub(super) fn name(self) -> &'static str {
        match self {
            Self::Room => "Room",
            Self::RoomWidth => "Room width (H)",
            Self::RoomHeight => "Room height (H)",
            Self::Background => "Background 1",
            Self::BackgroundOver => "Background 2",
            Self::Label => "Label",
            Self::Fill => "Fill",
            Self::Layer => "Layer",
            Self::EnemyType => "Type",
            Self::EnemyFacing => "Facing",
            Self::X => "X (H)",
            Self::Y => "Y (H)",
            Self::Width => "Width (H)",
            Self::Height => "Height (H)",
            Self::Rotation => "Rotation (deg)",
            Self::Armor => "Armor layers",
            Self::Wave => "Wave",
            Self::BodyHeight => "Body height (H)",
            Self::HurtboxWidth => "Hurtbox width (H)",
            Self::HurtboxHeight => "Hurtbox height (H)",
            Self::Attack => "Attack",
            Self::HitboxX => "Hitbox x (H)",
            Self::HitboxY => "Hitbox y (H)",
            Self::HitboxWidth => "Hitbox width (H)",
            Self::HitboxHeight => "Hitbox height (H)",
        }
    }

    pub(super) fn is_text(self) -> bool {
        matches!(
            self,
            Self::Label | Self::Fill | Self::Background | Self::BackgroundOver
        )
    }

    /// Whether Left/Right (and the row's < and >) change the field.
    pub(super) fn steps(self) -> bool {
        self != Self::Label
    }

    /// Whether C (and the row's x) clears the field: see `Field::clear`.
    pub(super) fn clears(self) -> bool {
        matches!(
            self,
            Self::Label | Self::BackgroundOver | Self::Rotation | Self::Armor
        )
    }

    /// Which of the room's backgrounds a background row stands for.
    fn backdrop_index(self) -> usize {
        usize::from(self == Self::BackgroundOver)
    }

    /// The field's text, for typing into it.
    pub(super) fn text(self, room: &RoomConfig, item: Option<EditorItem>) -> Option<String> {
        match (self, item) {
            (Self::Label, Some(EditorItem::Platform(index))) => room
                .platforms
                .get(index)
                .and_then(|block| block.label.clone()),
            (Self::Label, Some(EditorItem::Enemy(index))) => room
                .enemy_spawns
                .get(index)
                .and_then(|spawn| spawn.label.clone()),
            (Self::Label, Some(EditorItem::Decoration(index))) => room
                .decorations
                .get(index)
                .and_then(|decoration| decoration.label.clone()),
            (Self::Fill, Some(item)) => item.fill(room).map(ToString::to_string),
            (Self::Background | Self::BackgroundOver, _) => room
                .background_profiles
                .get(self.backdrop_index())
                .map(ToString::to_string),
            _ => None,
        }
    }

    /// Sets a text field from what was typed. A label is removed by empty
    /// text. A fill or a background takes a material or profile name it
    /// knows, else a hex code, and is left alone by anything else; Background
    /// 2 is taken off by empty text.
    pub(super) fn set_text(
        self,
        room: &mut RoomConfig,
        selection: &[EditorItem],
        data: &GameData,
        text: &str,
    ) {
        let text = text.trim();
        match self {
            Self::Label => {
                let label = Some(text.to_string()).filter(|text| !text.is_empty());
                Self::set_label(room, selection.last().copied(), label);
            }
            Self::Fill => {
                let fill = text
                    .parse::<MaterialId>()
                    .ok()
                    .filter(|material| data.surfaces.materials.contains_key(material))
                    .map(Fill::Material)
                    .or_else(|| text.parse().ok().map(Fill::Color));
                if let Some(fill) = fill {
                    for item in selection {
                        item.set_fill(room, &fill);
                    }
                }
            }
            Self::Background | Self::BackgroundOver => {
                if text.is_empty() {
                    self.set_backdrop(room, None);
                } else if data.backgrounds.profiles.contains_key(text) {
                    self.set_backdrop(room, Some(Backdrop::Profile(text.to_string())));
                } else if let Ok(color) = text.parse::<HexColor>() {
                    self.set_backdrop(room, Some(Backdrop::Color(color)));
                }
            }
            _ => {}
        }
    }

    /// Sets Background 1 or 2. `None` takes Background 2 off and leaves
    /// Background 1, which a room cannot do without, as it is.
    pub(super) fn set_backdrop(self, room: &mut RoomConfig, backdrop: Option<Backdrop>) {
        let index = self.backdrop_index();
        match backdrop {
            None if index == 1 && room.background_profiles.len() > 1 => {
                room.background_profiles.remove(1);
            }
            None => {}
            Some(backdrop) => match room.background_profiles.get_mut(index) {
                Some(slot) => *slot = backdrop,
                None => room.background_profiles.push(backdrop),
            },
        }
        room.forget_unused_layers();
    }

    fn set_label(room: &mut RoomConfig, item: Option<EditorItem>, text: Option<String>) {
        match item {
            Some(EditorItem::Platform(index)) => {
                if let Some(block) = room.platforms.get_mut(index) {
                    block.label = text;
                }
            }
            Some(EditorItem::Enemy(index)) => {
                if let Some(spawn) = room.enemy_spawns.get_mut(index) {
                    spawn.label = text;
                }
            }
            Some(EditorItem::Decoration(index)) => {
                if let Some(decoration) = room.decorations.get_mut(index) {
                    decoration.label = text;
                }
            }
            _ => {}
        }
    }

    pub(super) fn value(
        self,
        room: &RoomConfig,
        selection: &[EditorItem],
        data: &GameData,
        attack: usize,
    ) -> String {
        if selection.len() > 1 {
            return match self {
                Self::Fill => {
                    let mut fills = selection.iter().filter_map(|item| item.fill(room));
                    let first = fills.next();
                    match first {
                        Some(first) if fills.all(|fill| fill == first) => first.to_string(),
                        _ => String::from("(mixed)"),
                    }
                }
                _ => format!("Left/Right moves all {}", selection.len()),
            };
        }
        let item = selection.last().copied();
        let rect = item.and_then(|item| item.bounds(room));
        let origin = item.and_then(|item| item.origin(room));
        let spawn = match item {
            Some(EditorItem::Enemy(index)) => room.enemy_spawns.get(index),
            _ => None,
        };
        let defaults = spawn.and_then(|spawn| data.enemies.types.get(&spawn.type_id));
        match self {
            Self::Room => format!("{:02}", room.room_id),
            Self::RoomWidth => room.size.width_h.to_string(),
            Self::RoomHeight => room.size.height_h.to_string(),
            Self::Background => room
                .background_profiles
                .first()
                .map_or_else(String::new, ToString::to_string),
            Self::BackgroundOver => match room.background_profiles.get(1..) {
                Some([]) | None => String::from("None"),
                Some(over) => over
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(", "),
            },
            Self::Label => self.text(room, item).unwrap_or_else(|| String::from("-")),
            Self::Fill => item
                .and_then(|item| item.fill(room))
                .map_or_else(String::new, ToString::to_string),
            Self::Layer => match item {
                Some(EditorItem::Decoration(index)) => room
                    .decorations
                    .get(index)
                    .map_or_else(String::new, |decoration| decoration.layer.to_string()),
                _ => String::new(),
            },
            Self::EnemyType => spawn.map_or_else(String::new, |spawn| spawn.type_id.to_string()),
            Self::EnemyFacing => spawn.map_or_else(String::new, |spawn| spawn.facing.to_string()),
            Self::X => origin.map_or_else(String::new, |origin| origin.x.to_string()),
            Self::Y => origin.map_or_else(String::new, |origin| origin.y.to_string()),
            Self::Width => rect.map_or_else(String::new, |rect| rect.width.to_string()),
            Self::Height => rect.map_or_else(String::new, |rect| rect.height.to_string()),
            Self::Rotation => match item {
                Some(EditorItem::Platform(index)) => {
                    room.platforms.get(index).map_or_else(String::new, |block| {
                        // How far its gentler upper face leans, which is what
                        // decides whether a body walks it.
                        let lean = block.rotation_deg.abs() % 90.0;
                        let lean = lean.min(90.0 - lean);
                        if lean == 0.0 {
                            block.rotation_deg.to_string()
                        } else if lean <= data.gameplay.player.walkable_slope_deg {
                            format!("{}  (a slope to walk)", block.rotation_deg)
                        } else {
                            format!("{}  (too steep to walk: a wall)", block.rotation_deg)
                        }
                    })
                }
                _ => String::new(),
            },
            Self::Armor => match spawn.map(|spawn| spawn.armor_layers) {
                Some(0) => String::from("none"),
                Some(layers) => layers.to_string(),
                None => String::new(),
            },
            Self::Wave => match spawn.map(|spawn| spawn.wave) {
                Some(0) => String::from("0  (there as the room opens)"),
                Some(wave) => format!("{wave}  (arrives once wave {} is down)", wave - 1),
                None => String::new(),
            },
            Self::BodyHeight
            | Self::HurtboxWidth
            | Self::HurtboxHeight
            | Self::Attack
            | Self::HitboxX
            | Self::HitboxY
            | Self::HitboxWidth
            | Self::HitboxHeight => self.hitbox_value(data, defaults, attack),
        }
    }

    /// Steps the field by `steps` (negative to decrease) in the edited room,
    /// for every selected item.
    pub(super) fn adjust(
        self,
        room: &mut RoomConfig,
        selection: &[EditorItem],
        data: &GameData,
        steps: f32,
    ) {
        let item = selection.last().copied();
        let cycle = |options: Vec<String>, current: &str| {
            let count = options.len();
            let index = options
                .iter()
                .position(|option| option == current)
                .unwrap_or(0);
            let next = (index as isize + steps.signum() as isize).rem_euclid(count.max(1) as isize);
            options.get(next as usize).cloned()
        };
        match self {
            // Both sides keep at least one viewport, so the view never
            // shows past the room.
            Self::RoomWidth => {
                room.size.width_h = (room.size.width_h + steps * ROOM_SIZE_STEP_H)
                    .max(data.gameplay.viewport_width_h());
            }
            Self::RoomHeight => {
                room.size.height_h = (room.size.height_h + steps * ROOM_SIZE_STEP_H)
                    .max(data.gameplay.viewport.height_h);
            }
            // Left/Right walk the profiles; a color (typed, or picked in the
            // palette) steps onto the first of them. For Background 2 an
            // empty name stands for nothing, so Left/Right walk through it
            // like any other. Profiles a hand-written file stacks beyond that
            // are left alone.
            Self::Background | Self::BackgroundOver => {
                let none = (self == Self::BackgroundOver).then(String::new);
                let profiles = none
                    .into_iter()
                    .chain(data.backgrounds.profiles.keys().cloned())
                    .collect();
                let current = room
                    .background_profiles
                    .get(self.backdrop_index())
                    .map(ToString::to_string)
                    .unwrap_or_default();
                if let Some(profile) = cycle(profiles, &current) {
                    let backdrop = Some(profile)
                        .filter(|profile| !profile.is_empty())
                        .map(Backdrop::Profile);
                    self.set_backdrop(room, backdrop);
                }
            }
            Self::Fill => {
                let materials = data
                    .surfaces
                    .materials
                    .keys()
                    .map(ToString::to_string)
                    .collect();
                let current = match selection.iter().rev().find_map(|item| item.fill(room)) {
                    Some(Fill::Material(material)) => material.to_string(),
                    Some(Fill::Color(_)) | None => String::new(),
                };
                if let Some(material) =
                    cycle(materials, &current).and_then(|material| material.parse().ok())
                {
                    let fill = Fill::Material(material);
                    for item in selection {
                        item.set_fill(room, &fill);
                    }
                }
            }
            Self::Layer => {
                if let Some(EditorItem::Decoration(index)) = item
                    && let Some(decoration) = room.decorations.get_mut(index)
                {
                    decoration.layer = match decoration.layer {
                        DecorationLayer::Background => DecorationLayer::Foreground,
                        DecorationLayer::Foreground => DecorationLayer::Background,
                    };
                }
            }
            Self::EnemyType => {
                if let Some(EditorItem::Enemy(index)) = item
                    && let Some(spawn) = room.enemy_spawns.get_mut(index)
                {
                    let types = data.enemies.types.keys().map(ToString::to_string).collect();
                    if let Some(type_id) = cycle(types, &spawn.type_id)
                        .and_then(|type_id| type_id.parse::<EnemyTypeId>().ok())
                    {
                        spawn.type_id = type_id;
                    }
                }
            }
            Self::EnemyFacing => {
                if let Some(EditorItem::Enemy(index)) = item
                    && let Some(spawn) = room.enemy_spawns.get_mut(index)
                {
                    spawn.facing = match spawn.facing {
                        Facing::Left => Facing::Right,
                        Facing::Right => Facing::Left,
                    };
                }
            }
            Self::X | Self::Y => {
                let step = Vec2::new(
                    if self == Self::X { steps * SNAP_H } else { 0.0 },
                    if self == Self::Y { steps * SNAP_H } else { 0.0 },
                );
                for item in selection {
                    if let Some(origin) = item.origin(room) {
                        item.set_origin(room, origin + step);
                    }
                }
            }
            Self::Width | Self::Height => {
                if let Some(rect) = item.and_then(|item| item.rect_mut(room)) {
                    let size = if self == Self::Width {
                        &mut rect.width
                    } else {
                        &mut rect.height
                    };
                    *size = (*size + steps * SNAP_H).max(SNAP_H);
                }
            }
            Self::Rotation => {
                if let Some(EditorItem::Platform(index)) = item
                    && let Some(block) = room.platforms.get_mut(index)
                {
                    // Half a turn brings a rectangle back onto itself, so the
                    // angle stays within a quarter turn either way. Adding
                    // 0.0 turns -0.0 into 0.0.
                    let turned = block.rotation_deg + steps * ROTATION_STEP_DEG;
                    block.rotation_deg = (turned + 90.0).rem_euclid(180.0) - 90.0 + 0.0;
                }
            }
            Self::Armor => {
                if let Some(EditorItem::Enemy(index)) = item
                    && let Some(spawn) = room.enemy_spawns.get_mut(index)
                {
                    // Left from none stays at none: no armor is 0 layers.
                    spawn.armor_layers = if steps < 0.0 {
                        spawn.armor_layers.saturating_sub(1)
                    } else {
                        spawn.armor_layers.saturating_add(1).min(MAX_ARMOR_LAYERS)
                    };
                }
            }
            Self::Wave => {
                if let Some(EditorItem::Enemy(index)) = item
                    && let Some(spawn) = room.enemy_spawns.get_mut(index)
                {
                    spawn.wave = if steps < 0.0 {
                        spawn.wave.saturating_sub(1)
                    } else {
                        spawn.wave.saturating_add(1).min(MAX_WAVE)
                    };
                }
            }
            // Hitboxes live outside the room: see `Field::adjust_hitbox`.
            Self::Room
            | Self::Label
            | Self::BodyHeight
            | Self::HurtboxWidth
            | Self::HurtboxHeight
            | Self::Attack
            | Self::HitboxX
            | Self::HitboxY
            | Self::HitboxWidth
            | Self::HitboxHeight => {}
        }
    }

    /// Removes a label or Background 2, strips an enemy's armor, or stands a
    /// turned platform square again.
    pub(super) fn clear(self, room: &mut RoomConfig, item: Option<EditorItem>) {
        if self == Self::Label {
            Self::set_label(room, item, None);
            return;
        }
        if self == Self::BackgroundOver {
            self.set_backdrop(room, None);
            return;
        }
        if let (Self::Rotation, Some(EditorItem::Platform(index))) = (self, item)
            && let Some(block) = room.platforms.get_mut(index)
        {
            block.rotation_deg = 0.0;
            return;
        }
        if let (Self::Armor, Some(EditorItem::Enemy(index))) = (self, item)
            && let Some(spawn) = room.enemy_spawns.get_mut(index)
        {
            spawn.armor_layers = 0;
        }
    }
}
