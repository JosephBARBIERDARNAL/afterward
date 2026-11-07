//! Copy and paste: the selected platforms, decorations, and enemies, kept
//! relative to their lower-left corner so they paste at the cursor of any
//! room.

use super::*;
use crate::data::{DecorationConfig, EnemySpawnConfig, PlatformConfig};

#[derive(Clone)]
pub(super) struct Clipboard {
    /// Where the copied items' lower-left corner was in their room.
    pub(super) origin: Vec2,
    platforms: Vec<PlatformConfig>,
    decorations: Vec<DecorationConfig>,
    enemies: Vec<EnemySpawnConfig>,
}

impl Clipboard {
    /// Copies `items` of `room`; the player spawn and the exit are left out.
    /// `None` when nothing copyable is selected.
    pub(super) fn copy(room: &RoomConfig, items: &[EditorItem]) -> Option<Self> {
        let anchor = items
            .iter()
            .filter(|item| !matches!(item, EditorItem::PlayerSpawn | EditorItem::Exit))
            .filter_map(|item| item.bounds(room))
            .map(|rect| Vec2::new(rect.x, rect.y))
            .reduce(Vec2::min)?;
        let mut clipboard = Self {
            origin: anchor,
            platforms: Vec::new(),
            decorations: Vec::new(),
            enemies: Vec::new(),
        };
        for item in items {
            match *item {
                EditorItem::Platform(index) => {
                    clipboard
                        .platforms
                        .extend(room.platforms.get(index).cloned());
                }
                EditorItem::Decoration(index) => {
                    clipboard
                        .decorations
                        .extend(room.decorations.get(index).cloned());
                }
                EditorItem::Enemy(index) => {
                    clipboard
                        .enemies
                        .extend(room.enemy_spawns.get(index).cloned());
                }
                EditorItem::PlayerSpawn | EditorItem::Exit => {}
            }
        }
        clipboard.shift(-anchor);
        Some(clipboard)
    }

    pub(super) fn len(&self) -> usize {
        self.platforms.len() + self.decorations.len() + self.enemies.len()
    }

    /// Adds a copy of the clipboard to `room` with its lower-left corner at
    /// `at`, and returns the pasted items.
    pub(super) fn paste(&self, room: &mut RoomConfig, at: Vec2) -> Vec<EditorItem> {
        let mut pasted = self.clone();
        pasted.shift(at);
        let start = (
            room.platforms.len(),
            room.decorations.len(),
            room.enemy_spawns.len(),
        );
        room.platforms.extend(pasted.platforms);
        room.decorations.extend(pasted.decorations);
        room.enemy_spawns.extend(pasted.enemies);
        (start.0..room.platforms.len())
            .map(EditorItem::Platform)
            .chain((start.1..room.decorations.len()).map(EditorItem::Decoration))
            .chain((start.2..room.enemy_spawns.len()).map(EditorItem::Enemy))
            .collect()
    }

    fn shift(&mut self, offset: Vec2) {
        let rects = self
            .platforms
            .iter_mut()
            .map(|block| &mut block.rect_h)
            .chain(
                self.decorations
                    .iter_mut()
                    .map(|decoration| &mut decoration.rect_h),
            );
        for rect in rects {
            rect.x += offset.x;
            rect.y += offset.y;
        }
        for spawn in &mut self.enemies {
            spawn.position_h.x_h += offset.x;
            spawn.position_h.y_h += offset.y;
        }
    }
}
