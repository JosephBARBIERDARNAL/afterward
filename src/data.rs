mod models;
pub use models::*;
use std::{
    collections::BTreeMap,
    error::Error,
    fs,
    io::{Error as IoError, ErrorKind},
    path::Path,
};

use bevy::prelude::Resource;
use serde::de::DeserializeOwned;

#[derive(Resource, Debug)]
pub struct GameData {
    pub gameplay: GameplayConfig,
    pub enemies: EnemyTypes,
    pub backgrounds: BackgroundProfiles,
    pub bonuses: BonusesConfig,
    pub rooms: BTreeMap<u16, RoomConfig>,
}

impl GameData {
    pub fn load(asset_root: &Path) -> Result<Self, Box<dyn Error + Send + Sync>> {
        let gameplay = read_yaml(&asset_root.join("config/gameplay.yaml"))?;
        let enemies = read_yaml(&asset_root.join("enemies/enemy_types.yaml"))?;
        let backgrounds = read_yaml(&asset_root.join("background/background_profiles.yaml"))?;
        let bonuses = read_yaml(&asset_root.join("config/bonuses.yaml"))?;
        let mut rooms = BTreeMap::new();

        for room_id in 1..=10 {
            let path = asset_root.join(format!("rooms/room_{room_id:03}.yaml"));
            let room: RoomConfig = read_yaml(&path)?;
            rooms.insert(room.room_id, room);
        }

        let data = Self {
            gameplay,
            enemies,
            backgrounds,
            bonuses,
            rooms,
        };
        data.validate(asset_root)?;
        Ok(data)
    }

    fn validate(&self, asset_root: &Path) -> Result<(), Box<dyn Error + Send + Sync>> {
        if self.gameplay.viewport.height_h != 9.0
            || self.gameplay.viewport.aspect_ratio.width != 16
            || self.gameplay.viewport.aspect_ratio.height != 9
        {
            return Err(invalid_data(
                "the gameplay viewport must be fixed at 16:9 and 9 H",
            ));
        }
        let expected_jump_impulse =
            (2.0 * self.gameplay.player.gravity_h_s2 * self.gameplay.player.jump_height_h).sqrt();
        if (expected_jump_impulse - self.gameplay.player.jump_impulse_h_s).abs() > 0.05 {
            return Err(invalid_data(
                "jump height, gravity, and jump impulse must describe the same jump",
            ));
        }
        if self.gameplay.room_flow.clear_animation_s <= 0.0
            || self.gameplay.room_flow.bonus_reroll_animation_s <= 0.0
            || self.gameplay.room_flow.boss_intro_duration_s <= 0.0
            || self.gameplay.room_flow.room_transition_duration_s <= 0.0
            || self.gameplay.room_flow.exit_side != "right"
        {
            return Err(invalid_data(
                "Chapter 1 room flow requires a positive clear animation and right-side exits",
            ));
        }
        let audio = &self.gameplay.audio;
        if audio.footstep_interval_s <= 0.0
            || !(0.0..=1.0).contains(&audio.paused_music_volume_multiplier)
            || !(0.0..=1.0).contains(&audio.room_clear_music_volume_multiplier)
            || !(0.0..=1.0).contains(&audio.chapter_one_music_gain)
        {
            return Err(invalid_data("audio tuning values are out of range"));
        }
        let controls = &self.gameplay.controls;
        let default_bindings = [
            &controls.move_left,
            &controls.move_right,
            &controls.navigate_up,
            &controls.navigate_down,
            &controls.jump,
            &controls.dash,
            &controls.sword,
            &controls.gun,
            &controls.interact,
            &controls.pause,
        ];
        if default_bindings.iter().any(|binding| binding.is_empty()) {
            return Err(invalid_data("each game action needs a default key binding"));
        }

        if self.rooms.len() != 10 {
            return Err(invalid_data(
                "the Chapter 1 data must contain rooms 1 through 10",
            ));
        }

        if self.bonuses.bonuses.len() != 20 {
            return Err(invalid_data(
                "the bonus pool must contain exactly 20 entries",
            ));
        }

        let mut bonus_ids = self
            .bonuses
            .bonuses
            .iter()
            .map(|bonus| bonus.id.as_str())
            .collect::<Vec<_>>();
        bonus_ids.sort_unstable();
        bonus_ids.dedup();
        if bonus_ids.len() != self.bonuses.bonuses.len() {
            return Err(invalid_data("bonus IDs must be unique"));
        }
        if self
            .bonuses
            .bonuses
            .iter()
            .any(|bonus| !self.bonuses.categories.contains_key(&bonus.category))
        {
            return Err(invalid_data("a bonus references an unknown category"));
        }

        for (room_id, room) in &self.rooms {
            let room_width_h = f32::from(room.size.width_viewports) * 16.0;
            if room.room_id != *room_id || room.chapter_id != 1 {
                return Err(invalid_data(format!(
                    "room {room_id} has an inconsistent ID"
                )));
            }
            if !(2..=4).contains(&room.size.width_viewports) || room.size.height_h != 9.0 {
                return Err(invalid_data(format!(
                    "room {room_id} has invalid dimensions"
                )));
            }
            if room.next_room_id != room.room_id + 1 {
                return Err(invalid_data(format!(
                    "room {room_id} has an invalid next_room_id"
                )));
            }
            if !self
                .backgrounds
                .profiles
                .contains_key(&room.background_profile)
            {
                return Err(invalid_data(format!(
                    "room {room_id} references an unknown background profile"
                )));
            }
            if !(0.0..=room_width_h).contains(&room.player_spawn.x_h)
                || !(0.0..=room_width_h).contains(&room.exit.x_h)
            {
                return Err(invalid_data(format!(
                    "room {room_id} has an out-of-bounds spawn or exit"
                )));
            }

            if room_id < &10 {
                let unique_types = room
                    .enemy_spawns
                    .iter()
                    .map(|spawn| spawn.type_id.as_str())
                    .collect::<std::collections::BTreeSet<_>>();
                if !(5..=15).contains(&room.enemy_spawns.len()) || unique_types.len() < 2 {
                    return Err(invalid_data(format!(
                        "room {room_id} must have 5–15 spawns across at least two enemy types"
                    )));
                }
            } else if !room
                .enemy_spawns
                .iter()
                .any(|spawn| spawn.type_id == "warrior")
            {
                return Err(invalid_data("room 10 must include the Warrior boss"));
            }

            for spawn in &room.enemy_spawns {
                if !self.enemies.types.contains_key(&spawn.type_id) {
                    return Err(invalid_data(format!(
                        "room {room_id} references unknown enemy type {}",
                        spawn.type_id
                    )));
                }
                if !(0.0..=room_width_h).contains(&spawn.position_h.x_h) {
                    return Err(invalid_data(format!(
                        "room {room_id} has an out-of-bounds enemy spawn"
                    )));
                }
            }

            for platform in &room.platforms {
                let half_width = platform.size_h.width_h / 2.0;
                if platform.size_h.width_h <= 0.0
                    || platform.size_h.height_h <= 0.0
                    || platform.position_h.x_h - half_width < 0.0
                    || platform.position_h.x_h + half_width > room_width_h
                {
                    return Err(invalid_data(format!(
                        "room {room_id} has an invalid platform"
                    )));
                }
            }
        }

        for profile in self.backgrounds.profiles.values() {
            let mut assets = vec![profile.backdrop.as_str()];
            assets.extend(profile.layers.iter().map(|layer| layer.asset.as_str()));
            for asset in assets {
                if !asset_root.join(asset).is_file() {
                    return Err(invalid_data(format!("missing background asset: {asset}")));
                }
            }
        }

        Ok(())
    }
}

fn read_yaml<T: DeserializeOwned>(path: &Path) -> Result<T, Box<dyn Error + Send + Sync>> {
    let source = fs::read_to_string(path)?;
    let value = yaml_serde::from_str(&source)?;
    Ok(value)
}

fn invalid_data(message: impl Into<String>) -> Box<dyn Error + Send + Sync> {
    Box::new(IoError::new(ErrorKind::InvalidData, message.into()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chapter_one_yaml_loads_and_matches_the_room_graph() {
        let asset_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/assets");
        let data = GameData::load(&asset_root).expect("Chapter 1 YAML should be valid");

        assert_eq!(data.rooms.len(), 10);
        assert_eq!(data.bonuses.bonuses.len(), 20);
        assert_eq!(data.rooms[&10].enemy_spawns[0].type_id, "warrior");
    }
}
