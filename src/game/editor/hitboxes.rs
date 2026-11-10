//! Hitbox editing: the player's hurtbox and each enemy type's hurtbox, attack
//! hitboxes, and projectile sizes — and, riding with them, the size the
//! player is drawn at. They live in `config/gameplay.yaml`,
//! `enemies/enemy_types.yaml` and `player/animations.yaml`, not in a room, so
//! an edit applies to every room. Saving rewrites only their lines, which
//! keeps the files' comments.

use super::*;
use crate::data::{EnemyTypeConfig, EnemyTypes, GameplayConfig, PlayerAnimations, RectH, SizeH};
use std::{fs, path::Path};

/// Every hitbox the editor can change: an undo step, and the saved state
/// that tells whether there are unsaved hitbox edits.
#[derive(Clone, PartialEq)]
pub(super) struct Hitboxes {
    player: SizeH,
    /// How tall the player's body is drawn, in H (`body_height_h`).
    player_body_height_h: f32,
    enemies: BTreeMap<EnemyTypeId, EnemyHitboxes>,
}

#[derive(Clone, PartialEq)]
struct EnemyHitboxes {
    hurtbox: SizeH,
    /// Each attack's strike hitbox and projectile size, in file order.
    attacks: Vec<(Option<RectH>, Option<SizeH>)>,
}

impl Hitboxes {
    pub(super) fn of(data: &GameData) -> Self {
        Self {
            player: data.gameplay.player.hurtbox_h,
            player_body_height_h: data.player_animations.body_height_h,
            enemies: Self::enemies_of(&data.enemies),
        }
    }

    fn enemies_of(enemies: &EnemyTypes) -> BTreeMap<EnemyTypeId, EnemyHitboxes> {
        enemies
            .types
            .iter()
            .map(|(type_id, enemy)| {
                let attacks = enemy
                    .attack
                    .iter()
                    .chain(&enemy.attacks)
                    .map(|attack| (attack.hitbox_h, attack.projectile_size_h))
                    .collect();
                let hitboxes = EnemyHitboxes {
                    hurtbox: enemy.hurtbox_h,
                    attacks,
                };
                (type_id.clone(), hitboxes)
            })
            .collect()
    }

    /// Puts these hitboxes into `data`.
    pub(super) fn apply(&self, data: &mut GameData) {
        data.gameplay.player.hurtbox_h = self.player;
        data.player_animations.body_height_h = self.player_body_height_h;
        for (type_id, enemy) in &mut data.enemies.types {
            let Some(saved) = self.enemies.get(type_id) else {
                continue;
            };
            enemy.hurtbox_h = saved.hurtbox;
            let attacks = enemy.attack.iter_mut().chain(enemy.attacks.iter_mut());
            for (attack, (hitbox, projectile_size)) in attacks.zip(&saved.attacks) {
                attack.hitbox_h = *hitbox;
                attack.projectile_size_h = *projectile_size;
            }
        }
    }

    /// Writes these hitboxes into the YAML files under `asset_root`,
    /// checking that each rewritten file reads back as these hitboxes.
    pub(super) fn save(&self, asset_root: &Path) -> Result<(), String> {
        let gameplay_path = asset_root.join("config/gameplay.yaml");
        let enemies_path = asset_root.join("enemies/enemy_types.yaml");
        let animations_path = asset_root.join("player/animations.yaml");
        let read = |path: &Path| {
            fs::read_to_string(path).map_err(|error| format!("{}: {error}", path.display()))
        };
        let gameplay = self.rewrite_gameplay(&read(&gameplay_path)?);
        let enemies = self.rewrite_enemy_types(&read(&enemies_path)?);
        // `animations.yaml` with its one `body_height_h` line replaced.
        let animations = Self::rewrite_lines(&read(&animations_path)?, |indent, key| {
            key.starts_with("body_height_h:")
                .then(|| format!("{indent}body_height_h: {}", self.player_body_height_h))
        });
        let player = yaml_serde::from_str::<GameplayConfig>(&gameplay)
            .map(|gameplay| gameplay.player.hurtbox_h)
            .map_err(|error| error.to_string())?;
        let enemy_hitboxes = yaml_serde::from_str::<EnemyTypes>(&enemies)
            .map(|enemies| Self::enemies_of(&enemies))
            .map_err(|error| error.to_string())?;
        let body_height_h = yaml_serde::from_str::<PlayerAnimations>(&animations)
            .map(|animations| animations.body_height_h)
            .map_err(|error| error.to_string())?;
        if player != self.player
            || body_height_h != self.player_body_height_h
            || enemy_hitboxes != self.enemies
        {
            return Err(String::from(
                "a hitbox line is missing from the YAML files, so it can't be rewritten",
            ));
        }
        for (path, text) in [
            (gameplay_path, gameplay),
            (enemies_path, enemies),
            (animations_path, animations),
        ] {
            fs::write(&path, text).map_err(|error| format!("{}: {error}", path.display()))?;
        }
        Ok(())
    }

    /// `gameplay.yaml` with its one `hurtbox_h` line (the player's) replaced.
    fn rewrite_gameplay(&self, source: &str) -> String {
        Self::rewrite_lines(source, |indent, key| {
            key.starts_with("hurtbox_h:")
                .then(|| format!("{indent}hurtbox_h: {}", self.player))
        })
    }

    /// `enemy_types.yaml` with each type's `hurtbox_h` and each attack's
    /// `hitbox_h` and `projectile_size_h` lines replaced. Types are the
    /// two-space-indented keys; each `kind:` line starts the next attack.
    fn rewrite_enemy_types(&self, source: &str) -> String {
        let mut enemy = None;
        let mut attack = None;
        Self::rewrite_lines(source, |indent, key| {
            if indent == "  "
                && let Some(type_id) = key.strip_suffix(':')
                && let Some(hitboxes) = self.enemies.get(type_id)
            {
                enemy = Some(hitboxes);
                attack = None;
            }
            if key.starts_with("kind:") {
                attack = Some(attack.map_or(0, |index| index + 1));
            }
            let enemy = enemy?;
            let strike = attack.and_then(|index| enemy.attacks.get(index));
            match key.split(':').next() {
                Some("hurtbox_h") => Some(format!("{indent}hurtbox_h: {}", enemy.hurtbox)),
                Some("hitbox_h") => strike
                    .and_then(|(hitbox, _)| *hitbox)
                    .map(|hitbox| format!("{indent}hitbox_h: {hitbox}")),
                Some("projectile_size_h") => strike
                    .and_then(|(_, size)| *size)
                    .map(|size| format!("{indent}projectile_size_h: {size}")),
                _ => None,
            }
        })
    }

    /// `source` with each line replaced by what `replace` returns for it, given
    /// the line's indent (with any list dash) and the rest of the line.
    fn rewrite_lines(
        source: &str,
        mut replace: impl FnMut(&str, &str) -> Option<String>,
    ) -> String {
        let mut text = source
            .lines()
            .map(|line| {
                let key = line.trim_start().trim_start_matches("- ");
                let indent = &line[..line.len() - key.len()];
                replace(indent, key).unwrap_or_else(|| line.to_string())
            })
            .collect::<Vec<_>>()
            .join("\n");
        text.push('\n');
        text
    }
}

/// Hitbox values change in steps of the fine grid.
const HITBOX_STEP_H: f32 = FINE_SNAP_H;

impl Field {
    pub(super) fn is_hitbox(self) -> bool {
        matches!(
            self,
            Self::BodyHeight
                | Self::HurtboxWidth
                | Self::HurtboxHeight
                | Self::Attack
                | Self::HitboxX
                | Self::HitboxY
                | Self::HitboxWidth
                | Self::HitboxHeight
        )
    }

    /// The field's value for `enemy`'s type, or for the player without one;
    /// `attack` picks which of the type's attacks.
    pub(super) fn hitbox_value(
        self,
        data: &GameData,
        enemy: Option<&EnemyTypeConfig>,
        attack: usize,
    ) -> String {
        let hurtbox = enemy.map_or(data.gameplay.player.hurtbox_h, |enemy| enemy.hurtbox_h);
        let attacks = enemy
            .map(|enemy| {
                enemy
                    .attack
                    .iter()
                    .chain(&enemy.attacks)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let strike = attacks.get(attack);
        let hitbox = strike.and_then(|strike| strike.hitbox_h);
        let projectile = strike.and_then(|strike| strike.projectile_size_h);
        let shown = |value: Option<f32>| value.map_or_else(|| String::from("-"), |v| v.to_string());
        match self {
            Self::BodyHeight => data.player_animations.body_height_h.to_string(),
            Self::HurtboxWidth => hurtbox.width.to_string(),
            Self::HurtboxHeight => hurtbox.height.to_string(),
            Self::Attack => strike.map_or_else(
                || String::from("-"),
                |strike| format!("{}/{}  {}", attack + 1, attacks.len(), strike.kind),
            ),
            Self::HitboxX if projectile.is_some() => String::from("- (projectile)"),
            Self::HitboxY if projectile.is_some() => String::from("- (projectile)"),
            Self::HitboxX => shown(hitbox.map(|hitbox| hitbox.x)),
            Self::HitboxY => shown(hitbox.map(|hitbox| hitbox.y)),
            Self::HitboxWidth => shown(
                hitbox
                    .map(|hitbox| hitbox.width)
                    .or(projectile.map(|size| size.width)),
            ),
            Self::HitboxHeight => shown(
                hitbox
                    .map(|hitbox| hitbox.height)
                    .or(projectile.map(|size| size.height)),
            ),
            _ => String::new(),
        }
    }

    /// Steps the field for every enemy of `type_id` (the player's hurtbox
    /// and drawn size without one); `attack` picks which of the type's attacks. Width and
    /// height of a projectile attack size its projectile.
    pub(super) fn adjust_hitbox(
        self,
        data: &mut GameData,
        type_id: Option<&EnemyTypeId>,
        attack: usize,
        steps: f32,
    ) {
        let step = steps * HITBOX_STEP_H;
        let size = |value: &mut f32| *value = snap((*value + step).max(HITBOX_STEP_H), true);
        let offset = |value: &mut f32| *value = snap(*value + step, true);
        let Some(type_id) = type_id else {
            let hurtbox = &mut data.gameplay.player.hurtbox_h;
            match self {
                Self::BodyHeight => size(&mut data.player_animations.body_height_h),
                Self::HurtboxWidth => size(&mut hurtbox.width),
                Self::HurtboxHeight => size(&mut hurtbox.height),
                _ => {}
            }
            return;
        };
        let Some(enemy) = data.enemies.types.get_mut(type_id) else {
            return;
        };
        let strike = enemy
            .attack
            .iter_mut()
            .chain(enemy.attacks.iter_mut())
            .nth(attack);
        match (self, strike) {
            (Self::HurtboxWidth, _) => size(&mut enemy.hurtbox_h.width),
            (Self::HurtboxHeight, _) => size(&mut enemy.hurtbox_h.height),
            (
                Self::HitboxX | Self::HitboxY | Self::HitboxWidth | Self::HitboxHeight,
                Some(strike),
            ) => {
                if let Some(projectile) = &mut strike.projectile_size_h {
                    match self {
                        Self::HitboxWidth => size(&mut projectile.width),
                        Self::HitboxHeight => size(&mut projectile.height),
                        _ => {}
                    }
                } else if let Some(hitbox) = &mut strike.hitbox_h {
                    match self {
                        Self::HitboxX => offset(&mut hitbox.x),
                        Self::HitboxY => offset(&mut hitbox.y),
                        Self::HitboxWidth => size(&mut hitbox.width),
                        _ => size(&mut hitbox.height),
                    }
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hitbox_edits_undo_and_save_into_the_yaml_files_keeping_their_comments() {
        let asset_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/assets");
        let mut data = GameData::load(&asset_root).expect("Chapter 1 YAML should be valid");
        let original = Hitboxes::of(&data);
        let blade_unit = "blade_unit"
            .parse::<EnemyTypeId>()
            .expect("a valid type ID");
        let boss = "emitter_unit"
            .parse::<EnemyTypeId>()
            .expect("a valid type ID");

        // The player's size and hurtbox, a type's hurtbox, the boss's second
        // attack.
        Field::BodyHeight.adjust_hitbox(&mut data, None, 0, 3.0);
        let body_height_h = data.player_animations.body_height_h;
        assert!((body_height_h - original.player_body_height_h - 0.15).abs() < 1e-5);
        Field::HurtboxWidth.adjust_hitbox(&mut data, None, 0, 2.0);
        Field::HurtboxHeight.adjust_hitbox(&mut data, Some(&blade_unit), 0, -4.0);
        Field::HitboxX.adjust_hitbox(&mut data, Some(&boss), 1, 1.0);
        let player = data.gameplay.player.hurtbox_h.width;
        assert!((player - original.player.width - 0.1).abs() < 1e-5);
        let blade_unit_height = data.enemies.types[&blade_unit].hurtbox_h.height;
        let boss_x = data.enemies.types[&boss].attacks[1]
            .hitbox_h
            .expect("the lunge has a hitbox")
            .x;
        assert_eq!(boss_x, 0.05);
        let edited = Hitboxes::of(&data);
        assert!(edited != original);

        // Undo puts every hitbox back; redo brings the edits again.
        let redo = Snapshot::Hitboxes(original.clone())
            .restore(&mut data)
            .expect("hitboxes always restore");
        assert!(Hitboxes::of(&data) == original);
        redo.restore(&mut data);
        assert!(Hitboxes::of(&data) == edited);

        // Saving rewrites only the hitbox lines of a copy of the files.
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |elapsed| elapsed.as_nanos());
        let copy = std::env::temp_dir().join(format!("afterward-hitboxes-{nanos}"));
        for file in [
            "config/gameplay.yaml",
            "enemies/enemy_types.yaml",
            "player/animations.yaml",
        ] {
            fs::create_dir_all(copy.join(file).parent().expect("a folder"))
                .expect("the temp folder is writable");
            fs::copy(asset_root.join(file), copy.join(file)).expect("the YAML copies");
        }
        edited.save(&copy).expect("the hitboxes save");
        let enemies = fs::read_to_string(copy.join("enemies/enemy_types.yaml"))
            .expect("the saved file reads");
        let source = fs::read_to_string(asset_root.join("enemies/enemy_types.yaml"))
            .expect("the source file reads");
        assert!(enemies.contains("# Hitboxes, in H"), "comments survive");
        assert_eq!(enemies.lines().count(), source.lines().count());
        let saved = yaml_serde::from_str::<EnemyTypes>(&enemies).expect("valid YAML");
        assert_eq!(saved.types[&blade_unit].hurtbox_h.height, blade_unit_height);
        let gameplay =
            fs::read_to_string(copy.join("config/gameplay.yaml")).expect("the saved file reads");
        let saved = yaml_serde::from_str::<GameplayConfig>(&gameplay).expect("valid YAML");
        assert_eq!(saved.player.hurtbox_h.width, player);
        let animations =
            fs::read_to_string(copy.join("player/animations.yaml")).expect("the saved file reads");
        assert!(
            animations.contains("# Player sprite sheets"),
            "comments survive"
        );
        let saved = yaml_serde::from_str::<PlayerAnimations>(&animations).expect("valid YAML");
        assert_eq!(saved.body_height_h, body_height_h);
        fs::remove_dir_all(copy).expect("the temp folder is removable");
    }
}
