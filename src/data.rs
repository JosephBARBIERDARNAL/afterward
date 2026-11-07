mod backgrounds;
mod bonuses;
mod enemies;
mod gameplay;
mod player_animations;
mod room_yaml;
mod rooms;
mod terrain;

pub use backgrounds::*;
pub use bonuses::*;
pub use enemies::*;
pub use gameplay::*;
pub use player_animations::*;
pub use rooms::*;
pub use terrain::*;

use std::{
    collections::BTreeMap,
    error::Error,
    fmt::{self, Display, Formatter},
    fs,
    path::{Path, PathBuf},
};

use bevy::prelude::Resource;
use serde::de::DeserializeOwned;

#[derive(Resource, Debug)]
pub struct GameData {
    /// The `src/assets` folder the data was read from; the debug editor saves rooms there.
    pub asset_root: PathBuf,
    pub gameplay: GameplayConfig,
    pub player_animations: PlayerAnimations,
    pub enemies: EnemyTypes,
    pub backgrounds: BackgroundProfiles,
    pub bonuses: BonusesConfig,
    pub terrain: TerrainMaterials,
    pub rooms: BTreeMap<u16, RoomConfig>,
}

impl GameData {
    pub fn load(asset_root: &Path) -> Result<Self, Box<dyn Error + Send + Sync>> {
        let gameplay = read_yaml(&asset_root.join("config/gameplay.yaml"))?;
        let player_animations = read_yaml(&asset_root.join("player/animations.yaml"))?;
        let enemies = read_yaml(&asset_root.join("enemies/enemy_types.yaml"))?;
        let backgrounds = read_yaml(&asset_root.join("background/background_profiles.yaml"))?;
        let bonuses = read_yaml(&asset_root.join("config/bonuses.yaml"))?;
        let terrain = read_yaml(&asset_root.join("terrain/materials.yaml"))?;
        let mut rooms = BTreeMap::new();

        // Room 0 is the run's empty opening room; the chapter proper is 1..=10.
        for room_id in 0..=10 {
            let room: RoomConfig = read_yaml(&RoomConfig::file_path(asset_root, room_id))?;
            rooms.insert(room.room_id, room);
        }

        let data = Self {
            asset_root: asset_root.to_path_buf(),
            gameplay,
            player_animations,
            enemies,
            backgrounds,
            bonuses,
            terrain,
            rooms,
        };
        data.validate()?;
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

    /// Checks the loaded data against the game's rules; the debug editor
    /// re-runs it before saving or playtesting an edited room.
    pub(crate) fn validate(&self) -> Result<(), InvalidGameData> {
        let asset_root = &self.asset_root;
        if !(self.gameplay.viewport.height_h.is_finite() && self.gameplay.viewport.height_h > 0.0)
            || self.gameplay.viewport.aspect_ratio.width != 16
            || self.gameplay.viewport.aspect_ratio.height != 9
        {
            return Err(InvalidGameData::from(
                "the gameplay viewport must be fixed at 16:9 with a positive height",
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
        let door = &self.gameplay.room_flow.exit_door;
        if door.frames < 3
            || door.frame_size_px.width == 0
            || door.frame_size_px.height == 0
            || door.height_h.is_nan()
            || door.height_h <= 0.0
        {
            return Err(InvalidGameData::from(
                "the exit door needs locked, unlocked, and opening frames, and a positive size",
            ));
        }
        if !asset_root.join(&door.sheet).is_file() {
            return Err(InvalidGameData::from(format!(
                "missing exit door sheet: {}",
                door.sheet
            )));
        }
        let audio = &self.gameplay.audio;
        if audio.footstep_interval_s <= 0.0
            || !(0.0..=1.0).contains(&audio.paused_music_volume_multiplier)
            || !(0.0..=1.0).contains(&audio.room_clear_music_volume_multiplier)
            || !(0.0..=1.0).contains(&audio.paused_music_muffle)
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
        let positive = |width: f32, height: f32| {
            width.is_finite() && height.is_finite() && width > 0.0 && height > 0.0
        };
        let hurtbox = &self.gameplay.player.hurtbox_h;
        if !positive(hurtbox.width, hurtbox.height) {
            return Err(InvalidGameData::from(
                "the player hurtbox needs a positive size",
            ));
        }
        for (type_id, enemy) in &self.enemies.types {
            if !positive(enemy.hurtbox_h.width, enemy.hurtbox_h.height) {
                return Err(InvalidGameData::from(format!(
                    "enemy {type_id} needs a hurtbox with a positive size"
                )));
            }
            for attack in enemy.attack.iter().chain(&enemy.attacks) {
                let sized = if attack.kind.contains("projectile") {
                    attack
                        .projectile_size_h
                        .is_some_and(|size| positive(size.width, size.height))
                } else {
                    attack.hitbox_h.is_some_and(|hitbox| {
                        positive(hitbox.width, hitbox.height)
                            && hitbox.x.is_finite()
                            && hitbox.y.is_finite()
                    })
                };
                if !sized {
                    return Err(InvalidGameData::from(format!(
                        "enemy {type_id} attack {} needs a positive {}",
                        attack.kind,
                        if attack.kind.contains("projectile") {
                            "projectile_size_h"
                        } else {
                            "hitbox_h"
                        }
                    )));
                }
            }
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

        if self.rooms.len() != 11 {
            return Err(InvalidGameData::from(
                "the Chapter 1 data must contain the opening room 0 and rooms 1 through 10",
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

        let viewport_height_h = self.gameplay.viewport.height_h;
        let player = &self.gameplay.player;
        let walls = &player.walls;
        if !(self.gameplay.camera.vertical_dead_zone_h >= 0.0
            && self.gameplay.camera.vertical_dead_zone_h < viewport_height_h / 2.0)
        {
            return Err(InvalidGameData::from(
                "the camera dead zone must fit inside half the viewport",
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
        // Rooms are free-form: only what the game can't run or draw without is
        // checked (consistent IDs, known references, a room at least one
        // viewport in size, blocks with a positive size).
        for (room_id, room) in &self.rooms {
            if room.room_id != *room_id || room.next_room_id != room.room_id + 1 {
                return Err(InvalidGameData::from(format!(
                    "room {room_id} has an inconsistent room_id or next_room_id"
                )));
            }
            if room.size.width_viewports == 0
                || room.size.height_h.is_nan()
                || room.size.height_h < viewport_height_h
            {
                return Err(InvalidGameData::from(format!(
                    "room {room_id} must be at least one viewport wide and tall"
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
            let rects = room
                .terrain
                .iter()
                .chain(&room.platforms)
                .map(|block| block.rect_h)
                .chain(room.decorations.iter().map(|decoration| decoration.rect_h));
            for rect in rects {
                if rect.width.is_nan()
                    || rect.height.is_nan()
                    || rect.width <= 0.0
                    || rect.height <= 0.0
                {
                    return Err(InvalidGameData::from(format!(
                        "room {room_id} blocks and decorations need a positive size"
                    )));
                }
            }
            let materials = room
                .terrain
                .iter()
                .chain(&room.platforms)
                .map(|block| &block.material)
                .chain(
                    room.decorations
                        .iter()
                        .map(|decoration| &decoration.material),
                );
            for material in materials {
                if !self.terrain.materials.contains_key(material) {
                    return Err(InvalidGameData::from(format!(
                        "room {room_id} references unknown terrain material {material}"
                    )));
                }
            }
            for spawn in &room.enemy_spawns {
                if !self.enemies.types.contains_key(&spawn.type_id) {
                    return Err(InvalidGameData::from(format!(
                        "room {room_id} references unknown enemy type {}",
                        spawn.type_id
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

        let terrain = &self.terrain;
        let edges = &terrain.edges;
        let layers = &terrain.decoration_layers;
        if terrain.texture_px == 0
            || !(terrain.texture_tile_h.is_finite() && terrain.texture_tile_h > 0.0)
            || [edges.outline_px, edges.highlight_px]
                .iter()
                .any(|thickness| !(thickness.is_finite() && *thickness >= 0.0))
            || [
                edges.outline_alpha,
                edges.highlight_alpha,
                layers.background.alpha,
                layers.foreground.alpha,
            ]
            .iter()
            .any(|alpha| !(0.0..=1.0).contains(alpha))
        {
            return Err(InvalidGameData::from(
                "terrain textures need a positive size, edges a nonnegative thickness, \
                 and every alpha must be within 0..=1",
            ));
        }
        for (material_id, material) in &terrain.materials {
            let cap = material.top.as_ref();
            if cap.is_some_and(|cap| !(cap.depth_px.is_finite() && cap.depth_px > 0.0)) {
                return Err(InvalidGameData::from(format!(
                    "terrain material {material_id} needs a positive top depth"
                )));
            }
            let textures = [Some(&material.texture), cap.map(|cap| &cap.texture)];
            for texture in textures.into_iter().flatten() {
                if !asset_root.join(texture).is_file() {
                    return Err(InvalidGameData::from(format!(
                        "missing terrain texture: {texture}"
                    )));
                }
            }
        }

        let recorded = &self.gameplay.audio.recorded_effects;
        let sounds = [
            &recorded.gun_shot,
            &recorded.reload,
            &recorded.door_open,
            &recorded.damage_neutral,
            &recorded.damage_metal,
            &recorded.damage_metal_armor,
            &recorded.bonus_chosen,
            &recorded.bonus_reroll,
        ]
        .into_iter()
        .chain(&self.gameplay.audio.music_tracks);
        for sound in sounds {
            if !asset_root.join(&sound.asset).is_file() {
                return Err(InvalidGameData::from(format!(
                    "missing sound asset: {}",
                    sound.asset
                )));
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
pub(crate) struct InvalidGameData(String);

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

        assert_eq!(data.rooms.len(), 11);
        assert!(
            data.rooms[&0].enemy_spawns.is_empty(),
            "the opening room holds no enemies"
        );
        assert_eq!(data.bonuses.bonuses.len(), 21);
        assert_eq!(data.rooms[&10].enemy_spawns[0].type_id, "evil_wizard");
        assert!(
            data.terrain.materials.len() >= 64,
            "every texture of the pack is a material"
        );
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
