use super::*;
use crate::{
    data::{BonusPickupConfig, HealthPickupConfig},
    game::{bonus_pickup::BonusPickup, combat::HealthPickup},
};

mod boss_sheets;
mod effect_sheets;
mod enemy_sheets;
mod monster_sheets;
mod player_sheets;

pub(super) fn load_sprites(
    mut commands: Commands,
    data: Res<GameData>,
    asset_server: Res<AssetServer>,
    mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let specs = player_sheets::sprites(&data.player_animations)
        .into_iter()
        .chain(enemy_sheets::sprites())
        .chain(boss_sheets::sprites())
        .chain(monster_sheets::sprites())
        .chain(effect_sheets::sprites())
        .chain([
            exit_door_sheet(&data.gameplay.room_flow.exit_door),
            health_pickup_sheet(&data.gameplay.health_pickup),
            bonus_pickup_sheet(&data.gameplay.room_flow.bonus_pickup),
        ])
        .collect::<Vec<_>>();

    let mut sheets = BTreeMap::new();
    for spec in specs {
        // Sheets under `ui/`, `pickups/` and `effects/` are pixel art drawn
        // near 1:1, and the Black Knight and the monsters are pixel art drawn
        // larger than their sheets, so they keep their hard pixel edges; the
        // other character sheets are scaled down and look better with the
        // default filtering.
        let image = if spec.path.starts_with("ui/")
            || spec.path.starts_with("pickups/")
            || spec.path.starts_with("effects/")
            || spec.path.starts_with("enemies/black_knight/")
            || spec.path.starts_with("enemies/horror/")
            || spec.path.starts_with("enemies/eye_beast/")
        {
            crate::game::ui_style::load_pixel_art(&asset_server, spec.path)
        } else {
            asset_server.load(spec.path)
        };
        let layout = atlas_layouts.add(TextureAtlasLayout::from_grid(
            UVec2::new(spec.frame_width, spec.frame_height),
            spec.frames as u32,
            1,
            None,
            Some(spec.first_cell * UVec2::new(spec.frame_width, spec.frame_height)),
        ));
        sheets.insert(
            spec.key,
            SpriteSheet {
                image,
                layout,
                frame_size: Vec2::new(spec.frame_width as f32, spec.frame_height as f32),
                frames: spec.frames,
                fps: spec.fps,
            },
        );
    }
    commands.insert_resource(SpriteSheets(sheets));
}

/// The exit door's frames, shown by state (locked, unlocked, opening) rather
/// than played at a frame rate.
fn exit_door_sheet(door: &crate::data::ExitDoorConfig) -> SpriteSheetSpec {
    SpriteSheetSpec {
        key: ExitDoor::SHEET.into(),
        path: door.sheet.clone(),
        frame_width: door.frame_size_px.width,
        frame_height: door.frame_size_px.height,
        frames: door.frames,
        fps: 1.0,
        first_cell: UVec2::ZERO,
    }
}

/// The health drop an enemy may leave behind: one still frame, loaded with
/// every other sprite so laying one down never waits on the asset server.
fn health_pickup_sheet(pickup: &HealthPickupConfig) -> SpriteSheetSpec {
    SpriteSheetSpec {
        key: HealthPickup::SHEET.into(),
        path: pickup.sprite.clone(),
        frame_width: pickup.sprite_size_px.width,
        frame_height: pickup.sprite_size_px.height,
        frames: 1,
        fps: 1.0,
        first_cell: UVec2::ZERO,
    }
}

/// The obelisk a cleared room's bonus is taken from: one hovering loop, the
/// same for every category.
fn bonus_pickup_sheet(pickup: &BonusPickupConfig) -> SpriteSheetSpec {
    SpriteSheetSpec {
        key: BonusPickup::SHEET.into(),
        path: pickup.sheet.clone(),
        frame_width: pickup.frame_size_px.width,
        frame_height: pickup.frame_size_px.height,
        frames: pickup.frames,
        fps: pickup.fps,
        first_cell: UVec2::ZERO,
    }
}

pub(super) fn animate_sprites(
    time: Res<Time>,
    mut query: Query<(&mut Sprite, &mut AnimatedSprite)>,
) {
    for (mut sprite, mut animation) in &mut query {
        animation.timer.tick(time.delta());
        if !animation.timer.just_finished() {
            continue;
        }
        let Some(atlas) = sprite.texture_atlas.as_mut() else {
            continue;
        };
        if animation.frame + 1 == animation.frames {
            if animation.looping {
                animation.frame = 0;
            }
        } else {
            animation.frame += 1;
        }
        atlas.index = animation.frame;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, path::Path};

    #[test]
    fn every_clip_an_enemy_attack_names_is_a_sheet_long_enough_for_its_strike() {
        let asset_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/assets");
        let data = GameData::load(&asset_root).expect("game data should load");
        let frames = boss_sheets::sprites()
            .into_iter()
            .chain(enemy_sheets::sprites())
            .chain(monster_sheets::sprites())
            .map(|sheet| (sheet.key, sheet.frames))
            .collect::<BTreeMap<_, _>>();
        for (type_id, enemy) in &data.enemies.types {
            for attack in enemy.attack.iter().chain(&enemy.attacks) {
                let strikes = [
                    attack.clip.as_ref().map(|clip| (clip, attack.strike_frame)),
                    attack
                        .guard
                        .as_ref()
                        .map(|guard| (&guard.counter_clip, Some(guard.counter_strike_frame))),
                ];
                for (clip, strike_frame) in strikes.into_iter().flatten() {
                    let length = frames.get(&format!("{type_id}_{clip}")).copied();
                    assert!(
                        length.is_some_and(|length| strike_frame.unwrap_or(0) < length),
                        "{type_id} attack {} plays `{clip}`, which is not a sheet \
                         with a frame {strike_frame:?}",
                        attack.kind
                    );
                }
            }
        }
    }

    #[test]
    fn sprite_sheet_layouts_match_their_png_dimensions() {
        let asset_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/assets");
        let data = GameData::load(&asset_root).expect("game data should load");
        let specs = player_sheets::sprites(&data.player_animations)
            .into_iter()
            .chain(enemy_sheets::sprites())
            .chain(boss_sheets::sprites())
            .chain(monster_sheets::sprites())
            .chain(effect_sheets::sprites())
            .chain([
                exit_door_sheet(&data.gameplay.room_flow.exit_door),
                health_pickup_sheet(&data.gameplay.health_pickup),
                bonus_pickup_sheet(&data.gameplay.room_flow.bonus_pickup),
            ]);

        for spec in specs {
            let path = asset_root.join(&spec.path);
            let bytes = fs::read(&path)
                .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
            assert!(
                bytes.starts_with(b"\x89PNG\r\n\x1a\n"),
                "{} is not a PNG",
                path.display()
            );
            assert!(
                bytes.len() >= 24,
                "{} has a truncated PNG header",
                path.display()
            );
            assert_eq!(
                &bytes[12..16],
                b"IHDR",
                "{} has no IHDR chunk",
                path.display()
            );

            let image_width = u32::from_be_bytes(bytes[16..20].try_into().unwrap());
            let image_height = u32::from_be_bytes(bytes[20..24].try_into().unwrap());
            // Sheets are grids of whole frames, and a clip (which may read a
            // slice of one) must fit inside its grid.
            assert!(
                image_width % spec.frame_width == 0
                    && image_width >= spec.frame_width * (spec.first_cell.x + spec.frames as u32),
                "{} atlas width does not fit its configured frames",
                path.display()
            );
            assert!(
                image_height % spec.frame_height == 0
                    && image_height >= spec.frame_height * (spec.first_cell.y + 1),
                "{} atlas height does not fit its configured frame rows",
                path.display()
            );
        }
    }
}
