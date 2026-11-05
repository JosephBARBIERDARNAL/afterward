//! The inspector: the editable fields of the selection (or of the room when
//! nothing is selected), changed with Left/Right, typed into, or cleared.

use super::*;
use crate::data::{EnemyStatOverrides, EnemyTypeConfig, PlatformMaterial};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Field {
    Room,
    RoomWidth,
    RoomHeight,
    Background,
    Description,
    Label,
    Surface,
    EnemyType,
    X,
    Y,
    Width,
    Height,
    Health,
    Speed,
    Damage,
}

impl Field {
    pub(super) fn for_item(item: Option<EditorItem>) -> &'static [Self] {
        match item {
            None => &[
                Self::Room,
                Self::RoomWidth,
                Self::RoomHeight,
                Self::Background,
                Self::Description,
            ],
            Some(EditorItem::Terrain(_)) => &[
                Self::Label,
                Self::Surface,
                Self::X,
                Self::Y,
                Self::Width,
                Self::Height,
            ],
            Some(EditorItem::Platform(_)) => {
                &[Self::Label, Self::X, Self::Y, Self::Width, Self::Height]
            }
            Some(EditorItem::Enemy(_)) => &[
                Self::Label,
                Self::EnemyType,
                Self::X,
                Self::Y,
                Self::Health,
                Self::Speed,
                Self::Damage,
            ],
            Some(EditorItem::PlayerSpawn | EditorItem::Exit) => &[Self::X, Self::Y],
        }
    }

    pub(super) fn name(self) -> &'static str {
        match self {
            Self::Room => "Room",
            Self::RoomWidth => "Width (viewports)",
            Self::RoomHeight => "Height (H)",
            Self::Background => "Background",
            Self::Description => "Description",
            Self::Label => "Label",
            Self::Surface => "Surface",
            Self::EnemyType => "Type",
            Self::X => "X (H)",
            Self::Y => "Y (H)",
            Self::Width => "Width (H)",
            Self::Height => "Height (H)",
            Self::Health => "Health",
            Self::Speed => "Speed (H/s)",
            Self::Damage => "Attack damage",
        }
    }

    pub(super) fn is_text(self) -> bool {
        matches!(self, Self::Description | Self::Label)
    }

    /// The field's text, for typing into it.
    pub(super) fn text(self, room: &RoomConfig, item: Option<EditorItem>) -> Option<String> {
        match (self, item) {
            (Self::Description, _) => room.description.clone(),
            (Self::Label, Some(EditorItem::Terrain(index))) => room
                .terrain
                .get(index)
                .and_then(|block| block.label.clone()),
            (Self::Label, Some(EditorItem::Platform(index))) => room
                .platforms
                .get(index)
                .and_then(|block| block.label.clone()),
            (Self::Label, Some(EditorItem::Enemy(index))) => room
                .enemy_spawns
                .get(index)
                .and_then(|spawn| spawn.label.clone()),
            _ => None,
        }
    }

    /// Sets a text field; empty text removes it.
    pub(super) fn set_text(self, room: &mut RoomConfig, item: Option<EditorItem>, text: &str) {
        let text = Some(text.trim().to_string()).filter(|text| !text.is_empty());
        match (self, item) {
            (Self::Description, _) => room.description = text,
            (Self::Label, Some(EditorItem::Terrain(index))) => {
                if let Some(block) = room.terrain.get_mut(index) {
                    block.label = text;
                }
            }
            (Self::Label, Some(EditorItem::Platform(index))) => {
                if let Some(block) = room.platforms.get_mut(index) {
                    block.label = text;
                }
            }
            (Self::Label, Some(EditorItem::Enemy(index))) => {
                if let Some(spawn) = room.enemy_spawns.get_mut(index) {
                    spawn.label = text;
                }
            }
            _ => {}
        }
    }

    pub(super) fn value(
        self,
        room: &RoomConfig,
        item: Option<EditorItem>,
        data: &GameData,
    ) -> String {
        let rect = item.and_then(|item| item.bounds(room));
        let origin = item.and_then(|item| item.origin(room));
        let spawn = match item {
            Some(EditorItem::Enemy(index)) => room.enemy_spawns.get(index),
            _ => None,
        };
        let overrides = spawn.and_then(|spawn| spawn.stat_overrides.as_ref());
        let defaults = spawn.and_then(|spawn| data.enemies.types.get(&spawn.type_id));
        let stat = |value: Option<f32>, default: Option<f32>| match (value, default) {
            (Some(value), _) => format!("{value}  (override)"),
            (None, Some(default)) => format!("{default}  (type default)"),
            (None, None) => String::from("-"),
        };
        match self {
            Self::Room => format!("{:02}", room.room_id),
            Self::RoomWidth => room.size.width_viewports.to_string(),
            Self::RoomHeight => room.size.height_h.to_string(),
            Self::Background => room.background_profile.clone(),
            Self::Description | Self::Label => {
                self.text(room, item).unwrap_or_else(|| String::from("-"))
            }
            Self::Surface => match item {
                Some(EditorItem::Terrain(index)) => room
                    .terrain
                    .get(index)
                    .map_or_else(String::new, |block| block.surface.to_string()),
                _ => String::new(),
            },
            Self::EnemyType => spawn.map_or_else(String::new, |spawn| spawn.type_id.to_string()),
            Self::X => origin.map_or_else(String::new, |origin| origin.x.to_string()),
            Self::Y => origin.map_or_else(String::new, |origin| origin.y.to_string()),
            Self::Width => rect.map_or_else(String::new, |rect| rect.width.to_string()),
            Self::Height => rect.map_or_else(String::new, |rect| rect.height.to_string()),
            Self::Health => stat(
                overrides.and_then(|overrides| overrides.health),
                defaults.map(|enemy| enemy.health),
            ),
            Self::Speed => stat(
                overrides.and_then(|overrides| overrides.movement_speed_h_s),
                defaults.map(|enemy| enemy.movement_speed_h_s),
            ),
            Self::Damage => stat(
                overrides.and_then(|overrides| overrides.attack_damage),
                defaults.and_then(EnemyTypeConfig::attack_damage),
            ),
        }
    }

    /// Steps the field by `steps` (negative to decrease) in the edited room.
    pub(super) fn adjust(
        self,
        room: &mut RoomConfig,
        item: Option<EditorItem>,
        data: &GameData,
        steps: f32,
    ) {
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
            Self::RoomWidth => {
                let limits = &data.gameplay.room_size_limits.width_viewports;
                let width = f32::from(room.size.width_viewports) + steps.signum();
                room.size.width_viewports = width.clamp(limits.min, limits.max) as u8;
            }
            Self::RoomHeight => {
                let limits = &data.gameplay.room_size_limits.height_viewports;
                let viewport_h = data.gameplay.viewport.height_h;
                room.size.height_h = (room.size.height_h + steps * 0.5)
                    .clamp(limits.min * viewport_h, limits.max * viewport_h);
            }
            Self::Background => {
                let profiles = data.backgrounds.profiles.keys().cloned().collect();
                if let Some(profile) = cycle(profiles, &room.background_profile) {
                    room.background_profile = profile;
                }
            }
            Self::Surface => {
                if let Some(EditorItem::Terrain(index)) = item
                    && let Some(block) = room.terrain.get_mut(index)
                {
                    block.surface = match block.surface {
                        PlatformMaterial::Soil => PlatformMaterial::Stone,
                        PlatformMaterial::Stone | PlatformMaterial::Wood => PlatformMaterial::Soil,
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
            Self::X | Self::Y => {
                if let Some(item) = item
                    && let Some(origin) = item.origin(room)
                {
                    let step = Vec2::new(
                        if self == Self::X { steps * SNAP_H } else { 0.0 },
                        if self == Self::Y { steps * SNAP_H } else { 0.0 },
                    );
                    item.set_origin(room, origin + step);
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
            Self::Health | Self::Speed | Self::Damage => {
                let Some(EditorItem::Enemy(index)) = item else {
                    return;
                };
                let defaults = room
                    .enemy_spawns
                    .get(index)
                    .and_then(|spawn| data.enemies.types.get(&spawn.type_id));
                let Some(spawn) = room.enemy_spawns.get_mut(index) else {
                    return;
                };
                let overrides = spawn
                    .stat_overrides
                    .get_or_insert_with(EnemyStatOverrides::default);
                let (value, default, step, min) = match self {
                    Self::Health => (
                        &mut overrides.health,
                        defaults.map(|enemy| enemy.health),
                        5.0,
                        1.0,
                    ),
                    Self::Speed => (
                        &mut overrides.movement_speed_h_s,
                        defaults.map(|enemy| enemy.movement_speed_h_s),
                        0.25,
                        0.0,
                    ),
                    _ => (
                        &mut overrides.attack_damage,
                        defaults.and_then(EnemyTypeConfig::attack_damage),
                        1.0,
                        0.0,
                    ),
                };
                let current = value.or(default).unwrap_or(0.0);
                *value = Some((current + steps * step).max(min));
            }
            Self::Room | Self::Description | Self::Label => {}
        }
    }

    /// Removes a stat override or a text field.
    pub(super) fn clear(self, room: &mut RoomConfig, item: Option<EditorItem>) {
        if self.is_text() {
            self.set_text(room, item, "");
            return;
        }
        let Some(EditorItem::Enemy(index)) = item else {
            return;
        };
        let Some(spawn) = room.enemy_spawns.get_mut(index) else {
            return;
        };
        if let Some(overrides) = &mut spawn.stat_overrides {
            match self {
                Self::Health => overrides.health = None,
                Self::Speed => overrides.movement_speed_h_s = None,
                Self::Damage => overrides.attack_damage = None,
                _ => {}
            }
        }
        if spawn.stat_overrides == Some(EnemyStatOverrides::default()) {
            spawn.stat_overrides = None;
        }
    }
}
