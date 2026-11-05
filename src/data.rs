mod backgrounds;
mod bonuses;
mod enemies;
mod gameplay;
mod player_animations;
mod rooms;

pub use backgrounds::*;
pub use bonuses::*;
pub use enemies::*;
pub use gameplay::*;
pub use player_animations::*;
pub use rooms::*;

use std::{
    collections::BTreeMap,
    error::Error,
    fmt::{self, Display, Formatter},
    fs,
    path::Path,
};

use bevy::prelude::Resource;
use serde::de::DeserializeOwned;

#[derive(Resource, Debug)]
pub struct GameData {
    pub gameplay: GameplayConfig,
    pub player_animations: PlayerAnimations,
    pub enemies: EnemyTypes,
    pub backgrounds: BackgroundProfiles,
    pub bonuses: BonusesConfig,
    pub rooms: BTreeMap<u16, RoomConfig>,
}

impl GameData {
    pub fn load(asset_root: &Path) -> Result<Self, Box<dyn Error + Send + Sync>> {
        let gameplay = read_yaml(&asset_root.join("config/gameplay.yaml"))?;
        let player_animations = read_yaml(&asset_root.join("player/animations.yaml"))?;
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
            player_animations,
            enemies,
            backgrounds,
            bonuses,
            rooms,
        };
        data.validate(asset_root)?;
        Ok(data)
    }

    pub(crate) fn bonus_effect(&self, bonus_id: &str, effect_id: &str) -> f32 {
        self.bonuses
            .bonuses
            .iter()
            .find(|bonus| bonus.id == bonus_id)
            .and_then(|bonus| bonus.effects.get(effect_id))
            .and_then(|value| BonusEffect::try_from(value).ok())
            .map(f32::from)
            .unwrap_or_default()
    }

    fn validate(&self, asset_root: &Path) -> Result<(), InvalidGameData> {
        if self.gameplay.viewport.height_h != 9.0
            || self.gameplay.viewport.aspect_ratio.width != 16
            || self.gameplay.viewport.aspect_ratio.height != 9
        {
            return Err(InvalidGameData::from(
                "the gameplay viewport must be fixed at 16:9 and 9 H",
            ));
        }
        let player = &self.gameplay.player;
        if [
            player.max_speed_h_s,
            player.air_max_speed_h_s,
            player.crouch_speed_h_s,
        ]
        .iter()
        .any(|speed| !speed.is_finite() || *speed <= 0.0)
            || !(0.0..=1.0).contains(&player.cover_damage_taken_multiplier)
            || !(player.death_hold_s.is_finite() && player.death_hold_s >= 0.0)
            || !(player.walls.latch_s.is_finite() && player.walls.latch_s >= 0.0)
        {
            return Err(InvalidGameData::from(
                "player speeds must be positive, the cover multiplier within 0..=1, \
                 and the death hold and wall latch nonnegative",
            ));
        }
        let gun = &player.gun;
        let bursts = [&gun.muzzle_flash, &gun.impact_burst];
        if gun.magazine_size == 0
            || [
                gun.damage,
                gun.fire_interval_s,
                gun.reload_s,
                gun.bullet_speed_h_s,
                gun.bullet_range_h,
                gun.bullet_size_h.width,
                gun.bullet_size_h.height,
            ]
            .iter()
            .chain(
                bursts
                    .iter()
                    .flat_map(|burst| [&burst.radius_h, &burst.duration_s]),
            )
            .any(|value| !value.is_finite() || *value <= 0.0)
            || !(0.0..90.0).contains(&gun.diagonal_angle_degrees)
        {
            return Err(InvalidGameData::from(
                "the gun needs a magazine, positive damage, timings, bullet and burst sizes, \
                 and a diagonal angle below 90 degrees",
            ));
        }
        let animations = &self.player_animations;
        if animations.body_height_px <= 0.0
            || !(animations.body_height_h.is_finite() && animations.body_height_h > 0.0)
            || !(animations.body_height_px..=animations.frame_size_px.height as f32)
                .contains(&animations.feet_row_px)
            || !(animations.jog_speed_fraction > 0.0 && animations.jog_speed_fraction <= 1.0)
            || !(0.0..=1.0).contains(&animations.wounded_health_fraction)
            || !(animations.idle_fidget_after_s.is_finite() && animations.idle_fidget_after_s > 0.0)
            || !(animations.aim_hold_s.is_finite() && animations.aim_hold_s >= 0.0)
            || animations.run_shooting_shot_frame >= animations.clips.run_shooting.frames
        {
            return Err(InvalidGameData::from(
                "player sprite body heights, feet row, speed and health fractions, fidget delay, \
                 or run-shooting shot frame are out of range",
            ));
        }
        if Aim::ALL.iter().any(|aim| {
            let clips = animations.shooting.get(*aim);
            clips.shot.is_empty() || clips.shot.len() != clips.aim.len()
        }) {
            return Err(InvalidGameData::from(
                "each aim direction needs at least one shot clip and one aim pose per shot clip",
            ));
        }
        for (key, clip) in animations.keyed() {
            if clip.frames == 0 || !clip.fps.is_finite() || clip.fps <= 0.0 {
                return Err(InvalidGameData::from(format!(
                    "{key} needs at least one frame and a positive frame rate"
                )));
            }
            if !asset_root.join(&clip.sheet).is_file() {
                return Err(InvalidGameData::from(format!(
                    "missing player sprite sheet: {}",
                    clip.sheet
                )));
            }
        }
        let expected_jump_impulse =
            (2.0 * self.gameplay.player.gravity_h_s2 * self.gameplay.player.jump_height_h).sqrt();
        if (expected_jump_impulse - self.gameplay.player.jump_impulse_h_s).abs() > 0.05 {
            return Err(InvalidGameData::from(
                "jump height, gravity, and jump impulse must describe the same jump",
            ));
        }
        if !(0.0..=1.0).contains(&self.gameplay.player.jump_release_velocity_multiplier) {
            return Err(InvalidGameData::from(
                "the jump release velocity multiplier must be between 0 and 1",
            ));
        }
        if self.gameplay.room_flow.clear_animation_s <= 0.0
            || self.gameplay.room_flow.bonus_reroll_animation_s <= 0.0
            || self.gameplay.room_flow.boss_intro_duration_s <= 0.0
            || self.gameplay.room_flow.room_transition_duration_s <= 0.0
            || self.gameplay.room_flow.exit_side != "right"
        {
            return Err(InvalidGameData::from(
                "Chapter 1 room flow requires a positive clear animation and right-side exits",
            ));
        }
        let audio = &self.gameplay.audio;
        if audio.footstep_interval_s <= 0.0
            || !(0.0..=1.0).contains(&audio.paused_music_volume_multiplier)
            || !(0.0..=1.0).contains(&audio.room_clear_music_volume_multiplier)
            || !(0.0..=1.0).contains(&audio.chapter_one_music_gain)
            || !(0.0..=1.0).contains(&audio.chapter_one_ambience_gain)
        {
            return Err(InvalidGameData::from(
                "audio tuning values are out of range",
            ));
        }
        let controls = &self.gameplay.controls;
        let default_bindings = [
            &controls.move_left,
            &controls.move_right,
            &controls.move_down,
            &controls.navigate_up,
            &controls.navigate_down,
            &controls.jump,
            &controls.dash,
            &controls.shoot,
            &controls.aim_up,
            &controls.reload,
            &controls.cover,
            &controls.interact,
            &controls.pause,
            &controls.toggle_fullscreen,
        ];
        if default_bindings.iter().any(|binding| binding.is_empty()) {
            return Err(InvalidGameData::from(
                "each game action needs a default key binding",
            ));
        }

        let jumping = &self.enemies.jumping;
        if !jumping.air_speed_h_s.is_finite()
            || jumping.air_speed_h_s <= 0.0
            || !jumping.cooldown_s.is_finite()
            || jumping.cooldown_s < 0.0
            || !jumping.level_change_threshold_h.is_finite()
            || jumping.level_change_threshold_h <= 0.0
        {
            return Err(InvalidGameData::from(
                "enemy jumping tuning values must be finite and positive",
            ));
        }
        for enemy in self.enemies.types.values() {
            let jump_height_h = enemy.jump_height_h.unwrap_or(jumping.jump_height_h);
            if !jump_height_h.is_finite() || jump_height_h <= 0.0 {
                return Err(InvalidGameData::from(
                    "enemy jump heights must be finite and positive",
                ));
            }
            if enemy.procedural_movement.as_ref().is_some_and(|movement| {
                !movement.bob_height_h.is_finite()
                    || movement.bob_height_h < 0.0
                    || !movement.cycles_per_h.is_finite()
                    || movement.cycles_per_h <= 0.0
                    || !movement.lean_degrees.is_finite()
            }) {
                return Err(InvalidGameData::from(
                    "procedural enemy movement values must be finite and nonnegative",
                ));
            }
        }

        if self.rooms.len() != 10 {
            return Err(InvalidGameData::from(
                "the Chapter 1 data must contain rooms 1 through 10",
            ));
        }

        if self.bonuses.bonuses.len() != 21 {
            return Err(InvalidGameData::from(
                "the bonus pool must contain exactly 21 entries",
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
            return Err(InvalidGameData::from("bonus IDs must be unique"));
        }
        if self
            .bonuses
            .bonuses
            .iter()
            .any(|bonus| !self.bonuses.categories.contains_key(&bonus.category))
        {
            return Err(InvalidGameData::from(
                "a bonus references an unknown category",
            ));
        }

        let limits = &self.gameplay.room_size_limits;
        let viewport_height_h = self.gameplay.viewport.height_h;
        let player = &self.gameplay.player;
        let walls = &player.walls;
        if limits.width_viewports.min < 1.0
            || limits.width_viewports.min > limits.width_viewports.max
            || limits.height_viewports.min < 1.0
            || limits.height_viewports.min > limits.height_viewports.max
            || !(self.gameplay.camera.vertical_dead_zone_h >= 0.0
                && self.gameplay.camera.vertical_dead_zone_h < viewport_height_h / 2.0)
        {
            return Err(InvalidGameData::from(
                "room size limits must span at least one viewport, and the camera dead zone \
                 must fit inside half the viewport",
            ));
        }
        if [
            walls.slide_max_fall_speed_h_s,
            walls.jump_impulse_h_s,
            walls.jump_kick_speed_h_s,
            walls.ledge_climb_s,
            player.drop_through_s,
        ]
        .iter()
        .any(|value| !value.is_finite() || *value <= 0.0)
            || !(walls.jump_control_lock_s.is_finite() && walls.jump_control_lock_s >= 0.0)
            || !(walls.ledge_grab_reach_h.is_finite() && walls.ledge_grab_reach_h >= 0.0)
        {
            return Err(InvalidGameData::from(
                "wall movement and drop-through tuning values must be finite and positive",
            ));
        }
        let intro = &self.gameplay.room_flow.room_intro;
        let walk_in_distance_h = intro.player_walk_in_s * intro.player_walk_in_speed_h_s;
        for (room_id, room) in &self.rooms {
            let room_width_h = room.width_h(&self.gameplay);
            let room_height_h = room.size.height_h;
            if room.room_id != *room_id || room.chapter_id != 1 {
                return Err(InvalidGameData::from(format!(
                    "room {room_id} has an inconsistent ID"
                )));
            }
            let width_viewports = f32::from(room.size.width_viewports);
            let height_viewports = room_height_h / viewport_height_h;
            if !(limits.width_viewports.min..=limits.width_viewports.max).contains(&width_viewports)
                || !(limits.height_viewports.min..=limits.height_viewports.max)
                    .contains(&height_viewports)
            {
                return Err(InvalidGameData::from(format!(
                    "room {room_id} has invalid dimensions"
                )));
            }
            if room.next_room_id != room.room_id + 1 {
                return Err(InvalidGameData::from(format!(
                    "room {room_id} has an invalid next_room_id"
                )));
            }
            if !self
                .backgrounds
                .profiles
                .contains_key(&room.background_profile)
            {
                return Err(InvalidGameData::from(format!(
                    "room {room_id} references an unknown background profile"
                )));
            }
            if room.terrain.is_empty()
                || room.terrain.iter().chain(&room.platforms).any(|block| {
                    let rect = block.rect_h;
                    rect.width <= 0.0
                        || rect.height <= 0.0
                        || rect.x < 0.0
                        || rect.y < 0.0
                        || rect.right() > room_width_h
                        || rect.top() > room_height_h
                })
                || room
                    .platforms
                    .iter()
                    .any(|platform| platform.rect_h.height > 0.5)
            {
                return Err(InvalidGameData::from(format!(
                    "room {room_id} needs solid terrain, with every block inside the room and \
                     one-way platforms at most 0.5 H thick"
                )));
            }
            if !room.has_footing_at(room.player_spawn.x_h, room.player_spawn.y_h)
                || !room.has_footing_at(room.exit.x_h, room.exit.y_h)
            {
                return Err(InvalidGameData::from(format!(
                    "room {room_id} player spawn and exit must stand on a block top, clear of \
                     solid terrain"
                )));
            }
            // The entrance walk-in crosses this stretch, so it must be a doorway.
            let walk_in_start_h = room.player_spawn.x_h - walk_in_distance_h;
            if (0..=20).any(|step| {
                let x_h = walk_in_start_h + walk_in_distance_h * step as f32 / 20.0;
                !room.body_is_clear(x_h, room.player_spawn.y_h)
            }) {
                return Err(InvalidGameData::from(format!(
                    "room {room_id} needs a doorway clear of solid terrain where the player \
                     walks in"
                )));
            }

            if room_id < &10 {
                let unique_types = room
                    .enemy_spawns
                    .iter()
                    .map(|spawn| &*spawn.type_id)
                    .collect::<std::collections::BTreeSet<_>>();
                if !(5..=15).contains(&room.enemy_spawns.len()) || unique_types.len() < 2 {
                    return Err(InvalidGameData::from(format!(
                        "room {room_id} must have 5–15 spawns across at least two enemy types"
                    )));
                }
            } else if !room
                .enemy_spawns
                .iter()
                .any(|spawn| spawn.type_id == "evil_wizard")
            {
                return Err(InvalidGameData::from(
                    "room 10 must include the Evil Wizard boss",
                ));
            }

            for spawn in &room.enemy_spawns {
                if !self.enemies.types.contains_key(&spawn.type_id) {
                    return Err(InvalidGameData::from(format!(
                        "room {room_id} references unknown enemy type {}",
                        spawn.type_id
                    )));
                }
                if !room.has_footing_at(spawn.position_h.x_h, spawn.position_h.y_h) {
                    return Err(InvalidGameData::from(format!(
                        "room {room_id} enemy spawn at x = {} H must stand on a block top, \
                         clear of solid terrain",
                        spawn.position_h.x_h
                    )));
                }
            }
        }

        for profile in self.backgrounds.profiles.values() {
            let mut assets = vec![profile.backdrop.as_str()];
            assets.extend(profile.layers.iter().map(|layer| layer.asset.as_str()));
            for asset in assets {
                if !asset_root.join(asset).is_file() {
                    return Err(InvalidGameData::from(format!(
                        "missing background asset: {asset}"
                    )));
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

#[derive(Debug)]
struct InvalidGameData(String);

impl From<&str> for InvalidGameData {
    fn from(message: &str) -> Self {
        Self(message.to_string())
    }
}

impl From<String> for InvalidGameData {
    fn from(message: String) -> Self {
        Self(message)
    }
}

impl Display for InvalidGameData {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl Error for InvalidGameData {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chapter_one_yaml_loads_and_matches_the_room_graph() {
        let asset_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/assets");
        let data = GameData::load(&asset_root).expect("Chapter 1 YAML should be valid");

        assert_eq!(data.rooms.len(), 10);
        assert_eq!(data.bonuses.bonuses.len(), 21);
        assert_eq!(data.rooms[&10].enemy_spawns[0].type_id, "evil_wizard");
        for room in data.rooms.values() {
            assert!(
                room.terrain
                    .iter()
                    .all(|block| block.surface != PlatformMaterial::Wood),
                "solid terrain is soil or stone"
            );
            assert!(
                room.platforms
                    .iter()
                    .all(|platform| platform.surface == PlatformMaterial::Wood),
                "one-way platforms are wooden planks"
            );
        }
        assert_eq!(
            data.gameplay.player.max_speed_h_s,
            data.gameplay.player.air_max_speed_h_s
        );
    }

    #[test]
    fn bonus_category_icons_use_ascii_glyphs_available_in_the_game_font() {
        for icon in [
            BonusCategoryIcon::CrossedSwords,
            BonusCategoryIcon::Shield,
            BonusCategoryIcon::Wing,
            BonusCategoryIcon::Light,
            BonusCategoryIcon::Knot,
        ] {
            assert!(icon.to_string().is_ascii());
        }
    }
}
